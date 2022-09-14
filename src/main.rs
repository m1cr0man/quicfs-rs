use clap::Parser;
use tokio::{
    io::{stdin, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

mod cli;

async fn server(listen_addr: &String) {
    let listener = TcpListener::bind(listen_addr).await.unwrap();

    let (tx, _rx) = broadcast::channel(10);

    // This loop allows us to accept multiple connections
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("{} connected", addr);

        let tx = tx.clone();
        // Quirk: Clone the rx from the tx, rather than the original rx
        let mut rx = tx.subscribe();

        // Spawn a new thread to handle this connection
        // async move allows us to create a future, avoids simply writing a new function
        // Kind of like an async lambda I guess?
        tokio::spawn(async move {
            // Can't be in the loop due to the move
            let (read, mut write) = socket.split();

            // Note: BufReader owns the entire socket if we didn't split.
            // Hence we only pass the relevant half.
            let mut reader = BufReader::new(read);
            let mut line = String::new();

            loop {
                // Use select so we can read + write at the same time
                tokio::select! {
                    // Note: await is implicit
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();

                        // Wipe the line buffer each time
                        line.clear();
                    }

                    result = rx.recv() => {
                        let (msg, recv_addr) = result.unwrap();
                        // Don't repeat what the current connection sent
                        if recv_addr != addr {
                            // line.as_bytes -> provides underlying bytes
                            write.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }

            // Clean shutdown
            println!("{} disconnected", addr);
            socket.shutdown().await.unwrap();
        });
    }
}

async fn client(server_addr: &String) {
    let mut conn = TcpStream::connect(server_addr).await.unwrap();

    // No real need to split but we do it anyway in case things get more complex.
    let (read, mut write) = conn.split();

    let mut reader = BufReader::new(read);
    let mut line = String::new();

    // We can read stdin like any other object implementing AsyncRead
    let input = stdin();
    let mut input_reader = BufReader::new(input);
    let mut msg = String::new();

    loop {
        tokio::select! {
            _result = reader.read_line(&mut line) => {
                print!("{}", line);

                line.clear();
            }

            result = input_reader.read_line(&mut msg) => {
                // ctrl+d
                if result.unwrap() == 0 {
                    break;
                }

                write.write(msg.as_bytes()).await.unwrap();

                msg.clear();
            }
        }
    }

    // Clean shutdown
    println!("Bye!");
    conn.shutdown().await.unwrap();
}

#[tokio::main]
async fn main() {
    let out = "Hello world!";
    println!("{}", out);

    let cli = cli::QuicFSCli::parse();

    match &cli.command {
        Some(cli::Commands::Server { listen, serve: _ }) => {
            return server(listen).await;
        }
        Some(cli::Commands::Client {
            server: server_addr,
            src: _,
            dest: _,
        }) => {
            return client(server_addr).await;
        }
        None => {
            println!("Specify a command")
        }
    }
}
