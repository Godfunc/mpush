use std::fs;
use std::io::Write;

const LOG_DIR: &str = "/var/log/mpush";
const RETAIN_DAYS: i64 = 15;

pub fn mask(s: &str) -> String {
    if s.len() < 8 {
        return "***".to_string();
    }
    let mut result = String::with_capacity(11);
    result.push_str(&s[..4]);
    result.push_str("***");
    result.push_str(&s[s.len() - 4..]);
    result
}

pub fn now_str() -> String {
    let (y, mo, d, h, mi, s) = local_time_parts();
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

pub fn today_str() -> String {
    let (y, mo, d, _, _, _) = local_time_parts();
    format!("{y:04}-{mo:02}-{d:02}")
}

pub fn write_log(line: &str) {
    let path = format!("{LOG_DIR}/mpush-{}.log", today_str());
    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    let _ = writeln!(file, "{}", line);
}

pub fn cleanup_logs() {
    let Ok(entries) = fs::read_dir(LOG_DIR) else {
        return;
    };
    let cutoff = cutoff_date();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        // extract date from "mpush-YYYY-MM-DD.log"
        if name.len() == 24 && name.starts_with("mpush-") && name.ends_with(".log") {
            let date_part = &name[6..16];
            if date_part < cutoff.as_str() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

unsafe fn localtime(time: &libc::time_t, tm: &mut libc::tm) {
    unsafe {
        #[cfg(unix)]
        libc::localtime_r(time, tm);
        #[cfg(windows)]
        libc::localtime_s(tm, time);
    }
}

fn cutoff_date() -> String {
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let cutoff = now - RETAIN_DAYS * 86400;
        let mut tm: libc::tm = std::mem::zeroed();
        localtime(&cutoff, &mut tm);
        format!(
            "{:04}-{:02}-{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday
        )
    }
}

fn local_time_parts() -> (i32, i32, i32, i32, i32, i32) {
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        localtime(&now, &mut tm);
        (
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_normal() {
        // "oYhhP3NXQYimEqg-3_qsGJcvHJ-w" -> "oYhh***HJ-w"
        assert_eq!(mask("oYhhP3NXQYimEqg-3_qsGJcvHJ-w"), "oYhh***HJ-w");
    }

    #[test]
    fn test_mask_exactly_8_chars() {
        assert_eq!(mask("abcdefgh"), "abcd***efgh");
    }

    #[test]
    fn test_mask_short_string() {
        // fewer than 8 chars -> fully masked
        assert_eq!(mask("short"), "***");
    }

    #[test]
    fn test_mask_empty() {
        assert_eq!(mask(""), "***");
    }

    #[test]
    fn test_today_str_format() {
        let today = today_str();
        // YYYY-MM-DD format
        assert_eq!(today.len(), 10);
        assert_eq!(&today[4..5], "-");
        assert_eq!(&today[7..8], "-");
    }

    #[test]
    fn test_now_str_format() {
        let now = now_str();
        // YYYY-MM-DD HH:MM:SS format
        assert_eq!(now.len(), 19);
        assert_eq!(&now[4..5], "-");
        assert_eq!(&now[7..8], "-");
        assert_eq!(&now[10..11], " ");
        assert_eq!(&now[13..14], ":");
        assert_eq!(&now[16..17], ":");
    }
}
