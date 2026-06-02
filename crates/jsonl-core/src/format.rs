use std::time::Duration;

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    }

    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if value >= 10.0 || unit == 0 {
        format!("{:.0} {}", value, units[unit])
    } else {
        format!("{:.1} {}", value, units[unit])
    }
}

pub fn format_count(n: u64) -> String {
    let mut s = String::new();
    let n_str = n.to_string();
    let len = n_str.len();

    for (i, ch) in n_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            s.push(',');
        }
        s.push(ch);
    }

    s
}

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let nanos = d.subsec_nanos();

    if total_secs == 0 && nanos < 1_000_000 {
        return format!("{} ns", nanos);
    }

    if total_secs == 0 {
        if nanos < 1_000_000_000 {
            let micros = nanos as f64 / 1_000.0;
            if micros >= 10.0 {
                return format!("{:.0} us", micros);
            } else {
                return format!("{:.1} us", micros);
            }
        }
        let ms = nanos as f64 / 1_000_000.0;
        return format!("{:.1} ms", ms);
    }

    if total_secs < 60 {
        if total_secs >= 10 {
            return format!("{} s", total_secs);
        }
        let ms = (total_secs as f64 * 1000.0) + (nanos as f64 / 1_000_000.0);
        return format!("{:.1} s", ms);
    }

    if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        return format!("{}m {}s", mins, secs);
    }

    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    format!("{}h {}m", hours, mins)
}

pub fn format_throughput(bytes: u64, d: Duration) -> String {
    let secs = d.as_secs_f64();

    if secs < 0.001 || bytes == 0 {
        return "-".to_string();
    }

    let bytes_per_sec = bytes as f64 / secs;
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}
