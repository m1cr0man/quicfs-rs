use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let out = "Hello world!";
    println!("{}", out);

    let listener = TcpListener::bind("localhost:8012").await.unwrap();

    let (tx, _rx) = broadcast::channel(10);

    // This loop allows us to accept multiple connections
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

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
                            write.write_all(&msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
