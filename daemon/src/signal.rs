use std::sync::atomic::{AtomicBool, Ordering};

pub static SIGNAL_EXIT: AtomicBool = AtomicBool::new(false);

extern "C" fn sigexit_handler(_signo: i32) {
    SIGNAL_EXIT.store(true, Ordering::SeqCst);
}

pub fn install_signal_handlers() {
    use std::mem;
    unsafe {
        let mut act: libc::sigaction = mem::zeroed();
        libc::sigemptyset(&mut act.sa_mask);
        act.sa_flags = 0;
        act.sa_sigaction = sigexit_handler as *const () as usize;
        libc::sigaction(libc::SIGINT, &act, std::ptr::null_mut());
        libc::sigaction(libc::SIGTERM, &act, std::ptr::null_mut());
    }
}

pub fn ensure_single_instance(name: &str) {
    let uid = unsafe { libc::getuid() };
    let path_str = format!("/tmp/{}-{}.lock", name, uid);
    let path = std::path::Path::new(&path_str);
    let path_c = std::ffi::CString::new(path_str.as_str()).unwrap();
    let fd = unsafe { libc::open(path_c.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600) };
    if fd < 0 {
        eprintln!("[{}] Failed to open lock file", name);
        std::process::exit(1);
    }
    let res = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if res == 0 {
        let pid = unsafe { libc::getpid() };
        let _ = std::fs::write(path, format!("{}", pid));
    }
    if res < 0 {
        let err = unsafe { *libc::__errno_location() };
        if err == libc::EAGAIN || err == libc::EWOULDBLOCK {
            if let Ok(pid_str) = std::fs::read_to_string(path) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    let alive = unsafe { libc::kill(pid, 0) } == 0;
                    if !alive {
                        eprintln!(
                            "[{}] Stale lock from PID {}, removing and retrying...",
                            name, pid
                        );
                        unsafe { libc::close(fd) };
                        let _ = std::fs::remove_file(path);
                        let path_c2 = std::ffi::CString::new(path_str.as_str()).unwrap();
                        let fd2 = unsafe {
                            libc::open(path_c2.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600)
                        };
                        if fd2 >= 0 {
                            let res2 = unsafe { libc::flock(fd2, libc::LOCK_EX | libc::LOCK_NB) };
                            if res2 == 0 {
                                return;
                            }
                            unsafe { libc::close(fd2) };
                        }
                    } else {
                        eprintln!("[{}] Another instance (PID {}) is running. Exiting.", name, pid);
                        std::process::exit(0);
                    }
                }
            }
            eprintln!(
                "[{}] Another instance is already running (errno={}). Exiting.",
                name, err
            );
        } else {
            eprintln!(
                "[{}] Lock error (errno={}). Exiting.",
                name, err
            );
        }
        std::process::exit(0);
    }
}
