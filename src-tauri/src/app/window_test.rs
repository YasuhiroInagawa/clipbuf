use super::window::place_beside;

const MON: (f64, f64, f64, f64) = (0.0, 0.0, 1440.0, 900.0);
const SIZE: (f64, f64) = (480.0, 520.0);

#[test]
fn prefers_the_right_side_of_the_main_window() {
    let main = (100.0, 100.0, 480.0, 360.0);
    assert_eq!(place_beside(main, MON, SIZE, 16.0), (596.0, 100.0));
}

#[test]
fn falls_back_to_the_left_when_the_right_does_not_fit() {
    let main = (900.0, 100.0, 480.0, 360.0); // right edge at 1380, no room for 496 more
    assert_eq!(place_beside(main, MON, SIZE, 16.0), (404.0, 100.0));
}

#[test]
fn falls_back_below_when_neither_side_fits() {
    let main = (300.0, 50.0, 900.0, 300.0); // 1200 right edge; left needs 496 but only 300
    assert_eq!(place_beside(main, MON, SIZE, 16.0), (300.0, 366.0));
}

#[test]
fn clamps_to_the_monitor() {
    let main = (100.0, 700.0, 480.0, 360.0); // y + h would run off the bottom
    let (x, y) = place_beside(main, MON, SIZE, 16.0);
    assert_eq!(x, 596.0);
    assert_eq!(y, 900.0 - 520.0);
    let main = (200.0, 800.0, 1100.0, 300.0); // below would overflow → clamped
    let (x, y) = place_beside(main, MON, SIZE, 16.0);
    assert_eq!(x, 200.0);
    assert_eq!(y, 380.0);
}

#[test]
fn honours_monitor_origin_on_secondary_displays() {
    let mon = (-1920.0, 0.0, 1920.0, 1080.0);
    let main = (-1800.0, 200.0, 480.0, 360.0);
    assert_eq!(place_beside(main, mon, SIZE, 16.0), (-1304.0, 200.0));
}
