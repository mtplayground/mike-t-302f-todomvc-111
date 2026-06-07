use std::{env, error::Error, fmt, net::SocketAddr, path::PathBuf};

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8080";
const DEFAULT_FRONTEND_DIST_DIR: &str = "crates/frontend/dist";
const DEFAULT_DB_MAX_CONNECTIONS: u32 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub database_url: String,
    pub bind_address: SocketAddr,
    pub frontend_dist_dir: PathBuf,
    pub db_max_connections: u32,
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
        let frontend_dist_dir = optional_env("FRONTEND_DIST_DIR")
            .unwrap_or_else(|| DEFAULT_FRONTEND_DIST_DIR.to_owned())
            .into();
        let db_max_connections = optional_env("DB_MAX_CONNECTIONS")
            .map(parse_db_max_connections)
            .transpose()?
            .unwrap_or(DEFAULT_DB_MAX_CONNECTIONS);

        Ok(Self {
            database_url,
            bind_address,
            frontend_dist_dir,
            db_max_connections,
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
    InvalidDbMaxConnections {
        value: String,
        source: std::num::ParseIntError,
    },
    InvalidDbMaxConnectionsValue { value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { name } => write!(formatter, "required environment variable {name} is not set"),
            Self::InvalidBindAddress { value, source } => {
                write!(formatter, "BIND_ADDRESS value {value:?} is invalid: {source}")
            }
            Self::InvalidDbMaxConnections { value, source } => {
                write!(formatter, "DB_MAX_CONNECTIONS value {value:?} is invalid: {source}")
            }
            Self::InvalidDbMaxConnectionsValue { value } => {
                write!(formatter, "DB_MAX_CONNECTIONS value {value:?} must be greater than zero")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingEnv { .. } => None,
            Self::InvalidBindAddress { source, .. } => Some(source),
            Self::InvalidDbMaxConnections { source, .. } => Some(source),
            Self::InvalidDbMaxConnectionsValue { .. } => None,
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

fn parse_db_max_connections(value: String) -> Result<u32, ConfigError> {
    let parsed = value
        .parse()
        .map_err(|source| ConfigError::InvalidDbMaxConnections {
            value: value.clone(),
            source,
        })?;

    if parsed == 0 {
        Err(ConfigError::InvalidDbMaxConnectionsValue { value })
    } else {
        Ok(parsed)
    }
}
