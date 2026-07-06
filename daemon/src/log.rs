use std::fs;
use std::path::PathBuf;

fn get_log_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("vietc").join("vietc.log"))
}

fn get_timestamp() -> String {
    if let Ok(n) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let secs = n.as_secs();
        let millis = n.subsec_millis();
        unsafe {
            let t = secs as libc::time_t;
            let mut tm = std::mem::zeroed::<libc::tm>();
            if !libc::localtime_r(&t, &mut tm).is_null() {
                return format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                    tm.tm_year + 1900,
                    tm.tm_mon + 1,
                    tm.tm_mday,
                    tm.tm_hour,
                    tm.tm_min,
                    tm.tm_sec,
                    millis
                );
            }
        }
    }
    "".to_string()
}

pub fn log_info(msg: &str) {
    eprintln!("{}", msg);

    if let Some(log_path) = get_log_path() {
        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(metadata) = fs::metadata(&log_path) {
            if metadata.len() > 10 * 1024 * 1024 {
                let backup_path = log_path.with_extension("log.old");
                let _ = fs::rename(&log_path, backup_path);
            }
        }

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let timestamp = get_timestamp();
            let _ = writeln!(file, "[{}] {}", timestamp, msg);
        }
    }
}
