//! Fan curve editing and the chart's pixel mapping.

use iced::{Point, Rectangle};

use crate::sysfs::{CURVE_MAX_POINTS, CURVE_MIN_POINTS, CurvePoint};

pub const CHART_MIN_TEMP: f32 = 30.0;
pub const CHART_MAX_TEMP: f32 = 100.0;
/// Pixel radius within which a click hits a point.
pub const HIT_RADIUS: f32 = 11.0;

/// Moves point `idx` to `temp`/`percent`, keeping it inside the chart and strictly between its neighbours.
pub fn drag_to(points: &mut [CurvePoint], idx: usize, temp: f32, percent: f32) {
    let Some(&current) = points.get(idx) else {
        return;
    };
    let lo = idx.checked_sub(1).map_or(CHART_MIN_TEMP, |i| f32::from(points[i].temp) + 1.0).max(CHART_MIN_TEMP);
    let hi = points.get(idx + 1).map_or(CHART_MAX_TEMP, |p| f32::from(p.temp) - 1.0).min(CHART_MAX_TEMP);
    let temp = if lo <= hi { temp.round().clamp(lo, hi) as u8 } else { current.temp };
    points[idx] = CurvePoint::new(temp, percent.round().clamp(0.0, 100.0) as u8);
}

/// Adds a point in temperature order. Returns `false` when the curve is full or the temperature is taken.
pub fn insert_at(points: &mut Vec<CurvePoint>, temp: f32, percent: f32) -> bool {
    if points.len() >= CURVE_MAX_POINTS {
        return false;
    }
    let p = CurvePoint::new(
        temp.round().clamp(CHART_MIN_TEMP, CHART_MAX_TEMP) as u8,
        percent.round().clamp(0.0, 100.0) as u8,
    );
    match points.binary_search_by_key(&p.temp, |q| q.temp) {
        Ok(_) => false,
        Err(at) => {
            points.insert(at, p);
            true
        }
    }
}

/// Removes point `idx`. Returns `false` when the curve is at its minimum size.
pub fn remove(points: &mut Vec<CurvePoint>, idx: usize) -> bool {
    if points.len() <= CURVE_MIN_POINTS || idx >= points.len() {
        return false;
    }
    points.remove(idx);
    true
}

/// The plot area inside the canvas bounds (room for axis labels on the left and bottom).
pub fn plot_area(size: iced::Size) -> Rectangle {
    let (left, right, top, bottom) = (44.0, 14.0, 14.0, 30.0);
    Rectangle {
        x: left,
        y: top,
        width: (size.width - left - right).max(1.0),
        height: (size.height - top - bottom).max(1.0),
    }
}

pub fn to_screen(plot: Rectangle, temp: f32, percent: f32) -> Point {
    let t = temp.clamp(CHART_MIN_TEMP, CHART_MAX_TEMP);
    Point::new(
        plot.x + (t - CHART_MIN_TEMP) / (CHART_MAX_TEMP - CHART_MIN_TEMP) * plot.width,
        plot.y + (1.0 - percent.clamp(0.0, 100.0) / 100.0) * plot.height,
    )
}

/// Temperature and percent at a canvas-local position (not clamped).
pub fn from_screen(plot: Rectangle, p: Point) -> (f32, f32) {
    (
        CHART_MIN_TEMP + (p.x - plot.x) / plot.width * (CHART_MAX_TEMP - CHART_MIN_TEMP),
        (1.0 - (p.y - plot.y) / plot.height) * 100.0,
    )
}

/// Index of the point under `p`, if any.
pub fn point_at(points: &[CurvePoint], plot: Rectangle, p: Point) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .map(|(i, q)| (i, to_screen(plot, f32::from(q.temp), f32::from(q.percent)).distance(p)))
        .filter(|(_, d)| *d <= HIT_RADIUS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

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
