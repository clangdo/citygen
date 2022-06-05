use image::GenericImageView;
use image::GenericImage;

use super::geom::*;

pub trait Renderable {
    fn render(&self, offset: Vector2, scale: Vector2, image: &mut image::DynamicImage);
}

pub struct Block {
    pub footprint: Rectangle,
    pub buildings: Vec<Building>,
    pub sidewalk_breadth: f64,
}

impl Block {
    pub fn buildings_boundary(&self) -> Rectangle {
        self.footprint.inset(self.sidewalk_breadth)
    }
}

impl Renderable for Block {
    fn render(&self, offset: Vector2, scale: Vector2, image: &mut image::DynamicImage) {
        let region = self
            .footprint
            .translate(offset)
            .scale(scale)
            .interior_int_coords();
        for pixel in region {
            let concrete_color = image::Rgba::from([0xA0, 0xA0, 0xA0, 0xFF]);
            image.put_pixel(pixel.x, pixel.y, concrete_color);
        }

        let buildings_offset = offset + self.buildings_boundary().start();

        for building in self.buildings.iter() {
            building.render(buildings_offset, scale, image);
        }
    }
}

#[derive(Debug)]
pub struct Road {
    pub asphalt: Rectangle,
}

impl Renderable for Road {
    fn render(&self, offset: Vector2, scale: Vector2, image: &mut image::DynamicImage) {
        let region = self
            .asphalt
            .translate(offset)
            .scale(scale)
            .interior_int_coords();
        
        for pixel in region {
            let asphalt_color = image::Rgba::from([0x20, 0x20, 0x20, 0xFF]);
            image.put_pixel(pixel.x, pixel.y, asphalt_color);
        }
    }
}

#[derive(Debug)]
pub struct Building {
    pub footprint: Rectangle,
    pub roof_edge_breadth: f64,
    pub height: f64,
    pub roof_color: [u8; 4],
}

impl Renderable for Building {
    fn render(&self, offset: Vector2, scale: Vector2, image: &mut image::DynamicImage) {
        let roof_with_edge = self
            .footprint
            .translate(offset)
            .scale(scale);

        let roof = self.footprint.translate(offset)
            .inset(self.roof_edge_breadth)
            .scale(scale);

        
        for pixel in roof_with_edge.interior_int_coords() {
            let roof_edge_color = image::Rgba::from([0x50, 0x50, 0x50, 0xFF]);
            if pixel.y < image.dimensions().1 && pixel.x < image.dimensions().0 {
                image.put_pixel(pixel.x, pixel.y, roof_edge_color);
            }
        }

        for pixel in roof.interior_int_coords() {
            let roof_color = image::Rgba::from(self.roof_color);
            if pixel.y < image.dimensions().1 && pixel.x < image.dimensions().0 {
                image.put_pixel(pixel.x, pixel.y, roof_color);
            }
        }
    }
}

// The following code is to be fleshed out in a future version
// As of now park generation is not supported.

/*
#[derive(Debug)]
pub struct Tree {
    pub footprint: Circle,
    pub height: f64,
}

#[derive(Debug)]
pub struct Park {
    pub footprint: Rectangle,
    pub trees: Vec<Tree>,
}
*/
