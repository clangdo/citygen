use std::io::BufWriter;

mod settings;
use settings::config::Config;
pub use settings::Settings;
pub use settings::Error as SettingsError;

mod geom;
use geom::*;

mod objects;
use objects::{Block, Building, Road, Renderable};

mod stats;
use stats::{Distribution};

#[derive(Debug)]
pub enum GenerateError {
    StatsRequest(stats::RequestError),
    UnknownDistribution(String),
    DistributionInverted {
        min: f64,
        max: f64,
    },
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnknownDistribution(name) => format!(
                "The \"{}\" distribution is unknown, are \
                 you sure you typed it correctly? We support \
                 \"normal\", \"uniform\", and \"constant\" distributions.",
                name
            ),
            Self::DistributionInverted{min, max} => format!(
                "There was a distribution with a min ({}) greater than the max ({}). \
                 This is only allowed for constant distributions.",
                min,
                max,
            ),
            // Don't leak information about internal infrastructure to users
            _ => format!("Unknown"),
        };
        
        write!(f, "{}", message)
    }
}

pub struct Builder {
    config: Config,
    client: reqwest::Client,
    city: City,
}

pub struct City {
    /// The size of the city in meters
    size: Vector2,
    image_size: (u32, u32),
    roads: Vec<Road>,
    blocks: Vec<Block>,
}

impl City {
    /// Generates a new featureless city without roads, parks or
    /// buildings.
    fn new() -> Self {
        Self {
            size: Vector2::default(),
            image_size: (2048, 2048),
            roads: Vec::new(),
            blocks: Vec::new(),
        }
    }

    /// Renders the city as a jpeg albedo map and returns a buffered
    /// writer over the encoded image
    pub fn into_jpeg(self) -> BufWriter<Vec<u8>> {
        let (width, height) = self.image_size;
        let mut image = image::DynamicImage::new_rgb8(width, height);
        let scale = Vector2 {
            x: (width as f64 / self.size.x),
            y: (height as f64 / self.size.y)
        };

        let offset = Vector2 {
            x: 0.0,
            y: 0.0,
        };
        
        for road in self.roads {
            road.render(offset, scale, &mut image);
        }

        for block in self.blocks {
            block.render(offset, scale, &mut image);
        }
        
        let jpeg = Vec::with_capacity((width * height) as usize);
        let mut buf_writer = BufWriter::new(jpeg);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut buf_writer);
        encoder.encode_image(&image).expect("Failed to encode image to memory");
        buf_writer
    }
}

/// This structure represents a grid-like set of rectangles. Some of
/// the rectangles are the "lines" of the grid, and some are the
/// "squares". This is redundant data, but it makes things easier to
/// work with, and rectangles are fairly small.
struct GridPartition {
    lines: Vec<Rectangle>,
    rectangles: Vec<Rectangle>,
}

impl GridPartition {
    /// Note grid size is the actual size of the grid in logical units
    /// (a.k.a. meters), The grid may have fewer or more actual lines
    /// than grid size (in either dimension). The grid is not
    /// necessarily square, and it's division lines are not a uniform
    /// width (necessarily).
    fn new<Iter1, Iter2, Iter3>(
        x_offsets: Iter1,
        y_offsets: Iter2,
        mut line_breadths: Iter3,
        grid_bounds: Vector2,
    ) -> Result<GridPartition, GenerateError> where
        Iter1: Iterator<Item = f64>,
        Iter2: Iterator<Item = f64>,
        Iter3: Iterator<Item = f64>,
    {
        let mut grid_lines = Vec::new();
        let mut grid_rectangles = Vec::new();

        let mut square_x_starts = vec![0.0];
        let mut square_y_starts = vec![0.0];
        
        let mut square_x_ends = Vec::new();
        let mut square_y_ends = Vec::new();
        
        // Vertical lines, progressing horizontally
        let mut offset = 0.0;
        for (breadth, next_delta_offset) in line_breadths.by_ref().zip(x_offsets) {
            let outset = breadth / 2.0;
            if !(0.0 + outset..grid_bounds.x - outset).contains(&offset) {
                offset += next_delta_offset;
                continue;
            }

            let line_x_start = offset - outset;
            square_x_ends.push(line_x_start);
            let start = Vector2 {
                x: line_x_start,
                y: 0.0,
            };

            let line_x_end = offset + outset;
            square_x_starts.push(line_x_end);
            let end = Vector2 {
                x: line_x_end,
                y: grid_bounds.y,
            };
            
            grid_lines.push(Rectangle::new(start, end));
            offset += next_delta_offset;
        }

        // Add the last rectangle
        square_x_ends.push(grid_bounds.x);


        // Horizontal lines, progressing vertically
        offset = 0.0;
        for (breadth, next_delta_offset) in line_breadths.zip(y_offsets) {
            let outset = breadth / 2.0;
            if !(0.0 + outset..grid_bounds.y - outset).contains(&offset) {
                offset += next_delta_offset;
                continue;
            }

            let line_y_start = offset - outset;
            square_y_ends.push(line_y_start);
            let start = Vector2 {
                x: 0.0,
                y: line_y_start,
            };

            let line_y_end = offset + outset;
            square_y_starts.push(line_y_end);
            let end = Vector2 {
                x: grid_bounds.x,
                y: line_y_end,
            };
            
            grid_lines.push(Rectangle::new(start, end));

            offset += next_delta_offset;
        }

        // The last rectangle
        square_y_ends.push(grid_bounds.y);

        // These are the x start and end pairs for the grid rectangles
        let x_bounds = square_x_starts
            .into_iter()
            .zip(square_x_ends.into_iter());

        // These are the y start and end pairs for the grid rectangles
        let y_bounds: Vec<(f64, f64)> = square_y_starts
            .into_iter()
            .zip(square_y_ends.into_iter())
            .collect();

        for (x_start, x_end) in x_bounds {
            for (y_start, y_end) in y_bounds.iter().copied() {
                grid_rectangles.push(Rectangle::new(
                    Vector2 {
                        x: x_start,
                        y: y_start,
                    },
                    Vector2 {
                        x: x_end,
                        y: y_end,
                    },
                ));
            }
        }

        Ok(Self {
            lines: grid_lines,
            rectangles: grid_rectangles,
        })
    }
}

impl Builder {
    pub fn new(config: Config) -> Self {
        let mut city = City::new();
        city.size.x = config.city.width;
        city.size.y = config.city.height;
        city.image_size.0 = config.image.width;
        city.image_size.1 = config.image.height;
        
        Self {
            config,
            client: reqwest::Client::new(),
            city,
        }
    }

    /// This function returns the offset
    async fn get_sample(
        client: &reqwest::Client,
        multiplicity: u32,
        dist: &Distribution
    ) -> Result<Vec<f64>, GenerateError> {
        // We handle constant distributions ourselves
        if dist.dist == "constant" {
            return Ok(std::iter::repeat(dist.max).take(multiplicity as usize).collect())
        }

        if dist.dist != "normal" && dist.dist != "uniform" {
            return Err(GenerateError::UnknownDistribution(dist.dist.clone()))
        }

        if dist.max < dist.min {
            return Err(GenerateError::DistributionInverted{ min: dist.min, max: dist.max })
        }

        // Our partner's microservice helps us sample uniform and
        // normal distributions.
        stats::request(client, multiplicity, dist)
            .await
            .map_err(|err| GenerateError::StatsRequest(err))
    }

    pub fn build_buildings_around<I1, I2, I3>(
        size: Vector2,
        mut x_offsets: I1,
        mut y_offsets: I1,
        mut spacings: I2,
        mut stepbacks: I3,
    ) -> Result<Vec<Rectangle>, GenerateError> where
        I1: Iterator<Item = f64>,
        I2: Iterator<Item = f64>,
        I3: Iterator<Item = f64>,
    {
        // Clockwise corners
        let corners = [
            Vector2{ x: 0.0, y: 0.0 },
            Vector2{ x: size.x, y: 0.0 },
            Vector2{ x: size.x, y: size.y },
            Vector2{ x: 0.0, y: size.y },
        ];

        // Clockwise edges
        let edge_clockwise = [
            Vector2 {x: 1.0, y: 0.0},
            Vector2 {x: 0.0, y: 1.0},
            Vector2 {x: -1.0, y: 0.0},
            Vector2 {x: 0.0, y: -1.0},
        ];
        
        let mut buildings = Vec::new();
        // Move around the edges and build buildings 
        for i in 0..4 {
            let (offsets, length, depth_limit) = if i % 2 == 0 {
                (x_offsets.by_ref(), size.x, size.y)
            } else {
                (y_offsets.by_ref(), size.y, size.x)
            };

            let mut face_buildings = Self::line_blockface(
                length,
                corners[i],
                edge_clockwise[i],
                edge_clockwise[(i + 1) % 4],
                offsets,
                spacings.by_ref(),
                stepbacks.by_ref(),
                depth_limit,
            );

            buildings.append(&mut face_buildings);
        }
        
        Ok(buildings)
    }

    fn  line_blockface<I1, I2, I3>(
        length: f64,
        start: Vector2,
        clockwise: Vector2,
        inwards: Vector2,
        offsets: I1,
        spacings: I2,
        depths: I3,
        depth_limit: f64,
    ) -> Vec<Rectangle> where
        I1: Iterator<Item = f64>,
        I2: Iterator<Item = f64>,
        I3: Iterator<Item = f64>,
    {
        let mut buildings = Vec::new();

        let mut offset = 0.0;

        let mut starts = vec![0.0];
        let mut ends = Vec::new();
        for (delta_offset, spacing) in offsets.zip(spacings) {
            let half_breadth = spacing / 2.0;
            if offset >= length - half_breadth {
                break;
            }

            let alley_start = offset - half_breadth;
            ends.push(alley_start);
            let alley_end = offset + half_breadth;
            starts.push(alley_end);

            offset += delta_offset;
        }

        ends.push(length);

        for ((bound_start, bound_end), depth) in starts.into_iter().zip(ends).zip(depths) {

            let building = Rectangle::new(
                start + clockwise * bound_start,
                start + clockwise * bound_end + inwards * f64::min(depth, depth_limit)
            );

            buildings.push(building);
        }

        buildings
    }

    pub async fn generate_block_buildings(
        client: &reqwest::Client,
        building_config: &settings::config::BuildingConfig,
        block: &mut Block
    ) -> Result<(), GenerateError> {
        let size = block.buildings_boundary().dimensions();

        let max_buildings = Vector2i {
            x: 2 * (size.x * building_config.density.x.max + 1.0) as u32,
            y: 2 * (size.y * building_config.density.y.max + 1.0) as u32,
        };

        let max_num_buildings = max_buildings.x + max_buildings.y - 4;

        let x_densities = Self::get_sample(
            client,
            max_buildings.x,
            &building_config.density.x,
        );

        let y_densities = Self::get_sample(
            client,
            max_buildings.y,
            &building_config.density.y,
        );

        let roof_tints = Self::get_sample(
            client,
            max_num_buildings,
            &building_config.roof_tint,
        );

        let roof_edge_breadths = Self::get_sample(
            client,
            max_num_buildings,
            &building_config.roof_border,
        );

        let spacings = Self::get_sample(
            client,
            max_num_buildings,
            &building_config.spacing,
        );

        let stepbacks = Self::get_sample(
            client,
            max_num_buildings,
            &building_config.stepbacks,
        );
        
        let x_offsets: Vec<f64> = x_densities
            .await?
            .iter()
            .map(|density| density.recip())
            .collect();
        
        let y_offsets: Vec<f64> = y_densities
            .await?
            .iter()
            .map(|density| density.recip())
            .collect();

        let building_rectangles = 
            Self::build_buildings_around(
                size,
                x_offsets.into_iter(),
                y_offsets.into_iter(),
                spacings.await?.into_iter(),
                stepbacks.await?.into_iter(),
            )?;

        block.buildings.extend(
            &mut building_rectangles
                .into_iter()
                .zip(roof_edge_breadths.await?.iter())
                .zip(roof_tints.await?.iter())
                .map(|((building_rectangle, inset), tint)| {
                    let red_tint = (0x30 as f64 * tint) as u8;
                    let green_tint = (0x20 as f64 * tint) as u8;
                    let roof_color = [
                        0xB0 + red_tint,
                        0xB0 + green_tint,
                        0xB0,
                        0xFF
                    ];

                    Building {
                        footprint: building_rectangle,
                        roof_edge_breadth: *inset,
                        height: 0.0,
                        roof_color,
                    }
                })
    );

        Ok(())
    }

    pub async fn build_buildings(mut self) -> Result<Self, GenerateError> {
        let building_config = &self.config.buildings;
        for block in self.city.blocks.iter_mut() {
            Self::generate_block_buildings(
                &self.client,
                building_config,
                block
            ).await?;
        }
        
        Ok(self)
    }

    /// This function generates the roads and blocks for the city, but it
    /// does not initialize any buildings (or parks) on the blocks, they
    /// are assumed to be flat, empty, concrete
    pub async fn build_roads(mut self) -> Result<Self, GenerateError> {
        let road_config = &self.config.roads;
        let max_roads = Vector2i {
            x: (self.city.size.x * road_config.density.x.max + 1.0) as u32,
            y: (self.city.size.y * road_config.density.y.max + 1.0) as u32,
        };

        let max_breadth = max_roads.x + max_roads.y;

        let x_densities = Self::get_sample(
            &self.client,
            max_roads.x,
            &road_config.density.x,
        );

        let y_densities = Self::get_sample(
            &self.client,
            max_roads.y,
            &road_config.density.y
        );
        
        let breadths = Self::get_sample(
            &self.client,
            max_breadth,
            &road_config.breadth
        );

        let size = self.city.size;

        let x_offsets = x_densities.await?
            .into_iter()
            .map(|density| density.recip());

        let y_offsets = y_densities.await?
            .into_iter()
            .map(|density| density.recip());
            
        let GridPartition {
            lines: road_rectangles,
            rectangles: block_rectangles
        } = GridPartition::new(
            x_offsets,
            y_offsets,
            breadths.await?.into_iter(),
            size,
        )?;

        self.city.roads = road_rectangles
            .into_iter()
            .map(|road_rectangle| Road { asphalt: road_rectangle })
            .collect();

        self.city.blocks = block_rectangles
            .into_iter()
            .map(|block_rectangle| Block {
                footprint: block_rectangle,
                buildings: Vec::new(),
                sidewalk_breadth: self.config.city.sidewalk_breadth,
            })
            .collect();

        Ok(self)
    }

    pub fn build(self) -> City {
        self.city
    }
}
