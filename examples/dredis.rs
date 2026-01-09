use anyhow::Result;
use std::{io, net::SocketAddr};
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let addr = "0.0.0.0:6379";

    // 绑定一个 TCP 监听器
    let listener = TcpListener::bind(addr).await?;
    info!("Dredis: listening on: {}", addr);

    loop {
        // 执行阻塞操作时，等待新的连接到达
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);

        // 为每个连接启动一个新的任务
        tokio::spawn(async move {
            // 处理连接
            if let Err(e) = process_redis_conn(stream, raddr).await {
                warn!("Error processing conn with {}: {:?}", raddr, e);
            }
        });
    }
}

async fn process_redis_conn(mut stream: tokio::net::TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        // 等待直到流可读
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);

        match stream.try_read_buf(&mut buf) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                info!("read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("{:?}", line);
                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    warn!("Connection {} closed", raddr);
    Ok(())
}
