-- 动态路由表
CREATE TABLE IF NOT EXISTS dynamic_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_type TEXT NOT NULL CHECK(route_type IN ('memory', 'file', 'database')),
    path TEXT NOT NULL UNIQUE,
    handler_type TEXT NOT NULL CHECK(handler_type IN ('redirect', 'static', 'template', 'proxy', 'custom')),
    handler_config TEXT NOT NULL,  -- JSON配置
    enabled BOOLEAN DEFAULT 1,
    priority INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    metadata TEXT  -- JSON扩展字段
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_path ON dynamic_routes(path);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_type ON dynamic_routes(route_type);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_enabled ON dynamic_routes(enabled);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_priority ON dynamic_routes(priority DESC);

-- 创建触发器：自动更新时间戳
CREATE TRIGGER IF NOT EXISTS update_dynamic_routes_timestamp
AFTER UPDATE ON dynamic_routes
FOR EACH ROW
BEGIN
    UPDATE dynamic_routes SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- 动态路由操作日志表
CREATE TABLE IF NOT EXISTS dynamic_route_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER,
    action TEXT NOT NULL,  -- 'create', 'update', 'delete', 'enable', 'disable'
    old_config TEXT,
    new_config TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_route_id ON dynamic_route_logs(route_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_action ON dynamic_route_logs(action);
CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_created_at ON dynamic_route_logs(created_at DESC);

-- 动态路由统计表
CREATE TABLE IF NOT EXISTS dynamic_route_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0,
    last_accessed_at TEXT,
    total_response_time_ms INTEGER DEFAULT 0,
    avg_response_time_ms REAL DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE,
    UNIQUE(route_id)
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_stats_route_id ON dynamic_route_stats(route_id);