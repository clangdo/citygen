use super::{Vector2, Vector2i};

#[derive(Debug)]
pub struct Rectangle {
    start: Vector2,
    end: Vector2,
}

impl Rectangle {
    pub fn new(start: Vector2, end: Vector2) -> Self {
        Self {
            start: Vector2 {
                x: f64::min(start.x, end.x),
                y: f64::min(start.y, end.y),
            },
            end: Vector2 {
                x: f64::max(start.x, end.x),
                y: f64::max(start.y, end.y),
            },
        }
    }

    pub fn width(&self) -> f64 {
        (self.end.x - self.start.x).abs()
    }

    pub fn height(&self) -> f64 {
        (self.end.y - self.start.y).abs()
    }

    // Referenced:
    // https://stackoverflow.com/questions/27535289/what-is-the-correct-way-to-return-an-iterator-or-any-other-trait
    // on Mon 23 May 2022
    //
    // Using this I discovered the impl Trait syntax, which really
    // really helps here...  Basically you need the compiler to figure
    // out the type here because closures have concrete types, but
    // they are opaque, impl Iterator<Item = (u32, u32) tells the
    // compiler to figure out the concrete type for us!
    pub fn interior_int_coords(&self) -> impl Iterator<Item = Vector2i> {
        let startx = self.start.x as u32;
        let endx = self.end.x as u32;
        let starty = self.start.y as u32;
        let endy = self.end.y as u32;
        
        (startx..endx).flat_map(move |x| (starty..endy).map(move |y| Vector2i{ x, y } ))
    }

    pub fn dimensions(&self) -> Vector2 {
        Vector2{ x: self.width(), y: self.height() }
    }

    pub fn contains(&self, point: Vector2) -> bool {
        point.east_of(self.start) && point.west_of(self.end) &&
            point.north_of(self.end) && point.south_of(self.start)
    }

    pub fn scale(&self, by: Vector2) -> Self {
        Self {
            start: self.start * by,
            end: self.end * by,
        }
    }

    pub fn intersects(&self, other: Rectangle) -> bool {
        todo!()
    }
}
