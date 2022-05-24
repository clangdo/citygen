use super::Vector2;

#[derive(Debug)]
pub struct Circle {
    center: Vector2,
    radius: f64,
}

impl Circle {
    fn contains(&self, point: Vector2) -> bool {
        (point - self.center).mag() < self.radius
    }
}
