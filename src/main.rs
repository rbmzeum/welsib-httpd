mod server;
mod dispatcher;
mod resource;
mod context;
mod channel;
mod net;
mod helpers;
mod signature;
mod point;
mod config;
mod conv;
mod api;
mod welsib;
mod checksum;

use server::Server;

fn main() -> std::io::Result<()> {
    let mut server = Server::new()?;
    server.run()
}