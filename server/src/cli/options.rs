// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::{RegionId, ServerId, ServerKind, ServerToken};
use clap::{Parser, command};
use log::LevelFilter;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// Server options, to be specified as arguments.
#[derive(Debug, Parser)]
#[command(name = "kodiak_server")]
pub struct Options {
    /// Override bot count to a constant.
    #[arg(long)]
    pub bots: Option<u16>,
    
    /// Log incoming HTTP requests
    #[arg(long, default_value = "info")]
    pub debug_http: String,
    
    /// Log game diagnostics
    #[arg(long, default_value = "info")]
    pub debug_game: String,
    
    /// Log game engine diagnostics
    #[arg(long, default_value = "warn")]
    pub debug_engine: String,
    
    /// Log plasma diagnostics
    #[arg(long, default_value = "warn")]
    pub debug_plasma: String,
    
    #[arg(long, default_value = "./domain_backup.json")]
    pub domain_backup: String,
    
    /// Server ID.
    #[arg(long)]
    server_id: Option<ServerId>,
    
    /// Alternative to `server_id`.
    #[arg(long)]
    hostname: Option<String>,
    
    /// Initial secret key unique to this server.
    #[arg(long)]
    pub server_token: Option<ServerToken>,
    
    /// Override the server ipv4.
    #[arg(long)]
    pub ipv4_address: Option<Ipv4Addr>,
    
    /// Override the server ipv6 (currently unused).
    #[arg(long)]
    pub ipv6_address: Option<Ipv6Addr>,
    
    #[arg(long)]
    pub http_port: Option<u16>,
    
    #[arg(long)]
    pub https_port: Option<u16>,
    
    /// Override the region id.
    #[arg(long)]
    pub region_id: Option<RegionId>,
    
    /// Domain (without server id prepended).
    #[allow(dead_code)]
    #[deprecated = "now from game id"]
    #[arg(long)]
    pub domain: Option<String>,
    
    /// Certificate chain path.
    #[arg(long)]
    #[deprecated]
    pub certificate_path: Option<String>,
    
    /// Private key path.
    #[arg(long)]
    #[deprecated]
    pub private_key_path: Option<String>,
    
    /// HTTP request bandwidth limiting (in bytes per second).
    #[arg(long, default_value = "500000")]
    pub http_bandwidth_limit: u32,
    
    /// HTTP request rate limiting burst (in bytes).
    ///
    /// Implicit minimum is double the total size of the client static files.
    #[arg(long)]
    pub http_bandwidth_burst: Option<u32>,
    
    /// Client authenticate rate limiting period (in seconds).
    #[arg(long, default_value = "10")]
    pub client_authenticate_rate_limit: u64,
    
    /// Client authenticate rate limiting burst.
    #[arg(long, default_value = "16")]
    pub client_authenticate_burst: u32,
    
    #[arg(long)]
    pub cpu_profile: bool,
    
    #[arg(long)]
    pub heap_profile: bool,
}

impl Options {
    pub(crate) const STANDARD_HTTPS_PORT: u16 = 443;
    pub(crate) const STANDARD_HTTP_PORT: u16 = 80;
    
    // 转换字符串为 LevelFilter
    pub fn debug_http_filter(&self) -> LevelFilter {
        LevelFilter::from_str(&self.debug_http).unwrap_or(LevelFilter::Info)
    }
    
    pub fn debug_game_filter(&self) -> LevelFilter {
        LevelFilter::from_str(&self.debug_game).unwrap_or(LevelFilter::Info)
    }
    
    pub fn debug_engine_filter(&self) -> LevelFilter {
        LevelFilter::from_str(&self.debug_engine).unwrap_or(LevelFilter::Warn)
    }
    
    pub fn debug_plasma_filter(&self) -> LevelFilter {
        LevelFilter::from_str(&self.debug_plasma).unwrap_or(LevelFilter::Warn)
    }

    #[deprecated]
    pub(crate) fn certificate_private_key_paths(&self) -> Option<(&str, &str)> {
        #[allow(deprecated)]
        self.certificate_path
            .as_deref()
            .zip(self.private_key_path.as_deref())
    }

    pub(crate) fn server_id(&self) -> Option<ServerId> {
        self.server_id.or_else(|| {
            self.hostname.as_ref().and_then(|hostname| {
                hostname
                    .split('.')
                    .next()
                    .unwrap()
                    .parse()
                    .ok()
                    .map(|number| ServerId {
                        number,
                        kind: ServerKind::Cloud,
                    })
            })
        })
    }

    pub(crate) fn bandwidth_burst(&self, static_size: usize) -> u32 {
        self.http_bandwidth_burst.unwrap_or(static_size as u32 * 2)
    }

    pub(crate) fn http_and_https_ports(&self) -> (u16, u16) {
        #[cfg(unix)]
        let priviledged = nix::unistd::Uid::effective().is_root();

        #[cfg(not(unix))]
        let priviledged = true;

        let (http_port, https_port) = if priviledged {
            (Self::STANDARD_HTTP_PORT, Self::STANDARD_HTTPS_PORT)
        } else {
            (8080, 8443)
        };

        let ports = (
            self.http_port.unwrap_or(http_port),
            self.https_port.unwrap_or(https_port),
        );
        log::info!("HTTP port: {}, HTTPS port: {}", ports.0, ports.1);
        ports
    }
}

