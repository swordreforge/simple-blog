// Passage 模块 - 拆分后的模块结构
//
// 本模块将原本巨大的 passage.rs (1900+ 行) 拆分为多个子模块：
// - crud.rs: CRUD 操作处理器
// - validation.rs: 数据验证逻辑
// - markdown.rs: Markdown 处理相关函数
// - query_handlers.rs: 查询参数处理器

pub mod crud;
pub mod validation;
pub mod markdown;
pub mod query_handlers;

// 重新导出所有公共接口，保持向后兼容
pub use crud::{
    list,
    get,
    create,
    update,
    delete,
    delete_batch,
};

pub use query_handlers::{
    get_by_query,
    update_by_query,
    delete_by_query,
};