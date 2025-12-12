use crate::config::Settings;
use crate::conn::Conn;
use nix::errno::Errno;
use nix::poll::{PollFd, PollFlags, poll};
use std::collections::HashMap;
use std::io::{self, Error};
use std::net::TcpListener;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::path::PathBuf;

pub fn run_worker(listener: &TcpListener, settings: &Settings) -> io::Result<()> {
    let mut conns: HashMap<RawFd, Conn> = HashMap::new();
    let lfd = listener.as_raw_fd();

    loop {
        // Use poll
        let mut pollfds = Vec::with_capacity(1 + conns.len());

        unsafe {
            pollfds.push(PollFd::new(BorrowedFd::borrow_raw(lfd), PollFlags::POLLIN));

            for (&fd, conn) in &conns {
                let mut events = PollFlags::POLLIN;
                if conn.has_pending_write() {
                    events |= PollFlags::POLLOUT;
                }
                pollfds.push(PollFd::new(BorrowedFd::borrow_raw(fd), events));
            }
        }

        let nready = match poll(&mut pollfds, settings.poll_timeout_ms) {
            Ok(n) => n,
            Err(Errno::EINTR) => continue,
            Err(e) => return Err(Error::from(e)),
        };
        if nready == 0 {
            continue;
        }

        if let Some(rev) = pollfds[0].revents()
            && rev.contains(PollFlags::POLLIN)
        {
            accept_all(
                listener,
                &mut conns,
                settings.read_chunk,
                &settings.doc_root,
            )?;
        }

        for pfd in pollfds.iter().skip(1) {
            let rev = match pfd.revents() {
                Some(r) if !r.is_empty() => r,
                _ => continue,
            };

            let fd = pfd.as_fd().as_raw_fd();
            let Some(conn) = conns.get_mut(&fd) else {
                continue;
            };

            if rev.intersects(PollFlags::POLLERR | PollFlags::POLLHUP | PollFlags::POLLNVAL) {
                conns.remove(&fd);
                continue;
            }

            if rev.contains(PollFlags::POLLIN) && !conn.on_read()? {
                conns.remove(&fd);
                continue;
            }

            if rev.contains(PollFlags::POLLOUT) && !conn.on_write()? {
                conns.remove(&fd);
                continue;
            }
        }
    }
}

fn accept_all<'a>(
    listener: &TcpListener,
    conns: &mut HashMap<RawFd, Conn<'a>>,
    read_chunk: usize,
    doc_root: &'a PathBuf,
) -> io::Result<()> {
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                stream.set_nonblocking(true)?;
                let fd = stream.as_raw_fd();
                eprintln!("[worker] accepted {addr}");
                conns.insert(fd, Conn::new(stream, read_chunk, doc_root));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
