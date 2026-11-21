//! Environment checks for the driver-not-found screen.

use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureBoot {
    Enabled,
    Disabled,
    /// `mokutil` is missing or printed something unexpected.
    Unknown,
}

/// Parses `mokutil --sb-state` output.
pub fn parse_sb_state(output: &str) -> SecureBoot {
    let o = output.to_ascii_lowercase();
    if o.contains("secureboot enabled") {
        SecureBoot::Enabled
    } else if o.contains("secureboot disabled") {
        SecureBoot::Disabled
    } else {
        SecureBoot::Unknown
    }
}

pub fn secure_boot_state() -> SecureBoot {
    match Command::new("mokutil").arg("--sb-state").output() {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            parse_sb_state(&text)
        }
        Err(_) => SecureBoot::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Parse Secure Boot state
    #[test]
    fn parse_secure_boot_state() {
        assert_eq!(parse_sb_state("SecureBoot enabled\n"), SecureBoot::Enabled);
        assert_eq!(parse_sb_state("SecureBoot disabled\nPlatform is in Setup Mode\n"), SecureBoot::Disabled);
        assert_eq!(parse_sb_state("EFI variables are not supported on this system\n"), SecureBoot::Unknown);
    }
}
