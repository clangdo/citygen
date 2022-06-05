use super::Settings;

// This module only returns errors from the parent settings API
// This could happen if one of the settings doesn't exist, or is the wrong type.
use super::Error as Error;
use super::super::stats::{Distribution, Distribution2};

pub struct RoadConfig {
    pub density: Distribution2,
    pub breadth: Distribution,
}

impl TryFrom<&Settings> for RoadConfig {
    type Error = Error;

    fn try_from(settings: &Settings) -> Result<Self, Error> {
        Ok(Self {
            density: Distribution2::try_from_settings(
                settings,
                vec!["roads", "density"]
            )?,
            breadth: Distribution::try_from_settings(
                settings,
                vec!["roads", "breadth"]
            )?,
        })
    }
}

pub struct BuildingConfig {
    pub density: Distribution2,
    pub spacing: Distribution,
    pub stepbacks: Distribution,
    pub roof_border: Distribution,
    pub roof_tint: Distribution,
}

impl TryFrom<&Settings> for BuildingConfig {
    type Error = Error;

    fn try_from(settings: &Settings) -> Result<Self, Error> {
        Ok(Self {
            density: Distribution2::try_from_settings(
                settings,
                vec!["buildings", "density"],
            )?,
            spacing: Distribution::try_from_settings(
                settings,
                vec!["alleys", "breadth"],
            )?,
            stepbacks: Distribution::try_from_settings(
                settings,
                vec!["buildings", "walls", "stepback"],
            )?,
            roof_border: Distribution::try_from_settings(
                settings,
                vec!["buildings", "roof", "border"],
            )?,
            roof_tint: Distribution::try_from_settings(
                settings,
                vec!["buildings", "roof", "tint"],
            )?,
        })
    }
}

pub struct CityConfig {
    pub width: f64,
    pub height: f64,
    pub sidewalk_breadth: f64,
}

impl TryFrom<&Settings> for CityConfig {
    type Error = Error;

    fn try_from(settings: &Settings) -> Result<Self, Error> {
        Ok(Self {
            width: settings.get(vec!["city", "width"])?,
            height: settings.get(vec!["city", "height"])?,
            sidewalk_breadth: settings.get(vec!["sidewalk", "breadth"])?,
        })
    }
}

pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
}

impl TryFrom<&Settings> for ImageConfig {
    type Error = Error;

    fn try_from(settings: &Settings) -> Result<Self, Error> {
        Ok(Self {
            width: settings.get(vec!["image", "width"])?,
            height: settings.get(vec!["image", "height"])?,
        })
    }
}

pub struct Config {
    pub image: ImageConfig,
    pub city: CityConfig,
    pub roads: RoadConfig,
    pub buildings: BuildingConfig,
}

impl TryFrom<Settings> for Config {
    type Error = Error;

    fn try_from(settings: Settings) -> Result<Self, Error> {
        Ok(Self {
            image: ImageConfig::try_from(&settings)?,
            city: CityConfig::try_from(&settings)?,
            roads: RoadConfig::try_from(&settings)?,
            buildings: BuildingConfig::try_from(&settings)?,
        })
    }
}

