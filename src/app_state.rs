use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use handlebars::Handlebars;
use serde::Serialize;
use sqlx::PgPool;
use tracing::info;
use whynot_errors::{AppError, AppResult};

use crate::{
    db::{get_config, ConfigKeys},
    models::TemplateData,
};

pub type Locked<T> = Arc<RwLock<T>>;
pub fn locker<T>(obj: T) -> Locked<T> {
    Arc::new(RwLock::new(obj))
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    pub registry: Locked<Handlebars<'static>>,
    pub timer: Locked<SystemTime>,
    pub nonce_container: Locked<NonceContainer>,
}

#[derive(Clone, Debug)]
pub struct NonceContainer {
    pub nonce: String,
}

#[derive(Serialize)]
pub struct TestState {
    msg: String,
}

impl AppState {
    // TODO: Check mutex ordering, could be faster.
    pub async fn reload_templates(&self) -> AppResult<()> {
        let db_nonce_future = self.get_db_nonce().await;

        let mutex = self.registry.clone();

        let templates: Vec<TemplateData> = sqlx::query_as("SELECT * FROM templates;")
            .fetch_all(&self.db)
            .await
            .map_err(AppError::new)?;

        let mut registry = mutex.write().map_err(AppError::new)?;

        // Clear anything that currently exists.
        registry.clear_templates();

        for template in templates {
            registry
                .register_template_string(template.key.as_str(), template.template.as_str())
                .map_err(AppError::new)?;
        }

        let nonce_container_mutex = self.nonce_container.clone();
        let mut nonce_container = nonce_container_mutex.write().map_err(AppError::new)?;
        nonce_container.nonce = db_nonce_future;

        Ok(())
    }

    fn set_db_nonce(&self, value: String) -> AppResult<()> {
        Ok(())
    }

    pub fn reset_timer(&self) -> AppResult<()> {
        let timer_mutex = self.timer.clone();
        let mut timer = timer_mutex.write().map_err(AppError::new)?;
        *timer = SystemTime::now();

        Ok(())
    }

    // TODO: Make the timer value here configurable somehow
    pub fn timer_up(&self) -> AppResult<bool> {
        let timer_mutex = self.timer.clone();
        let timer = timer_mutex.read().map_err(AppError::new)?;

        Ok(timer.elapsed().map(|t| t.as_secs()).unwrap_or(0) > 300)
    }

    pub async fn get_db_nonce(&self) -> String {
        get_config(&self.db, ConfigKeys::TemplateNonce)
            .await
            .unwrap_or("lolnotset".to_string())
    }

    // TODO
    pub fn get_nonce(&self) -> AppResult<String> {
        let nonce_container_mutex = self.nonce_container.clone();
        let nonce_container = nonce_container_mutex.read().map_err(AppError::new)?;
        Ok(nonce_container.nonce.clone())
    }

    // TODO: Also add the nonce
    pub async fn should_reload(&self) -> AppResult<bool> {
        let timer_result = self.timer_up()?;

        if !timer_result {
            return Ok(false);
        }

        self.reset_timer()?;

        let nonce = self.get_nonce()?;
        let db_nonce = self.get_db_nonce().await;

        Ok(nonce != db_nonce)
    }

    pub async fn refresh_if_needed(&self) -> AppResult<()> {
        // If statements are expressions, so no need for return statements on either branch.
        if self.should_reload().await? {
            info!("Reloading Templates");
            self.reload_templates().await
        } else {
            Ok(())
        }
    }
}
