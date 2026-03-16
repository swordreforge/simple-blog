//! 路由持久化存储模块
//!
//! 提供多种存储后端实现，包括文件存储、内存存储和数据库存储。

#[cfg(feature = "database")]
mod database_storage;
mod file_storage;
mod memory_storage;
mod traits;

#[cfg(feature = "database")]
pub use database_storage::{DatabaseStorage, DatabaseStorageConfig, DatabaseType, DatabaseStorageError, RouteVersion};
pub use file_storage::FileStorage;
pub use memory_storage::MemoryStorage;
pub use traits::{KeyValueStorage, RouteStorage};
