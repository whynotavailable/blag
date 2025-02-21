use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use handlebars::Handlebars;
use serde::Serialize;
use sqlx::PgPool;
use whynot_errors::{AppError, AppResult};

use crate::models::TemplateData;

pub type Locked<T> = Arc<RwLock<T>>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    pub registry: Locked<Handlebars<'static>>,
    pub timer: Locked<SystemTime>,
}

#[derive(Serialize)]
pub struct TestState {
    msg: String,
}

impl AppState {
    pub async fn reload_templates(&self) -> AppResult<()> {
        let mutex = self.registry.clone();

        let templates: Vec<TemplateData> = sqlx::query_as("SELECT * FROM templates;")
            .fetch_all(&self.db)
            .await
            .map_err(AppError::from)?;

        let mut registry = mutex.write().map_err(AppError::from)?;

        // Clear anything that currently exists.
        registry.clear_templates();

        for template in templates {
            registry
                .register_template_string(template.key.as_str(), template.template.as_str())
                .map_err(AppError::from)?;
        }

        Ok(())
    }

    pub fn reset_timer(&self) -> AppResult<()> {
        let timer_mutex = self.timer.clone();
        let mut timer = timer_mutex.write().map_err(AppError::from)?;
        *timer = SystemTime::now();

        Ok(())
    }

    // I don't want to deal with upgrading, so this is a seperate thing.
    pub fn timer_up(&self) -> AppResult<bool> {
        let timer_mutex = self.timer.clone();
        let timer = timer_mutex.read().map_err(AppError::from)?;

        Ok(timer.elapsed().map(|t| t.as_secs()).unwrap_or(0) > 60)
    }

    pub async fn get_db_nonce(&self) -> AppResult<String> {
        Ok("yo".to_string())
    }

    pub fn get_nonce(&self) -> AppResult<String> {
        Ok("yo".to_string())
    }

    // TODO: Also add the nonce
    pub async fn should_reload(&self) -> AppResult<bool> {
        let timer_result = self.timer_up()?;

        if !timer_result {
            return Ok(false);
        }

        let nonce = self.get_nonce()?;
        let db_nonce = self.get_db_nonce().await?;

        Ok(nonce != db_nonce)
    }

    pub async fn refresh_if_needed(&self) -> AppResult<()> {
        // If statements are expressions, so no need for return statements on either branch.
        if self.should_reload().await? {
            self.reload_templates().await
        } else {
            Ok(())
        }
    }
}
