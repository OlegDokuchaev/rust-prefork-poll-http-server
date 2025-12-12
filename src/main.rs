mod config;
mod conn;
mod handler;
mod http;
mod page;
mod server;
mod worker;

use std::io;
use std::net::TcpListener;

fn main() -> io::Result<()> {
    let settings = config::Settings::load().map_err(io::Error::other)?;

    page::init_html(&settings.html_path)?;

    let listener = TcpListener::bind(&settings.addr)?;
    listener.set_nonblocking(true)?;

    server::run(listener, settings)
}
