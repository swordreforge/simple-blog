//! 路由持久化存储模块
//!
//! 提供多种存储后端实现，包括文件存储和内存存储。

mod file_storage;
mod memory_storage;
mod traits;

pub use file_storage::FileStorage;
pub use memory_storage::MemoryStorage;
pub use traits::{KeyValueStorage, RouteStorage};
