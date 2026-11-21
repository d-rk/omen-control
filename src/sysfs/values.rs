#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Parse driver colour output
    #[test]
    fn parse_driver_colour_output() {
        assert_eq!("#ff8000\n".parse::<Rgb>().unwrap(), Rgb::new(255, 128, 0));
        assert_eq!("ff8000".parse::<Rgb>().unwrap(), Rgb::new(255, 128, 0));
    }

    /// Scenario: Reject malformed colour
    #[test]
    fn reject_malformed_colour() {
        for bad in ["#ff80", "12345G", "#ff80001", ""] {
            let err = bad.parse::<Rgb>().unwrap_err();
            assert_eq!(err.value, bad);
        }
    }

    /// Scenario: Format colour for writing
    #[test]
    fn format_colour_for_writing() {
        assert_eq!(Rgb::new(255, 128, 0).to_string(), "FF8000");
    }

    /// Scenario: Parse curve
    #[test]
    fn parse_curve_text() {
        let c = parse_curve("45:0 60:40 75:70 92:100\n").unwrap();
        assert_eq!(
            c,
            vec![CurvePoint::new(45, 0), CurvePoint::new(60, 40), CurvePoint::new(75, 70), CurvePoint::new(92, 100)]
        );
    }

    /// Scenario: Parse max keyword
    #[test]
    fn parse_max_keyword() {
        assert_eq!(parse_curve("40:0 92:max").unwrap()[1], CurvePoint::new(92, 100));
    }

    /// Scenario: Parse GPU following CPU
    #[test]
    fn parse_gpu_following_cpu() {
        assert_eq!(parse_gpu_curve("(cpu)\n").unwrap(), GpuCurve::FollowsCpu);
    }

    /// Scenario: Parse unset curve
    #[test]
    fn parse_unset_curve() {
        assert_eq!(parse_curve("(unset)\n").unwrap(), vec![]);
    }

    /// Scenario: Format curve sorted
    #[test]
    fn format_curve_sorted() {
        let pts = [CurvePoint::new(75, 70), CurvePoint::new(45, 0), CurvePoint::new(60, 40)];
        assert_eq!(format_curve(&pts).unwrap(), "45:0 60:40 75:70");
    }

    /// Scenario: Reject curve with too few or too many points
    #[test]
    fn reject_curve_with_too_few_or_too_many_points() {
        assert!(validate_curve(&[CurvePoint::new(40, 0)]).is_err());
        let nine: Vec<_> = (0..9).map(|i| CurvePoint::new(30 + i * 5, i * 10)).collect();
        assert!(validate_curve(&nine).is_err());
        assert!(validate_curve(&nine[..8]).is_ok());
    }

    /// Scenario: Reject curve point out of range
    #[test]
    fn reject_curve_point_out_of_range() {
        assert!(validate_curve(&[CurvePoint::new(40, 0), CurvePoint::new(90, 101)]).is_err());
        assert!(validate_curve(&[CurvePoint::new(40, 0), CurvePoint::new(121, 100)]).is_err());
    }

    /// Scenario: Parse gradient config
    #[test]
    fn parse_gradient_config_text() {
        let g = parse_gradient_config("0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500").unwrap();
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].zones, vec![0, 1, 2]);
        assert_eq!(g[0].colors, vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255)]);
        assert_eq!(g[1].zones, vec![3]);
        assert_eq!(g[1].colors, vec![Rgb::new(0x80, 0, 0x80), Rgb::new(0xFF, 0xA5, 0)]);
    }

    /// Scenario: Gradient config round trip
    #[test]
    fn gradient_config_round_trip() {
        let s = "0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500";
        assert_eq!(format_gradient_config(&parse_gradient_config(s).unwrap()), s);
    }

    /// Scenario: Reject gradient zone out of range
    #[test]
    fn reject_gradient_zone_out_of_range() {
        assert!(parse_gradient_config("0,4:FF0000").is_err());
    }

    /// Scenario: Parse power limits
    #[test]
    fn parse_power_limits_text() {
        assert_eq!(parse_power_limits("65 90\n").unwrap(), PowerLimits::Custom { pl1: 65, pl2: 90 });
    }

    /// Scenario: Parse default power limits
    #[test]
    fn parse_default_power_limits() {
        assert_eq!(parse_power_limits("default\n").unwrap(), PowerLimits::Default);
    }

    /// Scenario: Reject PL2 below PL1
    #[test]
    fn reject_pl2_below_pl1() {
        let err = PowerLimits::custom(90, 65).unwrap_err();
        assert!(err.contains("PL2 must be at least PL1"));
    }

    /// Scenario: Parse unavailable RPM
    #[test]
    fn parse_unavailable_rpm() {
        assert_eq!(parse_rpm("n/a\n").unwrap(), None);
    }

    /// Scenario: Parse max RPM
    #[test]
    fn parse_max_rpm_text() {
        assert_eq!(parse_max_rpm("5200 5400\n").unwrap(), Some(MaxRpm { cpu: 5200, gpu: 5400 }));
    }
}
