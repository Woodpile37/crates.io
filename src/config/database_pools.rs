//! Configuration for setting up database pools
//!
//! - `DATABASE_URL`: The URL of the postgres database to use.
//! - `READ_ONLY_REPLICA_URL`: The URL of an optional postgres read-only replica database.
//! - `DB_PRIMARY_POOL_SIZE`: The number of connections of the primary database.
//! - `DB_REPLICA_POOL_SIZE`: The number of connections of the read-only / replica database.
//! - `DB_PRIMARY_MIN_IDLE`: The primary pool will maintain at least this number of connections.
//! - `DB_REPLICA_MIN_IDLE`: The replica pool will maintain at least this number of connections.
//! - `DB_OFFLINE`: If set to `leader` then use the read-only follower as if it was the leader.
//!   If set to `follower` then act as if `READ_ONLY_REPLICA_URL` was unset.
//! - `READ_ONLY_MODE`: If defined (even as empty) then force all connections to be read-only.
//! - `DB_TCP_TIMEOUT_MS`: TCP timeout in milliseconds. See the doc comment for more details.

use crate::config::Base;
use crate::{env, Env};

pub struct DatabasePools {
    /// Settings for the primary database. This is usually writeable, but will be read-only in
    /// some configurations.
    pub primary: DbPoolConfig,
    /// An optional follower database. Always read-only.
    pub replica: Option<DbPoolConfig>,
    /// Number of seconds to wait for unacknowledged TCP packets before treating the connection as
    /// broken. This value will determine how long crates.io stays unavailable in case of full
    /// packet loss between the application and the database: setting it too high will result in an
    /// unnecessarily long outage (before the unhealthy database logic kicks in), while setting it
    /// too low might result in healthy connections being dropped.
    pub tcp_timeout_ms: u64,
    /// Whether to enforce that all the database connections are encrypted with TLS.
    pub enforce_tls: bool,
}

#[derive(Debug)]
pub struct DbPoolConfig {
    pub url: String,
    pub read_only_mode: bool,
    pub pool_size: u32,
    pub min_idle: Option<u32>,
}

impl DatabasePools {
    pub fn are_all_read_only(&self) -> bool {
        self.primary.read_only_mode
    }
}

impl DatabasePools {
    const DEFAULT_POOL_SIZE: u32 = 3;

    /// Load settings for one or more database pools from the environment
    ///
    /// # Panics
    ///
    /// This function panics if `DB_OFFLINE=leader` but `READ_ONLY_REPLICA_URL` is unset.
    pub fn full_from_environment(base: &Base) -> Self {
        let leader_url = env("DATABASE_URL");
        let follower_url = dotenv::var("READ_ONLY_REPLICA_URL").ok();
        let read_only_mode = dotenv::var("READ_ONLY_MODE").is_ok();

        let primary_pool_size = match dotenv::var("DB_PRIMARY_POOL_SIZE") {
            Ok(num) => num.parse().expect("couldn't parse DB_PRIMARY_POOL_SIZE"),
            _ => Self::DEFAULT_POOL_SIZE,
        };

        let replica_pool_size = match dotenv::var("DB_REPLICA_POOL_SIZE") {
            Ok(num) => num.parse().expect("couldn't parse DB_REPLICA_POOL_SIZE"),
            _ => Self::DEFAULT_POOL_SIZE,
        };

        let primary_min_idle = match dotenv::var("DB_PRIMARY_MIN_IDLE") {
            Ok(num) => Some(num.parse().expect("couldn't parse DB_PRIMARY_MIN_IDLE")),
            _ => None,
        };

        let replica_min_idle = match dotenv::var("DB_REPLICA_MIN_IDLE") {
            Ok(num) => Some(num.parse().expect("couldn't parse DB_REPLICA_MIN_IDLE")),
            _ => None,
        };

        let tcp_timeout_ms = match dotenv::var("DB_TCP_TIMEOUT_MS") {
            Ok(num) => num.parse().expect("couldn't parse DB_TCP_TIMEOUT_MS"),
            Err(_) => 15 * 1000, // 15 seconds
        };

        let enforce_tls = base.env == Env::Production;

        match dotenv::var("DB_OFFLINE").as_deref() {
            // The actual leader is down, use the follower in read-only mode as the primary and
            // don't configure a replica.
            Ok("leader") => Self {
                primary: DbPoolConfig {
                    url: follower_url
                        .expect("Must set `READ_ONLY_REPLICA_URL` when using `DB_OFFLINE=leader`."),
                    read_only_mode: true,
                    pool_size: primary_pool_size,
                    min_idle: primary_min_idle,
                },
                replica: None,
                tcp_timeout_ms,
                enforce_tls,
            },
            // The follower is down, don't configure the replica.
            Ok("follower") => Self {
                primary: DbPoolConfig {
                    url: leader_url,
                    read_only_mode,
                    pool_size: primary_pool_size,
                    min_idle: primary_min_idle,
                },
                replica: None,
                tcp_timeout_ms,
                enforce_tls,
            },
            _ => Self {
                primary: DbPoolConfig {
                    url: leader_url,
                    read_only_mode,
                    pool_size: primary_pool_size,
                    min_idle: primary_min_idle,
                },
                replica: follower_url.map(|url| DbPoolConfig {
                    url,
                    // Always enable read-only mode for the follower. In staging, we attach the
                    // same leader database to both environment variables and this ensures the
                    // connection is opened read-only even when attached to a writeable database.
                    read_only_mode: true,
                    pool_size: replica_pool_size,
                    min_idle: replica_min_idle,
                }),
                tcp_timeout_ms,
                enforce_tls,
            },
        }
    }

    pub fn test_from_environment() -> Self {
        DatabasePools {
            primary: DbPoolConfig {
                url: env("TEST_DATABASE_URL"),
                read_only_mode: false,
                pool_size: 1,
                min_idle: None,
            },
            replica: None,
            tcp_timeout_ms: 1000, // 1 second
            enforce_tls: false,
        }
    }
}
