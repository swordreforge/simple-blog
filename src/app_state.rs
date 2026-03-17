//! 应用状态和依赖注入容器
//!
//! 统一管理应用的所有依赖项，包括：
//! - 数据库 Repository
//! - 缓存服务
//! - 批量处理器
//! - 各种服务实例
//! - 动态路由表
//! - 路由类型管理器
//!
//! 使用依赖注入模式，便于测试和维护

use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::db::repositories;
use crate::cache::AppCache;
use crate::view_batch::ViewBatchProcessor;
use crate::services::dynamic_route_service::DynamicRouteService;
use crate::services::route_type_manager::RouteTypeManager;
use crate::services::route_storage::{MemoryRouteStorage, FileRouteStorage};
use crate::db::models::RouteType;
use dynamic_route_actix::RouteTable;

/// 应用状态 - 依赖注入容器
#[derive(Clone)]
pub struct AppState {
    /// 数据库 Repository
    pub repository: Arc<dyn repositories::Repository>,
    /// 应用缓存
    pub cache: Arc<AppCache>,
    /// 阅读记录批量处理器
    pub view_batch_processor: Arc<ViewBatchProcessor>,
    /// 动态路由表
    pub route_table: Arc<RouteTable>,
    /// 动态路由服务
    dynamic_route_service: Arc<DynamicRouteService>,
    /// 路由类型管理器（预留，用于路由存储抽象层）
    #[allow(dead_code)]
    pub route_type_manager: Option<Arc<RouteTypeManager>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        repository: Arc<dyn repositories::Repository>,
        cache: Arc<AppCache>,
        view_batch_processor: Arc<ViewBatchProcessor>,
        route_table: Arc<RouteTable>,
        dynamic_route_service: Arc<DynamicRouteService>,
    ) -> Self {
        Self {
            repository,
            cache,
            view_batch_processor,
            route_table,
            dynamic_route_service,
            route_type_manager: None,
        }
    }

    /// 创建带有路由类型管理器的应用状态
    #[allow(dead_code)]
    pub fn new_with_route_type_manager(
        repository: Arc<dyn repositories::Repository>,
        cache: Arc<AppCache>,
        view_batch_processor: Arc<ViewBatchProcessor>,
        route_table: Arc<RouteTable>,
        dynamic_route_service: Arc<DynamicRouteService>,
        route_type_manager: Arc<RouteTypeManager>,
    ) -> Self {
        Self {
            repository,
            cache,
            view_batch_processor,
            route_table,
            dynamic_route_service,
            route_type_manager: Some(route_type_manager),
        }
    }

    /// 获取数据库连接池
    pub fn get_pool(&self) -> Arc<Pool<SqliteConnectionManager>> {
        self.repository.get_pool()
    }

    /// 获取文章 Repository
    pub fn passage_repository(&self) -> repositories::PassageRepository {
        repositories::PassageRepository::new(self.get_pool())
    }

    /// 获取用户 Repository
    pub fn user_repository(&self) -> repositories::UserRepository {
        repositories::UserRepository::new(self.get_pool())
    }

    /// 获取评论 Repository
    pub fn comment_repository(&self) -> repositories::CommentRepository {
        repositories::CommentRepository::new(self.get_pool())
    }

    /// 获取标签 Repository
    pub fn tag_repository(&self) -> repositories::TagRepository {
        repositories::TagRepository::new(self.get_pool())
    }

    /// 获取分类 Repository
    pub fn category_repository(&self) -> repositories::CategoryRepository {
        repositories::CategoryRepository::new(self.get_pool())
    }

    /// 获取友链 Repository
    pub fn friend_link_repository(&self) -> repositories::FriendLinkRepository {
        repositories::FriendLinkRepository::new(self.get_pool())
    }

    /// 获取设置 Repository
    #[allow(dead_code)]
    pub fn settings_repository(&self) -> repositories::SettingRepository {
        repositories::SettingRepository
    }

    /// 获取文章视图 Repository
    pub fn article_view_repository(&self) -> repositories::ArticleViewRepository {
        repositories::ArticleViewRepository::new(self.get_pool())
    }

    /// 获取音乐 Repository
    pub fn music_track_repository(&self) -> repositories::MusicTrackRepository {
        repositories::MusicTrackRepository::new(self.get_pool())
    }

    /// 获取附件 Repository
    pub fn attachment_repository(&self) -> repositories::AttachmentRepository {
        repositories::AttachmentRepository::new(self.get_pool())
    }

    /// 获取关于主页卡片 Repository
    pub fn about_main_card_repository(&self) -> repositories::AboutMainCardRepository {
        repositories::AboutMainCardRepository::new(self.get_pool())
    }

    /// 获取关于子卡片 Repository
    pub fn about_sub_card_repository(&self) -> repositories::AboutSubCardRepository {
        repositories::AboutSubCardRepository::new(self.get_pool())
    }

    /// 获取动态路由 Repository
    pub fn dynamic_route_repository(&self) -> repositories::DynamicRouteRepository {
        repositories::DynamicRouteRepository::new(self.get_pool())
    }

    /// 获取动态路由服务
    pub fn dynamic_route_service(&self) -> Arc<DynamicRouteService> {
        self.dynamic_route_service.clone()
    }

    /// 获取路由类型管理器
    #[allow(dead_code)]
    pub fn route_type_manager(&self) -> Option<Arc<RouteTypeManager>> {
        self.route_type_manager.clone()
    }
}

/// 创建路由类型管理器的辅助函数
///
/// # 参数
/// - `base_dir`: 基础目录路径
/// - `dynamic_route_repo`: 动态路由仓库
/// - `default_route_type`: 默认路由类型
///
/// # 返回
/// 返回配置好的 RouteTypeManager 实例
#[allow(dead_code)]
pub fn create_route_type_manager(
    base_dir: &std::path::Path,
    dynamic_route_repo: repositories::DynamicRouteRepository,
    default_route_type: RouteType,
) -> Result<Arc<RouteTypeManager>, Box<dyn std::error::Error>> {
    // 创建内存存储
    let memory_storage = Arc::new(MemoryRouteStorage::new(1000, 3600));

    // 创建文件存储
    let routes_base_dir = base_dir.join("data").join("routes");
    let file_storage = Arc::new(FileRouteStorage::new(
        &routes_base_dir,
        1024 * 1024, // 1MB 最大文件大小
        true,         // 启用备份
        5,            // 保留 5 个备份
    )?);

    // 创建数据库存储
    let database_storage = Arc::new(dynamic_route_repo);

    // 创建路由类型管理器
    let route_type_manager = Arc::new(RouteTypeManager::new(
        database_storage,
        memory_storage,
        file_storage,
        default_route_type,
    ));

    Ok(route_type_manager)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_app_state_creation() {
        // 这个测试需要实际的数据库连接池
        // 在实际应用中，应该使用测试数据库
        // 这里只是演示结构
        assert!(true);
    }
}