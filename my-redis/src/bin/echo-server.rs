use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("accept socketaddr {:?}", addr);
        tokio::spawn(async move {
            let mannually = true;
            if !mannually {
                let (mut rd, mut wr) = socket.split();

                if io::copy(&mut rd, &mut wr).await.is_err() {
                    eprintln!("failed to copy")
                }
            } else {
                // mannually copy
                let mut buf = vec![0; 1024];

                loop {
                    match socket.read(&mut buf).await {
                        // return 0 signifies the remote has closed
                        Ok(0) => return,
                        Ok(n) => {
                            println!("GOT clinet {:?}", std::str::from_utf8(&buf[..n]));
                            if socket.write_all(&buf[..n]).await.is_err() {
                                return;
                            }
                        }
                        Err(_) => {
                            return;
                        }
                    }
                }
            }
        });
    }
}
