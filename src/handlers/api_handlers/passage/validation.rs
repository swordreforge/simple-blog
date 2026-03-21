use crate::db::get_db_pool_sync;
use crate::db::repositories::{CategoryRepository, TagRepository};
use std::sync::Arc;

/// 确保标签存在于 tags 表中
pub async fn ensure_tags_exist(tag_names: &[String]) -> Result<(), String> {
    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let tag_repo = TagRepository::new(Arc::new(pool.clone()));

    for tag_name in tag_names {
        // 查找标签，如果不存在则创建
        if tag_repo.get_by_name(tag_name).await.is_err() {
            let now = chrono::Utc::now();
            let new_tag = crate::db::models::Tag {
                id: None,
                name: tag_name.clone(),
                description: format!("用户创建的标签: {}", tag_name),
                color: "#007bff".to_string(),
                category_id: 0,
                sort_order: 0,
                is_enabled: true,
                created_at: now,
                updated_at: now,
            };

            tag_repo
                .create(&new_tag)
                .await
                .map_err(|e| format!("创建标签失败: {}", e))?;
        }
    }

    Ok(())
}

/// 确保分类存在于 categories 表中
pub async fn ensure_category_exist(category_name: &str) -> Result<(), String> {
    // 如果分类为空或"未分类"，跳过
    if category_name.is_empty() || category_name == "未分类" {
        return Ok(());
    }

    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let category_repo = CategoryRepository::new(Arc::new(pool.clone()));

    // 查找分类，如果不存在则创建
    if category_repo.get_by_name(category_name).await.is_err() {
        let now = chrono::Utc::now();
        let new_category = crate::db::models::Category {
            id: None,
            name: category_name.to_string(),
            description: format!("用户创建的分类: {}", category_name),
            icon: "📁".to_string(),
            sort_order: 0,
            is_enabled: true,
            created_at: now,
            updated_at: now,
        };

        category_repo
            .create(&new_category)
            .await
            .map_err(|e| format!("创建分类失败: {}", e))?;
    }

    Ok(())
}
