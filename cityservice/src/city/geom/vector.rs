use std::ops::{Add, Mul, MulAssign, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Mul for Vector2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y
        }
    }
}

impl MulAssign for Vector2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl From<Vector2i> for Vector2 {
    fn from(vector: Vector2i) -> Self {
        Self {
            x: vector.x as f64,
            y: vector.y as f64,
        }
    }
}

impl Vector2 {
    pub fn mag(self) -> f64 {
        self.mag_squared().sqrt()
    }

    pub fn mag_squared(self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }

    pub fn north_of(self, point: Vector2) -> bool {
        self.y > point.y
    }

    pub fn south_of(self, point: Vector2) -> bool {
        self.y < point.y
    }

    pub fn east_of(self, point: Vector2) -> bool {
        self.x > point.x
    }

    pub fn west_of(self, point: Vector2) -> bool {
        self.x < point.x
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vector2i {
    pub x: u32,
    pub y: u32
}

impl Add for Vector2i {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2i {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul for Vector2i {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign for Vector2i {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Vector2i {
    pub fn mag(self) -> f64 {
        (self.mag_squared() as f64).sqrt()
    }

    pub fn mag_squared(self) -> u32 {
        self.x.pow(2) + self.y.pow(2)
    }

    pub fn north_of(self, point: Vector2i) -> bool {
        self.y > point.y
    }

    pub fn south_of(self, point: Vector2i) -> bool {
        self.y < point.y
    }

    pub fn east_of(self, point: Vector2i) -> bool {
        self.x > point.x
    }

    pub fn west_of(self, point: Vector2i) -> bool {
        self.x < point.x
    }
}
