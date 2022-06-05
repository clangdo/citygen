use serde::Serialize;

use super::city;

#[derive(Debug)]
pub enum InputError {
    BadSettingsStructure(city::SettingsError),
    BadDistribution(city::GenerateError),
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::BadSettingsStructure(err) => format!(
                "There was an error with the settings, this is \
                 likely a problem with your script, specifically: {}",
                err,
            ),
            Self::BadDistribution(err) => format!(
                "There was a random number sampling issue: {}",
                err,
            ),
        };
        
        write!(f, "{}", message)
    }
}

#[derive(Debug)]
pub enum Error {
    Overloaded,
    Input(InputError),
    Server,
    Submission,
}

impl From<city::SettingsError> for Error {
    fn from(error: city::SettingsError) -> Self {
        Self::Input(InputError::BadSettingsStructure(error))
    }
}

impl From<city::GenerateError> for Error {
    fn from(error: city::GenerateError) -> Self {
        match error {
            city::GenerateError::UnknownDistribution(_) |
            city::GenerateError::DistributionInverted{..} =>
                Self::Input(InputError::BadDistribution(error)),
            city::GenerateError::StatsRequest(_) => Self::Server,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Error::Overloaded =>String::from(
                "Server is overloaded, please try again later.",
            ),
            Error::Input(err) => format!("{}", err),
            Error::Server => String::from(
                "There is something wrong with our equipment at the moment, \
                 we recommend you stand by and try again in a few hours",
            ),
            Error::Submission => String::from(
                "If you're using a website to do this, then this \
                 is a problem for the developers, please contact them \
                 to get it fixed. \
                 \
                 Otherwise, there was a problem with the JSON you submitted, \
                 please submit a POST request to the /generate endpoint \
                 with the JSON body {cityscript: <script>}",
            ),
        };
        
        write!(f, "{}", message)
    }
}

impl warp::Reply for &Error {
    fn into_response(self) -> warp::http::Response<warp::hyper::Body> {
        use warp::http::status::StatusCode;
        
        let message = format!("{}", self);
        let status = match self {
            Error::Overloaded |
            Error::Server => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Input(_) |
            Error::Submission => StatusCode::BAD_REQUEST,
        };

        let json: String = ErrorJson::new(message).into();
        warp::reply::with_status(json, status).into_response()
    }
}

impl std::error::Error for Error {}
impl warp::reject::Reject for Error {}

#[derive(Serialize)]
pub struct ErrorJson {
    pub error: String
}

impl ErrorJson {
    fn new<T: Into<String>>(message: T) -> Self {
        Self { error: message.into() }
    }
}

impl From<ErrorJson> for String {
    fn from(error_json: ErrorJson) -> Self {
        serde_json::to_string(&error_json).unwrap()
    }
}
