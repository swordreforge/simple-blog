/// 预定义静态路由列表
///
/// 这些路由在应用启动时通过 Actix-Web 配置，具有更高的优先级。
/// 动态路由不能与这些路径冲突。
use std::collections::HashSet;

/// 获取所有预定义静态路由路径
pub fn get_static_routes() -> HashSet<String> {
    let mut routes = HashSet::new();

    // 页面路由
    routes.insert("/".to_string());
    routes.insert("/index".to_string());
    routes.insert("/passage".to_string());
    routes.insert("/collect".to_string());
    routes.insert("/about".to_string());
    routes.insert("/friends".to_string());
    routes.insert("/markdown-editor".to_string());
    routes.insert("/keyboard-test".to_string());
    routes.insert("/admin".to_string());
    routes.insert("/admin/dyn-routing".to_string());
    routes.insert("/health".to_string());

    // 静态文件路由（前缀）
    routes.insert("/favicon.ico".to_string());
    routes.insert("/css".to_string());
    routes.insert("/js".to_string());
    routes.insert("/img".to_string());
    routes.insert("/music".to_string());
    routes.insert("/attachments".to_string());
    routes.insert("/markdown".to_string());

    // API 路由（前缀）
    routes.insert("/api".to_string());

    // 状态页面路由（前缀）
    routes.insert("/status".to_string());

    routes
}

/// 检查路径是否与静态路由冲突
pub fn conflicts_with_static_route(path: &str) -> bool {
    let static_routes = get_static_routes();

    // 精确匹配
    if static_routes.contains(path) {
        return true;
    }

    // 检查前缀冲突（静态路由作为前缀）
    for static_route in &static_routes {
        // 如果静态路由是路径的前缀，或者路径是静态路由的前缀
        if path.starts_with(&format!("{}/", static_route))
            || static_route.starts_with(&format!("{}/", path))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(conflicts_with_static_route("/"));
        assert!(conflicts_with_static_route("/admin"));
        assert!(conflicts_with_static_route("/api"));
        assert!(conflicts_with_static_route("/favicon.ico"));
    }

    #[test]
    fn test_prefix_match() {
        assert!(conflicts_with_static_route("/api/users"));
        assert!(conflicts_with_static_route("/api/settings"));
        assert!(conflicts_with_static_route("/css/style.css"));
        assert!(conflicts_with_static_route("/img/avatar.png"));
        assert!(conflicts_with_static_route("/admin/users"));
    }

    #[test]
    fn test_no_conflict() {
        assert!(!conflicts_with_static_route("/custom-path"));
        assert!(!conflicts_with_static_route("/my-custom-route"));
        assert!(!conflicts_with_static_route("/external-api"));
    }
}
