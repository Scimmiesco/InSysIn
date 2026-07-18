use crate::models::hardware::MemBreakdown;

fn parse_vm_stat_val(line: &str, key: &str) -> Option<u64> {
    let line = line.trim();
    line.strip_prefix(key)
        .and_then(|rest| {
            let rest = rest.trim_end_matches('.');
            rest.split_whitespace().next()
        })
        .and_then(|s| s.replace(',', "").parse::<u64>().ok())
}

#[cfg(target_os = "macos")]
pub fn coletar_breakdown() -> Option<MemBreakdown> {
    let output = std::process::Command::new("vm_stat")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&output.stdout);

    let page_size = s
        .lines()
        .next()
        .and_then(|l| l.split('(').nth(1))
        .and_then(|s| s.split_whitespace().nth(3))
        .and_then(|s| s.parse::<u64>().ok())?;

    let mut active = 0u64;
    let mut wire = 0u64;
    let mut compressed = 0u64;
    let mut inactive = 0u64;
    let mut purgeable = 0u64;

    for line in s.lines() {
        if let Some(v) = parse_vm_stat_val(line, "Pages active:") {
            active = v;
        } else if let Some(v) = parse_vm_stat_val(line, "Pages wired down:") {
            wire = v;
        } else if let Some(v) = parse_vm_stat_val(line, "Pages occupied by compressor:") {
            compressed = v;
        } else if let Some(v) = parse_vm_stat_val(line, "Pages inactive:") {
            inactive = v;
        } else if let Some(v) = parse_vm_stat_val(line, "Pages purgeable:") {
            purgeable = v;
        }
    }

    Some(MemBreakdown {
        app_memory: active * page_size,
        wired_memory: wire * page_size,
        compressed_memory: compressed * page_size,
        cached_memory: (inactive + purgeable) * page_size,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn coletar_breakdown() -> Option<MemBreakdown> {
    None
}
