#[cfg(test)]
mod tests {
    use super::*;

    fn curve() -> Vec<CurvePoint> {
        vec![CurvePoint::new(40, 0), CurvePoint::new(60, 40), CurvePoint::new(80, 80)]
    }

    /// Scenario: Drag point clamped to chart
    #[test]
    fn drag_point_clamped_to_chart() {
        let mut c = curve();
        drag_to(&mut c, 1, 60.0, 140.0);
        assert_eq!(c[1].percent, 100);
        drag_to(&mut c, 2, 140.0, 50.0);
        assert_eq!(c[2].temp, 100);
    }

    /// Scenario: Drag point cannot pass neighbour
    #[test]
    fn drag_point_cannot_pass_neighbour() {
        let mut c = curve();
        drag_to(&mut c, 1, 20.0, 40.0);
        assert_eq!(c[1].temp, 41);
    }

    /// Scenario: Add point
    #[test]
    fn add_point() {
        let mut c = curve();
        assert!(insert_at(&mut c, 70.0, 55.0));
        assert_eq!(c.iter().map(|p| p.temp).collect::<Vec<_>>(), [40, 60, 70, 80]);
        assert_eq!(c[2], CurvePoint::new(70, 55));
    }

    /// Scenario: Add point at limit
    #[test]
    fn add_point_at_limit() {
        let mut c: Vec<_> = (0..8).map(|i| CurvePoint::new(30 + i * 9, i * 12)).collect();
        assert!(!insert_at(&mut c, 99.0, 50.0));
        assert_eq!(c.len(), 8);
    }

    /// Scenario: Remove point
    #[test]
    fn remove_point() {
        let mut c = curve();
        assert!(remove(&mut c, 1));
        assert_eq!(c, [CurvePoint::new(40, 0), CurvePoint::new(80, 80)]);
    }

    /// Scenario: Remove point at minimum
    #[test]
    fn remove_point_at_minimum() {
        let mut c = vec![CurvePoint::new(40, 0), CurvePoint::new(80, 80)];
        assert!(!remove(&mut c, 0));
        assert_eq!(c.len(), 2);
    }
}
