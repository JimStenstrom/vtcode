use std::time::Duration;
use vtcode_config::constants::http_client;

/// Configuration for HTTP client optimization
#[derive(Clone)]
pub struct ClientConfig {
    /// Maximum number of idle connections per host
    pub pool_max_idle_per_host: usize,
    /// How long to keep idle connections alive
    pub pool_idle_timeout: Duration,
    /// TCP keepalive duration
    pub tcp_keepalive: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// User agent string
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: http_client::DEFAULT_POOL_MAX_IDLE_PER_HOST,
            pool_idle_timeout: http_client::default_pool_idle_timeout(),
            tcp_keepalive: http_client::default_tcp_keepalive(),
            request_timeout: http_client::default_request_timeout(),
            connect_timeout: http_client::default_connect_timeout(),
            user_agent: "vtcode/1.0.0".to_string(),
        }
    }
}

impl ClientConfig {
    /// Configuration optimized for high-throughput scenarios
    pub fn high_throughput() -> Self {
        Self {
            pool_max_idle_per_host: http_client::HIGH_THROUGHPUT_POOL_MAX_IDLE,
            pool_idle_timeout: Duration::from_secs(http_client::HIGH_THROUGHPUT_POOL_IDLE_TIMEOUT_SECS),
            tcp_keepalive: http_client::default_tcp_keepalive(),
            request_timeout: Duration::from_secs(http_client::HIGH_THROUGHPUT_REQUEST_TIMEOUT_SECS),
            connect_timeout: Duration::from_secs(http_client::HIGH_THROUGHPUT_CONNECT_TIMEOUT_SECS),
            user_agent: "vtcode/1.0.0-high-throughput".to_string(),
        }
    }

    /// Configuration optimized for low memory usage (< 100MB target)
    pub fn low_memory() -> Self {
        Self {
            pool_max_idle_per_host: http_client::LOW_MEMORY_POOL_MAX_IDLE,
            pool_idle_timeout: Duration::from_secs(http_client::LOW_MEMORY_POOL_IDLE_TIMEOUT_SECS),
            tcp_keepalive: Duration::from_secs(http_client::LOW_MEMORY_POOL_IDLE_TIMEOUT_SECS),
            request_timeout: http_client::default_request_timeout(),
            connect_timeout: Duration::from_secs(http_client::LOW_MEMORY_CONNECT_TIMEOUT_SECS),
            user_agent: "vtcode/1.0.0-low-memory".to_string(),
        }
    }

    /// Configuration optimized for ultra-low memory (< 50MB target)
    pub fn ultra_low_memory() -> Self {
        Self {
            pool_max_idle_per_host: http_client::ULTRA_LOW_MEMORY_POOL_MAX_IDLE,
            pool_idle_timeout: Duration::from_secs(http_client::ULTRA_LOW_MEMORY_POOL_IDLE_TIMEOUT_SECS),
            tcp_keepalive: Duration::from_secs(http_client::ULTRA_LOW_MEMORY_TCP_KEEPALIVE_SECS),
            request_timeout: http_client::default_request_timeout(),
            connect_timeout: Duration::from_secs(http_client::ULTRA_LOW_MEMORY_CONNECT_TIMEOUT_SECS),
            user_agent: "vtcode/1.0.0-ultra-low-memory".to_string(),
        }
    }

    /// Configuration optimized for low-latency scenarios
    pub fn low_latency() -> Self {
        Self {
            pool_max_idle_per_host: http_client::LOW_LATENCY_POOL_MAX_IDLE,
            pool_idle_timeout: Duration::from_secs(http_client::LOW_MEMORY_POOL_IDLE_TIMEOUT_SECS),
            tcp_keepalive: Duration::from_secs(http_client::LOW_MEMORY_POOL_IDLE_TIMEOUT_SECS),
            request_timeout: http_client::default_request_timeout(),
            connect_timeout: Duration::from_secs(http_client::LOW_MEMORY_CONNECT_TIMEOUT_SECS),
            user_agent: "vtcode/1.0.0-low-latency".to_string(),
        }
    }
}
