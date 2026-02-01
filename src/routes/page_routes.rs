use actix_web::web;
use crate::handlers::page_handlers;

/// 配置页面路由
/// 单职责：仅负责 HTML 页面的路由配置
pub fn configure_page_routes(cfg: &mut web::ServiceConfig) {
    // 主页
    cfg.route("/", web::get().to(page_handlers::index));
    cfg.route("/index", web::get().to(page_handlers::index));

    // 文章页面
    cfg.route("/passage", web::get().to(page_handlers::passage_list));
    cfg.route("/passage/{id}", web::get().to(page_handlers::passage_detail));

    // 归档页面
    cfg.route("/collect", web::get().to(page_handlers::collect));

    // 关于页面
    cfg.route("/about", web::get().to(page_handlers::about));

    // 友链页面
    cfg.route("/friends", web::get().to(page_handlers::friends));

    // Markdown 编辑器
    cfg.route("/markdown-editor", web::get().to(page_handlers::markdown_editor));

    // 键盘测试页面
    cfg.route("/keyboard-test", web::get().to(page_handlers::keyboard_test));

    // 管理后台
    cfg.route("/admin", web::get().to(page_handlers::admin));

    // 状态页面
    cfg.route("/status/{status}", web::get().to(page_handlers::status_page));

    // 健康检查
    cfg.route("/health", web::get().to(page_handlers::health));
}
