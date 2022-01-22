use crate::error::{return_err_boxed, Error, GeneralError};

use mio;
use quiche::h3::NameValue;
use quiche::Connection;
use ring::{hmac, rand::*};
use std::{collections::HashMap, net::SocketAddr};

const SOCKET_TOKEN: mio::Token = mio::Token(0);
const MAX_DATAGRAM_SIZE: usize = 1350;

type UDPBuffer = [u8; MAX_DATAGRAM_SIZE];

struct PartialResponse {
    headers: Option<Vec<quiche::h3::Header>>,

    body: Vec<u8>,

    written: usize,
}

enum ClientState {
    Initial,
    VersionNegotiate,
    ValidateToken,
    HttpInitiate,
    Data,
    Error,
    Shutdown,
}

struct Client {
    // Don't know if I need to box this... probably not.
    conn: std::pin::Pin<Box<quiche::Connection>>,

    http3_conn: Option<Box<quiche::h3::Connection>>,

    partial_responses: HashMap<u64, PartialResponse>,

    state: ClientState,
}

impl Client {
    pub fn new(conn: std::pin::Pin<Box<Connection>>) -> Self {
        Self {
            conn,
            http3_conn: None,
            partial_responses: HashMap::new(),
            state: ClientState::HttpInitiate,
        }
    }

    pub fn handle_data(
        &mut self,
        recv_buf: &mut [u8],
        recv_info: quiche::RecvInfo,
    ) -> Result<usize, Error> {
        // Process potentially coalesced packets.
        let read_len = match self.conn.recv(recv_buf, recv_info) {
            Ok(val) => val,
            Err(err) => return_err_boxed!(err),
        };

        debug!("{} processed {} bytes", self.conn.trace_id(), read_len);

        // Create a new HTTP/3 connection as soon as the QUIC connection
        // is established.
        match self.http3_conn {
            Some(http_conn) => {
                if let Err(err) = self.handle_http3_data(http_conn) {
                    return Err(err);
                }
            }
            None => {
                if let Err(err) = self.handle_http3_initiate() {
                    return Err(err);
                }
            }
        }

        Ok(read_len)
    }

    pub fn handle_http3_initiate(&mut self) -> Result<(), Error> {
        if self.conn.is_in_early_data() || self.conn.is_established() {
            debug!(
                "{} QUIC handshake completed, now trying HTTP/3",
                self.conn.trace_id()
            );

            let h3_config = quiche::h3::Config::new().unwrap();
            let h3_conn = match quiche::h3::Connection::with_transport(&mut self.conn, &h3_config) {
                Ok(val) => val,
                Err(err) => return_err_boxed!(err),
            };

            // TODO: sanity check h3 connection before adding to map
            self.http3_conn = Some(Box::new(h3_conn));
            self.state = ClientState::Data;
        } else {
            self.state = ClientState::Error;
            return Err(Box::new(GeneralError {
                message: format!(
                    "Client {} has no http3_conn and is not in early data.",
                    self.conn.trace_id()
                ),
            }));
        }

        Ok(())
    }

    pub fn handle_http3_data(
        &mut self,
        http3_conn: Box<quiche::h3::Connection>,
    ) -> Result<(), Error> {
        // Handle writable streams.
        for stream_id in self.conn.writable() {
            self.handle_writable(http3_conn, stream_id);
        }

        // Process HTTP/3 events.
        loop {
            match http3_conn.poll(&mut self.conn) {
                Ok((stream_id, quiche::h3::Event::Headers { list, .. })) => {
                    self.handle_request(http3_conn, stream_id, &list, "examples/root");
                }

                Ok((stream_id, quiche::h3::Event::Data)) => {
                    info!(
                        "{} got data on stream id {}",
                        self.conn.trace_id(),
                        stream_id
                    );
                }

                Ok((_stream_id, quiche::h3::Event::Finished)) => (),

                Ok((_stream_id, quiche::h3::Event::Reset { .. })) => (),

                Ok((_flow_id, quiche::h3::Event::Datagram)) => (),

                Ok((_goaway_id, quiche::h3::Event::GoAway)) => (),

                Err(quiche::h3::Error::Done) => {
                    self.state = ClientState::Shutdown;
                    break;
                }

                Err(err) => {
                    return Err(Box::new(GeneralError {
                        message: format!("Client {} HTTP/3 error {:?}", self.conn.trace_id(), err),
                    }));
                }
            }
        }

        Ok(())
    }

    /// Handles incoming HTTP/3 requests.
    fn handle_request(
        &mut self,
        http3_conn: Box<quiche::h3::Connection>,
        stream_id: u64,
        headers: &[quiche::h3::Header],
        root: &str,
    ) {
        let conn = &mut self.conn;

        info!(
            "{} got request {:?} on stream id {}",
            conn.trace_id(),
            headers,
            stream_id
        );

        // We decide the response based on headers alone, so stop reading the
        // request stream so that any body is ignored and pointless Data events
        // are not generated.
        conn.stream_shutdown(stream_id, quiche::Shutdown::Read, 0)
            .unwrap();

        let (headers, body) = build_response(root, headers);

        match http3_conn.send_response(conn, stream_id, &headers, false) {
            Ok(v) => v,

            Err(quiche::h3::Error::StreamBlocked) => {
                // TODO I think it would be much easier to make a queue of responses and make all attempts
                // to send them in one place. Building responses and sending them should be handled separately.
                let response = PartialResponse {
                    headers: Some(headers),
                    body,
                    written: 0,
                };

                self.partial_responses.insert(stream_id, response);
                return;
            }

            Err(e) => {
                error!("{} stream send failed {:?}", conn.trace_id(), e);
                return;
            }
        }

        let written = match http3_conn.send_body(conn, stream_id, &body, true) {
            Ok(v) => v,

            Err(quiche::h3::Error::Done) => 0,

            Err(e) => {
                error!("{} stream send failed {:?}", conn.trace_id(), e);
                return;
            }
        };

        if written < body.len() {
            let response = PartialResponse {
                headers: None,
                body,
                written,
            };

            self.partial_responses.insert(stream_id, response);
        }
    }

    /// Handles newly writable streams.
    fn handle_writable(&mut self, http3_conn: Box<quiche::h3::Connection>, stream_id: u64) {
        let conn = &mut self.conn;

        debug!("{} stream {} is writable", conn.trace_id(), stream_id);

        if !self.partial_responses.contains_key(&stream_id) {
            return;
        }

        let resp = self.partial_responses.get_mut(&stream_id).unwrap();

        if let Some(ref headers) = resp.headers {
            match http3_conn.send_response(conn, stream_id, headers, false) {
                Ok(_) => (),

                Err(quiche::h3::Error::StreamBlocked) => {
                    return;
                }

                Err(e) => {
                    error!("{} stream send failed {:?}", conn.trace_id(), e);
                    return;
                }
            }
        }

        resp.headers = None;

        let body = &resp.body[resp.written..];

        let written = match http3_conn.send_body(conn, stream_id, body, true) {
            Ok(v) => v,

            Err(quiche::h3::Error::Done) => 0,

            Err(e) => {
                self.partial_responses.remove(&stream_id);

                error!("{} stream send failed {:?}", conn.trace_id(), e);
                return;
            }
        };

        resp.written += written;

        if resp.written == resp.body.len() {
            self.partial_responses.remove(&stream_id);
        }
    }
}

type ClientMap<'a> = HashMap<quiche::ConnectionId<'a>, Client>;

pub struct QuicServer<'a> {
    bind_addr: SocketAddr,
    poll: mio::Poll,
    conn_id_key: hmac::Key,
    clients: ClientMap<'a>,
    send_buf: UDPBuffer,
}

impl<'a> QuicServer<'a> {
    // This function returns a slice of self.send_buf to avoid it being mutated before the
    // data is sent.
    fn handle_new_client(
        &mut self,
        header: &'a quiche::Header,
        from: SocketAddr,
    ) -> Result<&[u8], Error> {
        // All new connections (aka unknown clients) should start with Initial packet.
        if header.ty != quiche::Type::Initial {
            let err = GeneralError::from("Packet is not Initial");
            return_err_boxed!(err);
        }

        // Handle version negotiation
        if !quiche::version_is_supported(header.version) {
            warn!("Doing version negotiation");

            // Generate a version negotiation packet
            match quiche::negotiate_version(&header.scid, &header.dcid, &mut self.send_buf) {
                Ok(v) => {
                    return Ok(&self.send_buf[..v]);
                }
                Err(err) => {
                    return_err_boxed!(err);
                }
            }
        }

        // Token should always present in Initial packets.
        let token = header.token.as_ref().unwrap();

        // If it's not we need to do a stateless retry
        if token.is_empty() {
            warn!(
                "{:?} sent an initial packet with no token. Sending stateless retry.",
                from.to_string()
            );

            // Sign the client's DCID, generating a new one from the resulting HMAC.
            // This will be the SCID we give the client.
            let scid = ring::hmac::sign(&self.conn_id_key, &header.dcid);
            let scid_bytes = &scid.as_ref()[..quiche::MAX_CONN_ID_LEN];
            let scid = quiche::ConnectionId::from_ref(&scid_bytes);

            // TODO generate a token that we can validate later. mint_token is a demo function.
            let new_token = mint_token(&header, &from);

            match quiche::retry(
                &header.scid,
                &header.dcid,
                &scid,
                &new_token,
                header.version,
                &mut self.send_buf,
            ) {
                Ok(v) => {
                    return Ok(&self.send_buf[..v]);
                }
                Err(err) => {
                    return_err_boxed!(err);
                }
            }
        }

        let odcid = validate_token(&from, token);

        // The token was not valid, meaning the retry failed, so
        // drop the packet.
        if odcid.is_none() {
            let err = GeneralError::from("Invalid address validation token");
            return_err_boxed!(err);
        }

        // Validate that the DCID is of a length that we would have given.
        // This seems like a really weak security check?
        // if scid.len() != header.dcid.len() {
        //     let err = GeneralError::from("Invalid destination connection ID");
        //     return_err_boxed!(err);
        // }

        // At this point, we can trust the dcid from the client
        // and reuse it, establishing a connection.
        let scid = header.dcid.clone();

        println!("New connection: dcid={:?} scid={:?}", scid, header.scid);

        // Since config is borrowed mutably, lets generate one per connection
        // TODO this might not be necessary, looking at the quiche examples.
        let config = crate::quic::get_config_server();
        let conn = quiche::accept(&scid, odcid.as_ref(), from, &mut config).unwrap();

        self.clients.insert(scid, Client::new(conn));

        // TODO this function returns the number of bytes to send back to the client
        // from send_buf. If it is 0, and there was no error, self.clients[header.dcid]
        // is guaranteed to exist.
        // - Handle errors from this function and sending of data
        // - Continue implementing client state machine

        Ok(&[])
    }

    fn do_receive(&mut self, socket: &mio::net::UdpSocket) -> Result<usize, Error> {
        // Read all we can from the socket
        // TODO avoid this dynamic allocation
        let mut buf = [0; 65535];
        let mut total: usize = 0;

        'process_data: loop {
            let (len, from) = match socket.recv_from(&mut buf) {
                Ok(v) => v,

                Err(err) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                        // The event loop will tell us later when we can do I/O again
                        debug!("Socket I/O would block");
                        break;
                    }
                    return_err_boxed!(err);
                }
            };

            debug!("got {} bytes", len);

            // Take the slice of data read
            let recv_buf = &mut buf[..len];

            let recv_info = quiche::RecvInfo { from };

            // Try and parse a QUIC header from this data
            // In the future, delegate parsing this data to another thread
            let header = match quiche::Header::from_slice(recv_buf, quiche::MAX_CONN_ID_LEN) {
                Ok(new_hdr) => new_hdr,

                Err(err) => {
                    error!("Parsing packet header failed: {:?}", err);
                    continue;
                }
            };

            println!("Got packet {:?}", header);

            // Read the destination connection ID, AKA our identifier
            // for established clients, or a randomly generated one otherwise.
            // MOVED to handle_new_client, use header.dcid instead
            // let rng: ring::rand::SystemRandom = SystemRandom::new();
            // let conn_id_seed: ring::hmac::Key =
            // ring::hmac::Key::generate(ring::hmac::HMAC_SHA256, &rng).unwrap();
            // let conn_id = ring::hmac::sign(&conn_id_seed, &header.dcid);
            // let conn_id_bytes = &conn_id.as_ref()[..quiche::MAX_CONN_ID_LEN];
            // let conn_id: ConnectionId = conn_id_bytes.to_vec().into();

            // Try and find a matching client connection, or create a new connection.
            let client = if let Some(cli) = self.clients.get_mut(&header.dcid) {
                cli
            // TODO why was this being done too?
            // } else if let Some(cli) = clients.get_mut(&conn_id) {
            //     cli
            } else {
                let val = match self.handle_new_client(&header, from) {
                    Ok(send_buf) => {
                        if let Some(cli) = self.clients.get_mut(&header.dcid) {
                            cli
                        } else {
                            // Assume at this point that send_buf > 0, as guaranteed by handle_new_client
                            if let Err(err) = Self::do_socket_send(socket, from, send_buf) {
                                return Err(err);
                            }
                            continue 'process_data;
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                };
                val
            };

            let recv_info = quiche::RecvInfo { from };

            if let Err(err) = client.handle_data(recv_buf, recv_info) {
                error!(
                    "Client {} failed to handle data: {:?}",
                    client.conn.trace_id(),
                    err
                );
                continue 'process_data;
            }

            // TODO handle sending data after client.handle_data
            // Also handle shutdown properly

            total += len;
        }

        Ok(total)
    }

    fn do_send(&mut self, socket: &mio::net::UdpSocket) -> Result<usize, Error> {
        let mut out = [0; MAX_DATAGRAM_SIZE];
        for client in self.clients.values_mut() {
            loop {
                // Generate the output data from the QUIC protocol
                let (out_len, send_info) = match client.conn.send(&mut out) {
                    Ok(v) => v,

                    Err(quiche::Error::Done) => {
                        debug!("{} done writing", client.conn.trace_id());
                        break;
                    }

                    Err(err) => {
                        error!("{} send failed: {:?}", client.conn.trace_id(), err);
                        client.conn.close(false, 0x1, b"fail").ok();
                        break;
                    }
                };

                if let Err(err) = Self::do_socket_send(socket, send_info.to, &out[..out_len]) {
                    // If a socket send fails, it's unlikely to succeed again
                    return Err(err);
                }
            }
        }
        Ok(0)
    }

    fn do_socket_send(
        socket: &mio::net::UdpSocket,
        to: SocketAddr,
        send_buf: &[u8],
    ) -> Result<usize, Error> {
        let mut bytes_sent = 0;

        // Send as much of the data as possible
        while bytes_sent < send_buf.len() {
            match socket.send_to(&send_buf[bytes_sent..], &to) {
                Ok(new_sent) => {
                    bytes_sent += new_sent;
                }
                Err(err) => {
                    return_err_boxed!(err);
                }
            }
        }

        Ok(bytes_sent)
    }

    // Taking self instead of &mut self because we don't expect this
    // struct to be reusable after being run.
    fn run(mut self) -> Result<u8, Error> {
        // START SERVER SETUP
        // Try to bind to our bind_addr early. It's the most likely part to fail
        let socket = match mio::net::UdpSocket::bind(&self.bind_addr) {
            Ok(s) => s,
            Err(err) => return_err_boxed!(err),
        };

        // Register the socket with the event loop.
        self.poll
            .register(
                &socket,
                SOCKET_TOKEN,
                mio::Ready::readable(),
                mio::PollOpt::edge(),
            )
            .unwrap();

        // We only have one thing to listen to for events
        let mut events = mio::Events::with_capacity(1);
        // END SERVER SETUP

        // There should only ever be one event, and it should only ever
        // be our SOCKET_TOKEN, but do it the right way regardless.
        let mut is_readable;
        let mut is_writable;
        loop {
            // TODO check timeouts from all clients
            self.poll.poll(&mut events, None).unwrap();

            is_readable = false;
            is_writable = false;

            for event in events.iter() {
                match event.token() {
                    SOCKET_TOKEN => {
                        is_readable = event.readiness().is_readable();
                        is_writable = event.readiness().is_writable();
                    }

                    _ => {
                        let err = GeneralError {
                            message: format!("Unexpected event in loop: {:?}", event),
                        };
                        return_err_boxed!(err);
                    }
                }
            }

            if is_readable {
                if let Err(err) = self.do_receive(&socket) {
                    return Err(err);
                }
            }

            if is_writable {
                // if let Err(err) = self.do_send()
            }
        }
    }
}

impl<'a> From<SocketAddr> for QuicServer<'a> {
    fn from(bind_addr: SocketAddr) -> Self {
        let poll = mio::Poll::new().unwrap();

        let rng = SystemRandom::new();
        let conn_id_key = ring::hmac::Key::generate(ring::hmac::HMAC_SHA256, &rng).unwrap();
        let clients = ClientMap::new();

        Self {
            bind_addr,
            poll,
            conn_id_key,
            clients,
            send_buf: [0; MAX_DATAGRAM_SIZE],
        }
    }
}

fn build_response(
    root: &str,
    request: &[quiche::h3::Header],
) -> (Vec<quiche::h3::Header>, Vec<u8>) {
    let mut file_path = std::path::PathBuf::from(root);
    let mut path = std::path::Path::new("");
    let mut method = None;

    // Look for the request's path and method.
    for hdr in request {
        match hdr.name() {
            b":path" => path = std::path::Path::new(std::str::from_utf8(hdr.value()).unwrap()),

            b":method" => method = Some(hdr.value()),

            _ => (),
        }
    }

    let (status, body) = match method {
        Some(b"GET") => {
            for c in path.components() {
                if let std::path::Component::Normal(v) = c {
                    file_path.push(v)
                }
            }

            match std::fs::read(file_path.as_path()) {
                Ok(data) => (200, data),

                Err(_) => (404, b"Not Found!".to_vec()),
            }
        }

        _ => (405, Vec::new()),
    };

    let headers = vec![
        quiche::h3::Header::new(b":status", status.to_string().as_bytes()),
        quiche::h3::Header::new(b"server", b"quiche"),
        quiche::h3::Header::new(b"content-length", body.len().to_string().as_bytes()),
    ];

    (headers, body)
}

/// Generate a stateless retry token.
///
/// The token includes the static string `"quiche"` followed by the IP address
/// of the client and by the original destination connection ID generated by the
/// client.
///
/// Note that this function is only an example and doesn't do any cryptographic
/// authenticate of the token. *It should not be used in production system*.
fn mint_token(hdr: &quiche::Header, src: &std::net::SocketAddr) -> Vec<u8> {
    let mut token = Vec::new();

    token.extend_from_slice(b"quiche");

    let addr = match src.ip() {
        std::net::IpAddr::V4(a) => a.octets().to_vec(),
        std::net::IpAddr::V6(a) => a.octets().to_vec(),
    };

    token.extend_from_slice(&addr);
    token.extend_from_slice(&hdr.dcid);

    token
}

/// Validates a stateless retry token.
///
/// This checks that the ticket includes the `"quiche"` static string, and that
/// the client IP address matches the address stored in the ticket.
///
/// Note that this function is only an example and doesn't do any cryptographic
/// authenticate of the token. *It should not be used in production system*.
fn validate_token<'a>(
    src: &std::net::SocketAddr,
    token: &'a [u8],
) -> Option<quiche::ConnectionId<'a>> {
    if token.len() < 6 {
        return None;
    }

    if &token[..6] != b"quiche" {
        return None;
    }

    let token = &token[6..];

    let addr = match src.ip() {
        std::net::IpAddr::V4(a) => a.octets().to_vec(),
        std::net::IpAddr::V6(a) => a.octets().to_vec(),
    };

    if token.len() < addr.len() || &token[..addr.len()] != addr.as_slice() {
        return None;
    }

    Some(quiche::ConnectionId::from_ref(&token[addr.len()..]))
}
