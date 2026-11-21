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
