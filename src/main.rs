use std::io;

pub mod api;
pub mod solver;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut server = api::Server::bind("0.0.0.0:1984").await?;
    server.run().await;
    Ok(())
}
