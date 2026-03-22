pub mod arc_route_entry;
pub mod bytes_optimized;
pub mod cache;
pub mod cache_optimized;
pub mod cow_optimized;
pub mod dynamic_route_table;
pub mod dynamic_sharding;
pub mod fast_hashmap;
pub mod lockfree_cache;
pub mod lockfree_shard;
pub mod object_pool;
pub mod papaya_route_table;
pub mod route_entry;
pub mod route_matcher;
pub mod route_radix_tree;
pub mod route_registry;
pub mod route_table;
pub mod route_trie;
pub mod route_validator;
pub mod simple_route;
pub mod simd_optimized;
pub mod string_optimized;
pub mod phf_static_routes;

pub use arc_route_entry::ArcRouteEntry;
pub use cache::{BatchOperations, PerformanceOptions, RouteCache};
pub use cache_optimized::{CacheOptimizedShard, CompactRadixTree};
pub use dynamic_route_table::{DynamicRouteTable, DynamicRouteTableConfig};
pub use dynamic_sharding::{
    DynamicShard, DynamicShardManager, DynamicShardingConfig, LoadBalanceStrategy, ShardMetrics,
};
pub use lockfree_cache::{CacheStatsSnapshot, LockfreeCacheStats, LockfreeLruCache};
pub use lockfree_shard::{LockfreeDynamicShard, LockfreeShardMetrics, ShardMetricsSnapshot};
pub use simd_optimized::{SimdComparator, SimdPathSplitter};
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
pub use string_optimized::{
    extract_params_pooled, global_path_pool, global_stats, join_paths_optimized, PathStringPool,
    SmallString, SmartString, split_path_pooled, split_path_small, split_path_smart,
    StringOptimizationStats, StringPool,
};
pub use cow_optimized::{
    CowRoutePattern, join_cow, normalize_path, OptimizedMatchResult, OptimizedStr, ParamExtractor,
    PathMatchCache, StringFragment, StringFragmentBuilder,
};
pub use bytes_optimized::{
    BytesBuilder, BytesComparator, BytesConverter, BytesPool, BytesSplitter, BytesView,
    OptimizedBytes,
};
#[cfg(feature = "papaya")]
pub use papaya_route_table::PapayaRouteTable;
pub use phf_static_routes::{HybridRouteTable, StaticRouteRegistry};
// static_routes宏被导出到crate根目录
pub use fast_hashmap::{FastHashMap, FastHashSet};
