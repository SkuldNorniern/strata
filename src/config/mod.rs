use std::path::PathBuf;
use std::sync::Arc;

/// Application configuration and constants
pub struct Config {
    pub base_dir: Arc<PathBuf>,
    pub static_dir: Arc<PathBuf>,
    pub port: u16,
    pub host: String,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            base_dir: Arc::new(PathBuf::from("wiki")),
            static_dir: Arc::new(PathBuf::from("static")),
            port: 5004,
            host: "0.0.0.0".to_string(),
        }
    }

    /// Create configuration with custom values
    pub fn with_custom(
        base_dir: PathBuf,
        static_dir: PathBuf,
        port: Option<u16>,
        host: Option<String>,
    ) -> Self {
        Self {
            base_dir: Arc::new(base_dir),
            static_dir: Arc::new(static_dir),
            port: port.unwrap_or(5004),
            host: host.unwrap_or_else(|| "0.0.0.0".to_string()),
        }
    }

    /// Get the socket address for binding
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([0, 0, 0, 0], self.port))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
