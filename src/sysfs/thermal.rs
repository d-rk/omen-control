//! Temperatures from `/sys/class/thermal/thermal_zone*/{type,temp}`.

use std::fs;
use std::path::Path;

/// Zone types tried, in order, when the driver's `fan_temp_zone` reads `(auto)`.
pub const AUTO_CPU_ZONES: [&str; 4] = ["x86_pkg_temp", "coretemp", "k10temp", "acpitz"];

/// Temperature in °C of the first thermal zone whose `type` is `zone_type`.
pub fn zone_temp(thermal_root: &Path, zone_type: &str) -> Option<f32> {
    let entries = fs::read_dir(thermal_root).ok()?;
    let mut dirs: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("thermal_zone")))
        .collect();
    dirs.sort();
    dirs.into_iter().find_map(|d| {
        let t = fs::read_to_string(d.join("type")).ok()?;
        if t.trim() != zone_type {
            return None;
        }
        let milli: i64 = fs::read_to_string(d.join("temp")).ok()?.trim().parse().ok()?;
        Some(milli as f32 / 1000.0)
    })
}

/// CPU temperature for a `fan_temp_zone` value (`(auto)`, empty or a zone type).
pub fn cpu_temp(thermal_root: &Path, setting: Option<&str>) -> Option<f32> {
    match setting.map(str::trim) {
        Some(z) if !z.is_empty() && z != "(auto)" => zone_temp(thermal_root, z),
        _ => AUTO_CPU_ZONES.iter().find_map(|z| zone_temp(thermal_root, z)),
    }
}

/// GPU temperature for a `gpu_fan_temp_zone` value; `(cpu)` or empty means the CPU zone.
pub fn gpu_temp(thermal_root: &Path, setting: Option<&str>, cpu_setting: Option<&str>) -> Option<f32> {
    match setting.map(str::trim) {
        Some(z) if !z.is_empty() && z != "(cpu)" => zone_temp(thermal_root, z),
        _ => cpu_temp(thermal_root, cpu_setting),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Read zone temperature
    #[test]
    fn read_zone_temperature() {
        let dir = tempfile::tempdir().unwrap();
        for (i, (ty, temp)) in [("acpitz", "40000"), ("x86_pkg_temp", "54000")].iter().enumerate() {
            let z = dir.path().join(format!("thermal_zone{i}"));
            fs::create_dir(&z).unwrap();
            fs::write(z.join("type"), format!("{ty}\n")).unwrap();
            fs::write(z.join("temp"), format!("{temp}\n")).unwrap();
        }
        assert_eq!(zone_temp(dir.path(), "x86_pkg_temp"), Some(54.0));
        assert_eq!(cpu_temp(dir.path(), Some("(auto)")), Some(54.0));
    }
}
