use image::GenericImage;
use std::io::BufWriter;
use serde_json::json;
use serde_json::Value as Json;
mod settings;
pub use settings::Settings;
mod geom;
use geom::*;

trait Renderable {
    fn render(&self, scale: &Vector2, image: &mut image::DynamicImage);
}

#[derive(Debug)]
pub struct Road {
    asphalt: Rectangle,
}

impl Renderable for Road {
    fn render(&self, scale: &Vector2, image: &mut image::DynamicImage) {
        for coord in self.asphalt.scale(*scale).interior_int_coords() {
            let asphalt_color = image::Rgba::from([200, 200, 200, 255]);
            image.put_pixel(coord.x, coord.y, asphalt_color);
        }
    }
}

#[derive(Debug)]
pub struct Building {
    footprint: Rectangle,
    roof_edge_width: f64,
    height: f64,
}

#[derive(Debug)]
pub struct Tree {
    footprint: Circle,
    height: f64,
}

#[derive(Debug)]
pub struct Park {
    footprint: Rectangle,
    trees: Vec<Tree>,
}

#[derive(Debug)]
pub struct City {
    /// The size of the city in meters
    size: Vector2,
    roads: Vec<Road>,
    buildings: Vec<Building>,
    parks: Vec<Park>,
}

impl City {
    fn new(width: f64, height: f64) -> Self {
        Self {
            size: Vector2 {
                x: width,
                y: height,
            },
            roads: Vec::new(),
            buildings: Vec::new(),
            parks: Vec::new(),
        }
    }

    pub fn into_jpeg(self, width: u32, height: u32) -> BufWriter<Vec<u8>> {
        let mut image = image::DynamicImage::new_rgb8(width, height);
        let scale = Vector2 { x: width as f64 / self.size.x, y: height as f64 / self.size.y };

        for road in self.roads {
            road.render(&scale, &mut image);
        }
        
        let jpeg = Vec::with_capacity((width * height * 2) as usize);
        let mut buf_writer = BufWriter::new(jpeg);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut buf_writer);
        encoder.encode_image(&image).expect("Failed to encode image to memory");
        buf_writer
    }
}

// We're yet to include serde
//#[derive(Serialize)]
#[derive(Debug)]
struct Distribution {
    min: f64,
    max: f64,
    alpha: f64,
    dist: String,
}

struct RoadConfig {
    density: [Distribution; 2],
    breadth: Distribution,
}

impl RoadConfig {
    fn from_settings(settings: &Settings) -> Self {
        let mut density_dists = Vec::new();
        for dimension in ["x", "y"] {
            density_dists.push(Distribution {
                alpha: settings.unwrap_float(vec!["roads", "density", dimension]),
                dist: settings.unwrap_string(vec!["roads", "density", dimension, "distribution"]),
                min: settings.unwrap_float(vec!["roads", "density", dimension, "min"]),
                max: settings.unwrap_float(vec!["roads", "density", dimension, "max"]),
            });
        }

        let breadth_dist = Distribution {
            alpha: settings.unwrap_float(["roads", "breadth"]),
            dist: settings.unwrap_string(["roads", "breadth", "distribution"]),
            min: settings.unwrap_float(["roads", "breadth", "min"]),
            max: settings.unwrap_float(["roads", "breadth", "max"]),
        };

        Self {
            density: density_dists.try_into().unwrap(),
            breadth: breadth_dist,
        }
    }
}

#[derive(Debug)]
pub enum GenerateError{
    StatsService,
    MalformedStats,
}

pub struct Builder {
    settings: Settings,
    client: reqwest::Client,
    city: City,
}

impl Builder {
    pub fn new(settings: Settings) -> Self {
        let width = settings.unwrap_float(["city", "width"]);
        let height = settings.unwrap_float(["city", "height"]);
        Self {
            settings,
            client: reqwest::Client::new(),
            city: City::new(width, height),
        }
    }

    pub async fn generate_roads(mut self) -> Result<Self, GenerateError> {
        let road_config = RoadConfig::from_settings(&self.settings);
        

        let max_horizontal = self.city.size.x * road_config.density[0].max + 1.0;
        let max_vertical = self.city.size.y * road_config.density[1].max + 1.0;

        let horizontal_response = self.client
            .post("http://localhost:8000")
            .header("Content-Type", "application/json")
            .json(&json!({
                "distribution": road_config.density[0].dist,
                "params": {
                    "min": road_config.density[0].min,
                    "max": road_config.density[0].max,
                },
                "multiplicity": max_horizontal
            }))
            .send()
            .await
            .map_err(|_| GenerateError::StatsService)?
            .json::<Json>()
            .await
            .map_err(|_| { println!("{}", e); GenerateError::MalformedStats })?;
        
        let horizontal_jitters = horizontal_response
            .get("data").ok_or(GenerateError::MalformedStats)?
            .as_array().ok_or(GenerateError::MalformedStats)?;

        let vertical_response = self.client
            .post("http://localhost:8000")
            .header("Content-Type", "application/json")
            .json(&json!({
                "distribution": road_config.density[1].dist,
                "params": {
                    "min": road_config.density[1].min,
                    "max": road_config.density[1].max,
                },
                "multiplicity": max_vertical
            }))
            .send()
            .await
            .map_err(|_| GenerateError::StatsService)?
            .json::<Json>()
            .await
            .map_err(|_| { GenerateError::MalformedStats })?;

        let vertical_jitters = vertical_response
            .get("data").ok_or(GenerateError::MalformedStats)?
            .as_array().ok_or(GenerateError::MalformedStats)?;

        // Horizontal
        let density = road_config.density[0].alpha;
        let delta = if density == 0.0 { 0.0 } else { density.recip() };
        let mut next_offset = 0.5 * delta;
        let mut i = 0;
        while next_offset < self.city.size.x {
            self.city.roads.push(Road {
                asphalt: Rectangle::new(
                    Vector2 {
                        x: 0.0,
                        y: next_offset - road_config.breadth.alpha / 2.0,
                    },
                    Vector2 {
                        x: self.city.size.x,
                        y: next_offset + road_config.breadth.alpha / 2.0,
                    },
                ),
            });

            next_offset += horizontal_jitters[i].as_f64().ok_or(GenerateError::MalformedStats)?.recip();
            i += 1;
        }

        // Vertical
        i = 0;
        let density = road_config.density[1].alpha;
        let delta = if density == 0.0 { 0.0 } else { density.recip() };
        let mut next_offset = 0.5 * delta;
        while next_offset < self.city.size.y {
            self.city.roads.push(Road {
                asphalt: Rectangle::new(
                    Vector2 {
                        y: 0.0,
                        x: next_offset - road_config.breadth.alpha / 2.0,
                    },
                    Vector2 {
                        y: self.city.size.x,
                        x: next_offset + road_config.breadth.alpha / 2.0,
                    },
                ),
            });

            next_offset += vertical_jitters[i].as_f64().ok_or(GenerateError::MalformedStats)?.recip();
            i += 1;
        }

        Ok(self)
    }

    pub fn build(self) -> City {
        self.city
    }
}
