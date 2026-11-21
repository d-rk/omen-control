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
