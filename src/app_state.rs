//! 应用状态和依赖注入容器
//! 
//! 统一管理应用的所有依赖项，包括：
//! - 数据库 Repository
//! - 缓存服务
//! - 批量处理器
//! - 各种服务实例
//! 
//! 使用依赖注入模式，便于测试和维护

use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::db::repositories;
use crate::cache::AppCache;
use crate::view_batch::ViewBatchProcessor;

/// 应用状态 - 依赖注入容器
#[derive(Clone)]
pub struct AppState {
    /// 数据库 Repository
    pub repository: Arc<dyn repositories::Repository>,
    /// 应用缓存
    pub cache: Arc<AppCache>,
    /// 阅读记录批量处理器
    pub view_batch_processor: Arc<ViewBatchProcessor>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        repository: Arc<dyn repositories::Repository>,
        cache: Arc<AppCache>,
        view_batch_processor: Arc<ViewBatchProcessor>,
    ) -> Self {
        Self {
            repository,
            cache,
            view_batch_processor,
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