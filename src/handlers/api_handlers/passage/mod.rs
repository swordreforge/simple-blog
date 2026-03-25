// Passage 模块 - 拆分后的模块结构
//
// 本模块将原本巨大的 passage.rs (1900+ 行) 拆分为多个子模块：
// - crud.rs: CRUD 操作处理器
// - validation.rs: 数据验证逻辑
// - markdown.rs: Markdown 处理相关函数
// - query_handlers.rs: 查询参数处理器
// - crud_helper.rs: 缓存辅助函数
// - version.rs: 文章版本历史管理处理器

pub mod crud;
pub mod crud_helper;
pub mod markdown;
pub mod query_handlers;
pub mod validation;
// pub mod version; // 暂时注释，待修复编译错误后启用

// 重新导出所有公共接口，保持向后兼容
pub use crud::{create, delete, delete_batch, get, get_latest, list, update};

pub use query_handlers::{delete_by_query, get_by_query, update_by_query};

// 导出版本管理 Handler
// pub use version::{
//     list_versions,
//     create_version,
//     get_version,
//     delete_version,
//     diff_versions,
//     restore_version,
//     undo,
//     redo,
// };
