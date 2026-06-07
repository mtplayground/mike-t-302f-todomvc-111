use std::{env, error::Error, fmt, net::SocketAddr};

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8080";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub database_url: String,
    pub bind_address: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = required_env("DATABASE_URL")?;
        let bind_address_value =
            optional_env("BIND_ADDRESS").unwrap_or_else(|| DEFAULT_BIND_ADDRESS.to_owned());
        let bind_address = bind_address_value
            .parse()
            .map_err(|source| ConfigError::InvalidBindAddress {
                value: bind_address_value,
                source,
            })?;

        Ok(Self {
            database_url,
            bind_address,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnv { name: &'static str },
    InvalidBindAddress {
        value: String,
        source: std::net::AddrParseError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { name } => write!(formatter, "required environment variable {name} is not set"),
            Self::InvalidBindAddress { value, source } => {
                write!(formatter, "BIND_ADDRESS value {value:?} is invalid: {source}")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingEnv { .. } => None,
            Self::InvalidBindAddress { source, .. } => Some(source),
        }
    }
}

fn required_env(name: &'static str) -> Result<String, ConfigError> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or(ConfigError::MissingEnv { name })
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
