//! 动态路由管理库 - Dynamic Route Management Library
//!
//! 基于 Actix-Web 的动态路由管理库，支持运行时添加、删除和查询路由。
//!
//! # 概述
//!
//! 本库提供了一个灵活的路由管理系统，允许在运行时动态管理路由表。
//! 核心功能包括：
//!
//! - 动态路由注册和管理
//! - 线程安全的路由表
//! - 可扩展的路由处理器接口
//! - 持久化支持（文件、内存、数据库）
//! - Actix-Web 深度集成
//!
//! # 快速开始
//!
//! ```no_run
//! use dynamic_route_actix::{RouteTable, SimpleRoute};
//!
//! #[tokio::main]
//! async fn main() {
//!     // 创建路由表
//!     let table = RouteTable::new();
//!
//!     // 添加简单路由
//!     let route = SimpleRoute::new("Hello, World!", "text/plain");
//!     table.insert("/hello".into(), Box::new(route));
//!
//!     // 查询路由
//!     if let Some(result) = table.get_with("/hello", |_route| {
//!         "route found"
//!     }) {
//!         println!("{}", result);
//!     }
//! }
//! ```
//!
//! # 模块结构
//!
//! - [`core`][]: 核心数据结构和抽象
//! - [`storage`][]: 持久化存储接口和实现
//! - [`actix`][]: Actix-Web 集成
//!
//! [`core`]: core/index.html
//! [`storage`]: storage/index.html
//! [`actix`]: actix/index.html

pub mod actix;
pub mod core;
pub mod storage;

pub use core::{RouteEntry, RouteTable, SerializableRoute, SimpleRoute};
pub use core::cache_optimized::CacheOptimizedRouteTable;
pub use storage::{FileStorage, KeyValueStorage, MemoryStorage, RouteStorage};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
