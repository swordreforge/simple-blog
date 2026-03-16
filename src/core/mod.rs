pub mod cache;
pub mod cache_optimized;
pub mod object_pool;
pub mod route_entry;
pub mod route_matcher;
pub mod route_radix_tree;
pub mod route_registry;
pub mod route_table;
pub mod route_trie;
pub mod route_validator;
pub mod simple_route;

pub use cache::{BatchOperations, PerformanceOptions, RouteCache};
pub use cache_optimized::{CacheOptimizedShard, CompactRadixTree};
pub use object_pool::{PoolConfig, RouteObjectPool};
pub use route_entry::{RouteEntry, SerializableRoute};
pub use route_matcher::{MatchResult, RouteMatcher, RoutePattern};
pub use route_radix_tree::RouteRadixTree;
pub use route_registry::{RouteFactory, RouteRegistry, RouteRegistryError};
pub use route_table::RouteTable;
pub use route_validator::{
    MigrationRule, RouteTypeMetadata, RouteTypeRegistry, RouteValidator, ValidationError,
};
pub use simple_route::SimpleRoute;
