use crate::config::Settings;
use crate::worker::run_worker;
use nix::errno::Errno;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, Pid, fork};
use std::io::{self, Error};
use std::net::TcpListener;

pub fn run(listener: TcpListener, settings: Settings) -> io::Result<()> {
    eprintln!(
        "[parent] listening on {}, workers={}, poll_timeout_ms={}, read_chunk={}",
        settings.addr, settings.workers, settings.poll_timeout_ms, settings.read_chunk,
    );

    let children = spawn_workers(&listener, &settings)?;
    wait_all_children(children)
}

fn spawn_workers(listener: &TcpListener, settings: &Settings) -> io::Result<Vec<Pid>> {
    let mut children = Vec::with_capacity(settings.workers);

    // Prefork
    for _ in 0..settings.workers {
        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                let code = match run_worker(listener, settings) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("[worker] fatal error: {e}");
                        1
                    }
                };
                std::process::exit(code);
            }
            Ok(ForkResult::Parent { child }) => children.push(child),
            Err(e) => return Err(Error::from(e)),
        }
    }

    Ok(children)
}

fn wait_all_children(mut children: Vec<Pid>) -> io::Result<()> {
    while !children.is_empty() {
        match waitpid(None, None) {
            Ok(status) => match status {
                WaitStatus::Exited(pid, code) => {
                    eprintln!("[parent] child {pid} exited with status={code}");
                    children.retain(|&p| p != pid);
                }
                WaitStatus::Signaled(pid, sig, _) => {
                    eprintln!("[parent] child {pid} killed by {sig}");
                    children.retain(|&p| p != pid);
                }
                WaitStatus::Stopped(_, _)
                | WaitStatus::Continued(_)
                | WaitStatus::StillAlive => {}
            },
            Err(Errno::EINTR) => continue,
            Err(e) => return Err(Error::from(e)),
        }
    }

    Ok(())
}
