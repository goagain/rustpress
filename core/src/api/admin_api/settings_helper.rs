//! Helper functions for settings KV operations

use crate::entity::settings;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::sync::Arc;

/// Get a setting value by key
pub async fn get_setting(
    db: &Arc<sea_orm::DatabaseConnection>,
    key: &str,
) -> Result<Option<String>, sea_orm::DbErr> {
    let setting = settings::Entity::find_by_id(key).one(db.as_ref()).await?;
    Ok(setting.map(|s| s.value))
}

/// Set a setting value by key
pub async fn set_setting(
    db: &Arc<sea_orm::DatabaseConnection>,
    key: &str,
    value: String,
    description: Option<String>,
) -> Result<(), sea_orm::DbErr> {
    let existing = settings::Entity::find_by_id(key).one(db.as_ref()).await?;

    if let Some(existing_model) = existing {
        let mut active_model: settings::ActiveModel = existing_model.into();
        active_model.value = Set(value);
        if let Some(desc) = description {
            active_model.description = Set(Some(desc));
        }
        active_model.update(db.as_ref()).await?;
    } else {
        let setting = settings::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value),
            description: Set(description),
            ..Default::default()
        };
        setting.insert(db.as_ref()).await?;
    }

    Ok(())
}

/// Delete a setting by key
pub async fn delete_setting(
    db: &Arc<sea_orm::DatabaseConnection>,
    key: &str,
) -> Result<bool, sea_orm::DbErr> {
    let result = settings::Entity::delete_by_id(key)
        .exec(db.as_ref())
        .await?;
    Ok(result.rows_affected > 0)
}

/// Get all settings as a HashMap
pub async fn get_all_settings(
    db: &Arc<sea_orm::DatabaseConnection>,
) -> Result<std::collections::HashMap<String, String>, sea_orm::DbErr> {
    let settings_models = settings::Entity::find().all(db.as_ref()).await?;
    let mut settings_map = std::collections::HashMap::new();
    for setting in settings_models {
        settings_map.insert(setting.key.clone(), setting.value);
    }
    Ok(settings_map)
}
