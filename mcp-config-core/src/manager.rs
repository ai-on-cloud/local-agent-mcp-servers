use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::RwLock;
use zeroclaw::config::schema::Config;
use zeroclaw::security::SecretStore;

pub struct ConfigManager {
    config: RwLock<Config>,
    zeroclaw_dir: PathBuf,
}

impl ConfigManager {
    pub fn new(config: Config) -> Self {
        let zeroclaw_dir = config
            .config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            config: RwLock::new(config),
            zeroclaw_dir,
        }
    }

    pub async fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Config) -> R,
    {
        let guard = self.config.read().await;
        f(&guard)
    }

    pub async fn write<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Config) -> R,
    {
        let mut guard = self.config.write().await;
        let result = f(&mut guard);
        guard.save()?;
        Ok(result)
    }

    pub fn secret_store(&self, encrypt: bool) -> SecretStore {
        SecretStore::new(&self.zeroclaw_dir, encrypt)
    }

    pub async fn reload(&self) -> Result<()> {
        let new_config = Config::load_or_init()?;
        let mut guard = self.config.write().await;
        *guard = new_config;
        Ok(())
    }
}
