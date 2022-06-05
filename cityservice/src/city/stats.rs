use super::Settings;
use super::settings::Error as Error;

use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
pub struct Distribution {
    pub min: f64,
    pub max: f64,
    pub skew: f64,
    pub dist: String,
}

impl Distribution {
    pub fn try_from_settings(
        settings: &Settings,
        base_path: Vec<&str>,
    ) -> Result<Self, Error> {
        Ok(Distribution {
            skew: settings.get_endpoint(base_path.clone(), "skew")?,
            dist: settings.get_endpoint(base_path.clone(), "distribution")?,
            min: settings.get_endpoint(base_path.clone(), "min")?,
            max: settings.get_endpoint(base_path.clone(), "max")?,
        })
    }
}

pub struct Distribution2 {
    pub x: Distribution,
    pub y: Distribution,
}

impl TryFrom<Vec<Distribution>> for Distribution2 {
    type Error = ();

    fn try_from(mut vec: Vec<Distribution>) -> Result<Self, ()> {
        if vec.len() < 2 {
            return Err(());
        }
        
        Ok(Self {
            y: vec.pop().unwrap(),
            x: vec.pop().unwrap(),
        })
    }

}

impl Distribution2 {
    pub fn try_from_settings(
        settings: &Settings,
        base_path: Vec<&str>
    ) -> Result<Self, Error> {
        let mut dists = Vec::new();
        for dimension in ["x", "y"] {
            let mut dim_path = base_path.clone();
            dim_path.push(&dimension);
            let dim_dist = Distribution::try_from_settings(settings, dim_path)?;
            dists.push(dim_dist);
        }

        Ok(Distribution2::try_from(dists).unwrap())
    }
}

#[derive(Debug)]
pub enum RequestError {
    Network(reqwest::Error),
    MalformedResponse,
}

#[derive(Deserialize)]
struct ResponsePayload {
    data: Vec<f64>,
}

/// This function does the work of making a request to the stats
/// microservice, then parsing the data as json and returning the
/// vector of values in which we're interested.
pub async fn request(
    client: &reqwest::Client,
    multiplicity: u32,
    dist: &Distribution
) -> Result<Vec<f64>, RequestError> {
    client
        .post("http://localhost:8000")
        .header("User-Agent", "cityservice")
        .header("Content-Type", "application/json")
        .json(&json!({
            "distribution": dist.dist,
            "params": {
                "alpha": dist.skew,
                "min": dist.min,
                "max": dist.max,
            },
            "multiplicity": multiplicity
        }))
        .send()
        .await
        .map_err(|err| RequestError::Network(err))?
        .json::<ResponsePayload>()
        .await
        .map_err(|_| RequestError::MalformedResponse)
        .map(|response| response.data)
}
