use crate::error::{return_err_boxed, Error, GeneralError};

use mio;
use quiche::{h3, Connection, ConnectionId};
use ring::rand::*;
use std::net::SocketAddr;
use url::Url;

const MAX_DATAGRAM_SIZE: usize = 1350;

const SOCKET_TOKEN: mio::Token = mio::Token(0);

fn hex_dump(buf: &[u8]) -> String {
    let vec: Vec<String> = buf.iter().map(|b| format!("{:02x}", b)).collect();

    vec.join("")
}

fn get_peer_addr(url: &Url) -> SocketAddr {
    let peer_addr = url.socket_addrs(|| Some(443)).unwrap();
    peer_addr.first().unwrap().clone()
}

// Returns total number of packets written, or an error
fn do_send(conn: &mut Connection, socket: &mio::net::UdpSocket) -> Result<usize, Error> {
    // TODO avoid this dynamic allocation
    let mut out = [0; MAX_DATAGRAM_SIZE];
    let mut total: usize = 0;

    loop {
        let (len, send_info) = match conn.send(&mut out) {
            Ok(data) => data,
            Err(quiche::Error::Done) => {
                // No more data to send.
                break;
            }
            Err(err) => return_err_boxed!(err),
        };

        // Send this data
        let len = match socket.send_to(&out[..len], &send_info.to) {
            Ok(v) => v,
            Err(err) => return_err_boxed!(err),
        };

        total += len;
    }

    Ok(total)
}

fn do_receive(conn: &mut Connection, socket: &mio::net::UdpSocket) -> Result<usize, Error> {
    // Read all we can from the socket
    // TODO avoid this dynamic allocation
    let mut buf = [0; 65535];
    let mut total: usize = 0;

    loop {
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

        let recv_info = quiche::RecvInfo { from };

        // Process potentially coalesced packets.
        let len = match conn.recv(&mut buf[..len], recv_info) {
            Ok(v) => v,

            Err(e) => {
                // TODO consider returning error here, and handling retry further up the stack
                error!("recv failed: {:?}", e);
                continue;
            }
        };

        debug!("processed {} bytes", len);
        total += len;
    }

    Ok(total)
}

pub struct QuicClient {
    url: Url,
    bind_addr: SocketAddr,
    poll: mio::Poll,
    socket: Option<mio::net::UdpSocket>,
    conn: Option<std::pin::Pin<Box<Connection>>>,
    http3_conn: Option<h3::Connection>,
}

impl QuicClient {
    pub fn connect(&mut self) -> Result<&mut Self, Error> {
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
                mio::Ready::readable() | mio::Ready::writable(),
                mio::PollOpt::edge(),
            )
            .unwrap();

        // Create the configuration for the QUIC connection.
        let mut config = crate::quic::get_config_client();

        // Generate a random source connection ID for the connection.
        let mut scid = [0; quiche::MAX_CONN_ID_LEN];
        SystemRandom::new().fill(&mut scid[..]).unwrap();

        let scid = ConnectionId::from_ref(&scid);
        let peer_addr = get_peer_addr(&self.url);

        let conn = quiche::connect(self.url.domain(), &scid, peer_addr, &mut config);
        let conn = match conn {
            Ok(conn) => conn,
            Err(err) => return_err_boxed!(err),
        };

        info!(
            "connecting to {:} from {:} with scid {}",
            self.url,
            socket.local_addr().unwrap(),
            hex_dump(&scid)
        );

        // TODO why was this commented out?
        // let writes = match do_send(&conn, &socket) {
        //     Some(w) => w,
        //     Err(err) => return Err(err),
        // };

        // debug!("Wrote {} packets during connection", writes);

        self.socket = Some(socket);
        self.conn = Some(conn);
        Ok(self)
    }

    fn process_events(&mut self) -> Result<usize, Error> {
        let conn = self.conn.as_mut().unwrap();
        let http3_conn = self.http3_conn.as_mut().unwrap();

        // Process HTTP/3 events.
        loop {
            match http3_conn.poll(conn) {
                Ok((stream_id, h3::Event::Headers { list, .. })) => {
                    info!("got response headers {:?} on stream id {}", list, stream_id);
                }

                Ok((stream_id, h3::Event::Data)) => {
                    // TODO avoid this dynamic allocation
                    let mut buf = [0; 65535];

                    while let Ok(read) = http3_conn.recv_body(conn, stream_id, &mut buf) {
                        debug!(
                            "got {} bytes of response data on stream {}",
                            read, stream_id
                        );

                        print!("{}", unsafe { std::str::from_utf8_unchecked(&buf[..read]) });
                    }
                }

                Ok((_stream_id, h3::Event::Finished)) => {
                    info!("response received, closing...");

                    conn.close(true, 0x00, b"kthxbye").unwrap();
                }

                Ok((_stream_id, h3::Event::Reset(e))) => {
                    error!("request was reset by peer with {}, closing...", e);

                    conn.close(true, 0x00, b"kthxbye").unwrap();
                }

                Ok((_flow_id, h3::Event::Datagram)) => (),

                Ok((goaway_id, h3::Event::GoAway)) => {
                    info!("GOAWAY id={}", goaway_id);
                }

                Err(h3::Error::Done) => {}

                Err(err) => {
                    error!("HTTP/3 processing failed: {:?}", err);
                }
            }
        }
    }

    fn run(&mut self) -> Result<usize, Error> {
        match self.conn {
            None => {
                if let Err(err) = self.connect() {
                    return Err(err);
                }
            }
            _ => {}
        }

        let socket = self.socket.as_ref().unwrap();
        let poll = &mut self.poll;
        let conn = self.conn.as_mut().unwrap();

        let events = &mut mio::Events::with_capacity(1);

        loop {
            poll.poll(events, conn.timeout()).unwrap();

            if events.is_empty() {
                conn.on_timeout();
                let err = GeneralError::from("Timed out waiting for events");
                return_err_boxed!(err);
            }

            // There should only ever be one event, and it should only ever
            // be our SOCKET_TOKEN, but do it the right way regardless.
            let mut is_readable = false;
            let mut is_writable = false;
            for event in events.iter() {
                match event.token() {
                    SOCKET_TOKEN => {
                        is_readable = event.readiness().is_readable();
                        // TODO check if is_writable ever becomes true
                        is_writable = is_readable || event.readiness().is_writable();
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
                if let Err(err) = do_receive(conn, &socket) {
                    return Err(err);
                }
            }

            // Processed received data before doing any sending
            if let Some(http3_conn) = &mut self.http3_conn {
                // TODO create a queue of requests, and send them here
            } else if conn.is_established() {
                // Once QUIC is established, start HTTP/3
                let http3_config = h3::Config::new().unwrap();
                self.http3_conn = match h3::Connection::with_transport(conn, &http3_config) {
                    Ok(conn) => Some(conn),
                    Err(err) => return_err_boxed!(err),
                }
            }

            if is_writable {
                if let Err(err) = do_send(conn, &socket) {
                    return Err(err);
                }
            }

            if conn.is_closed() {
                info!("Connection closed, {:?}", conn.stats());
                break;
            }
        }

        Ok(0)
    }
}

impl From<Url> for QuicClient {
    fn from(url: Url) -> Self {
        // Resolve server address.
        // The port number doesn't actually matter in this context, but will
        // prevent a panic on unwrap.
        let peer_addr = get_peer_addr(&url);

        // Bind to INADDR_ANY or IN6ADDR_ANY depending on the IP family of the
        // server address. This is needed on macOS and BSD variants that don't
        // support binding to IN6ADDR_ANY for both v4 and v6.
        let bind_addr: SocketAddr = match peer_addr {
            SocketAddr::V4(_) => "0.0.0.0:0",
            SocketAddr::V6(_) => "[::]:0",
        }
        .parse()
        .unwrap();

        let poll = mio::Poll::new().unwrap();

        Self {
            url,
            bind_addr,
            poll,
            socket: None,
            conn: None,
            http3_conn: None,
        }
    }
}
