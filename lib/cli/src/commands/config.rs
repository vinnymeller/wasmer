use crate::VERSION;
use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::str::ParseBoolError;
use wasmer_registry::WasmerConfig;

#[derive(Debug, Parser)]
/// The options for the `wasmer config` subcommand: `wasmer config get prefix`
pub enum Config {
    /// Get a value from the current wasmer config
    #[clap(subcommand)]
    Get(RetrievableConfigField),
    /// Set a value in the current wasmer config
    #[clap(subcommand)]
    Set(StorableConfigField),
}

/// Value that can be queried from the wasmer config
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, clap::Subcommand)]
pub enum RetrievableConfigField {
    /// Print the wasmer installation path (WASMER_DIR)
    Prefix,
    /// Print the /bin directory where wasmer is installed
    Bindir,
    /// Print the /include dir
    Includedir,
    /// Print the /lib dir
    Libdir,
    /// Print the linker flags for linking to libwasmer
    Libs,
    /// Print the compiler flags for linking to libwasmer
    Cflags,
    /// Print the pkg-config configuration
    PkgConfig,
    /// Print the path to the configuration file
    #[clap(name = "config.path")]
    ConfigPath,
    /// Print the registry URL of the currently active registry
    #[clap(name = "registry.url")]
    RegistryUrl,
    /// Print the token for the currently active registry or nothing if not logged in
    #[clap(name = "registry.token")]
    RegistryToken,
    /// Print whether telemetry is currently enabled
    #[clap(name = "telemetry.enabled")]
    TelemetryEnabled,
    /// Print whether update notifications are enabled
    #[clap(name = "update-notifications.enabled")]
    UpdateNotificationsEnabled,
    /// Print the proxy URL
    #[clap(name = "proxy.url")]
    ProxyUrl,
}

/// Setting that can be stored in the wasmer config
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, clap::Subcommand)]
pub enum StorableConfigField {
    /// `registry.url`
    #[clap(name = "registry.url")]
    RegistryUrl(SetRegistryUrl),
    /// `registry.token`
    #[clap(name = "registry.token")]
    RegistryToken(SetRegistryToken),
    /// `telemetry.enabled`
    #[clap(name = "telemetry.enabled")]
    TelemetryEnabled(SetTelemetryEnabled),
    /// `update-notifications.url`
    #[clap(name = "update-notifications.enabled")]
    UpdateNotificationsEnabled(SetUpdateNotificationsEnabled),
    /// `proxy.url`
    #[clap(name = "proxy.url")]
    ProxyUrl(SetProxyUrl),
}

/// Set the current active registry URL
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Parser)]
pub struct SetRegistryUrl {
    /// Url of the registry
    #[clap(name = "URL")]
    pub url: String,
}

/// Set or change the token for the current active registry
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Parser)]
pub struct SetRegistryToken {
    /// Token to set
    #[clap(name = "TOKEN")]
    pub token: String,
}

/// Set if update notifications are enabled
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Parser)]
pub struct SetUpdateNotificationsEnabled {
    /// Whether to enable update notifications
    #[clap(name = "ENABLED", possible_values = ["true", "false"])]
    pub enabled: BoolString,
}

/// "true" or "false" for handling input in the CLI
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoolString(pub bool);

impl std::str::FromStr for BoolString {
    type Err = ParseBoolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(bool::from_str(s)?))
    }
}

/// Set if telemetry is enabled
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Parser)]
pub struct SetTelemetryEnabled {
    /// Whether to enable telemetry
    #[clap(name = "ENABLED", possible_values = ["true", "false"])]
    pub enabled: BoolString,
}

/// Set if a proxy URL should be used
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Parser)]
pub struct SetProxyUrl {
    /// Set if a proxy URL should be used (empty = unset proxy)
    #[clap(name = "URL")]
    pub url: Option<String>,
}

impl Config {
    /// Runs logic for the `config` subcommand
    pub fn execute(&self) -> Result<()> {
        self.inner_execute()
            .context("failed to retrieve the wasmer config".to_string())
    }
    fn inner_execute(&self) -> Result<()> {
        use self::Config::{Get, Set};

        let key = "WASMER_DIR";
        let wasmer_dir = env::var(key)
            .or_else(|e| {
                option_env!("WASMER_INSTALL_PREFIX")
                    .map(str::to_string)
                    .ok_or(e)
            })
            .context(format!(
                "failed to retrieve the {} environment variables",
                key
            ))?;

        let prefix = PathBuf::from(wasmer_dir);

        let prefixdir = prefix.display().to_string();
        let bindir = prefix.join("bin").display().to_string();
        let includedir = prefix.join("include").display().to_string();
        let libdir = prefix.join("lib").display().to_string();
        let cflags = format!("-I{}", includedir);
        let libs = format!("-L{} -lwasmer", libdir);

        match self {
            Get(g) => match g {
                RetrievableConfigField::PkgConfig => {
                    println!("prefix={}", prefixdir);
                    println!("exec_prefix={}", bindir);
                    println!("includedir={}", includedir);
                    println!("libdir={}", libdir);
                    println!();
                    println!("Name: wasmer");
                    println!("Description: The Wasmer library for running WebAssembly");
                    println!("Version: {}", VERSION);
                    println!("Cflags: {}", cflags);
                    println!("Libs: {}", libs);
                }
                RetrievableConfigField::Prefix => {
                    println!("{}", prefixdir);
                }
                RetrievableConfigField::Bindir => {
                    println!("{}", bindir);
                }
                RetrievableConfigField::Includedir => {
                    println!("{}", includedir);
                }
                RetrievableConfigField::Libdir => {
                    println!("{}", libdir);
                }
                RetrievableConfigField::Libs => {
                    println!("{}", libs);
                }
                RetrievableConfigField::Cflags => {
                    println!("{}", cflags);
                }
                RetrievableConfigField::ConfigPath => {
                    let path = WasmerConfig::get_file_location()
                        .map_err(|e| anyhow::anyhow!("could not find config file: {e}"))?;
                    println!("{}", path.display());
                }
                other => {
                    let config = WasmerConfig::from_file()
                        .map_err(|e| anyhow::anyhow!("could not find config file: {e}"))?;
                    match other {
                        RetrievableConfigField::RegistryUrl => {
                            println!("{}", config.registry.get_current_registry());
                        }
                        RetrievableConfigField::RegistryToken => {
                            if let Some(s) = config.registry.get_login_token_for_registry(
                                &config.registry.get_current_registry(),
                            ) {
                                println!("{s}");
                            }
                        }
                        RetrievableConfigField::ProxyUrl => {
                            if let Some(s) = config.proxy.url.as_ref() {
                                println!("{s}");
                            }
                        }
                        RetrievableConfigField::TelemetryEnabled => {
                            println!("{:?}", config.telemetry_enabled);
                        }
                        RetrievableConfigField::UpdateNotificationsEnabled => {
                            println!("{:?}", config.update_notifications_enabled);
                        }
                        _ => {}
                    }
                }
            },
            Set(s) => {
                let config_file = WasmerConfig::get_file_location()
                    .map_err(|e| anyhow::anyhow!("could not find config file {e}"))?;
                let mut config = WasmerConfig::from_file().map_err(|e| {
                    anyhow::anyhow!(
                        "could not find config file {e} at {}",
                        config_file.display()
                    )
                })?;

                match s {
                    StorableConfigField::RegistryUrl(s) => {
                        config.registry.set_current_registry(&s.url);
                        let current_registry = config.registry.get_current_registry();
                        if let Some(u) = wasmer_registry::utils::get_username().ok().and_then(|o| o)
                        {
                            println!("Successfully logged into registry {current_registry:?} as user {u:?}");
                        }
                    }
                    StorableConfigField::RegistryToken(t) => {
                        config.registry.set_login_token_for_registry(
                            &config.registry.get_current_registry(),
                            &t.token,
                            wasmer_registry::config::UpdateRegistry::LeaveAsIs,
                        );
                    }
                    StorableConfigField::TelemetryEnabled(t) => {
                        config.telemetry_enabled = t.enabled.0;
                    }
                    StorableConfigField::ProxyUrl(p) => {
                        config.proxy.url = p.url.clone();
                    }
                    StorableConfigField::UpdateNotificationsEnabled(u) => {
                        config.update_notifications_enabled = u.enabled.0;
                    }
                }

                config
                    .save(config_file)
                    .with_context(|| anyhow::anyhow!("could not save config file"))?;
            }
        }
        Ok(())
    }
}
