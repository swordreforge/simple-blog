//! 路由类型验证器
//!
//! 提供路由类型验证和迁移机制。

use crate::core::route_registry::RouteRegistry;
use crate::core::SerializableRoute;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// 验证错误类型
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Unknown route type: {0}")]
    UnknownRouteType(String),

    #[error("Invalid route path: {0}")]
    InvalidPath(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid content type: {0}")]
    InvalidContentType(String),

    #[error("Extra data validation failed: {0}")]
    ExtraDataValidationError(String),

    #[error("Route version incompatible: expected {expected}, got {actual}")]
    VersionIncompatible { expected: String, actual: String },
}

/// 路由类型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTypeMetadata {
    pub type_name: String,
    pub version: String,
    pub description: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub supported_content_types: Vec<String>,
    pub requires_extra_data: bool,
}

/// 路由迁移规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRule {
    pub from_version: String,
    pub to_version: String,
    pub description: String,
    pub transformation: String, // 在实际实现中，可以使用函数指针或闭包
}

/// 路由类型注册表元数据
pub struct RouteTypeRegistry {
    metadata: HashMap<String, RouteTypeMetadata>,
    migrations: HashMap<String, Vec<MigrationRule>>,
}

impl RouteTypeRegistry {
    /// 创建新的路由类型注册表
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            migrations: HashMap::new(),
        }
    }

    /// 注册路由类型元数据
    pub fn register_metadata(&mut self, metadata: RouteTypeMetadata) -> Result<(), ValidationError> {
        // 验证元数据
        if metadata.type_name.is_empty() {
            return Err(ValidationError::MissingField("type_name".to_string()));
        }

        if metadata.required_fields.is_empty() {
            return Err(ValidationError::MissingField("required_fields".to_string()));
        }

        self.metadata.insert(metadata.type_name.clone(), metadata);
        Ok(())
    }

    /// 获取路由类型元数据
    pub fn get_metadata(&self, type_name: &str) -> Option<&RouteTypeMetadata> {
        self.metadata.get(type_name)
    }

    /// 注册迁移规则
    pub fn register_migration(&mut self, type_name: &str, rule: MigrationRule) {
        self.migrations
            .entry(type_name.to_string())
            .or_insert_with(Vec::new)
            .push(rule);
    }

    /// 获取迁移规则
    pub fn get_migrations(&self, type_name: &str) -> &[MigrationRule] {
        self.migrations.get(type_name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 检查是否支持路由类型
    pub fn is_type_registered(&self, type_name: &str) -> bool {
        self.metadata.contains_key(type_name)
    }
}

impl Default for RouteTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由验证器
pub struct RouteValidator {
    metadata_registry: RouteTypeRegistry,
}

impl RouteValidator {
    /// 创建新的路由验证器
    pub fn new() -> Self {
        Self {
            metadata_registry: RouteTypeRegistry::new(),
        }
    }

    /// 创建带有默认元数据的验证器
    pub fn with_defaults() -> Self {
        let mut validator = Self::new();

        // 注册 SimpleRoute 的元数据
        validator.metadata_registry.register_metadata(RouteTypeMetadata {
            type_name: "SimpleRoute".to_string(),
            version: "1.0.0".to_string(),
            description: "Simple route with basic response".to_string(),
            required_fields: vec!["route_type".to_string(), "body".to_string(), "content_type".to_string()],
            optional_fields: vec!["extra_data".to_string()],
            supported_content_types: vec![
                "text/plain".to_string(),
                "text/html".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
            ],
            requires_extra_data: false,
        }).unwrap();

        validator
    }

    /// 注册路由类型元数据
    pub fn register_type_metadata(&mut self, metadata: RouteTypeMetadata) -> Result<(), ValidationError> {
        self.metadata_registry.register_metadata(metadata)
    }

    /// 验证路由路径
    fn validate_path(&self, path: &str) -> Result<(), ValidationError> {
        if path.is_empty() {
            return Err(ValidationError::InvalidPath("Path cannot be empty".to_string()));
        }

        // 检查路径是否以 / 开头
        if !path.starts_with('/') {
            return Err(ValidationError::InvalidPath(
                "Path must start with '/'".to_string(),
            ));
        }

        // 检查路径是否包含非法字符
        if path.contains("..") {
            return Err(ValidationError::InvalidPath(
                "Path cannot contain '..'".to_string(),
            ));
        }

        Ok(())
    }

    /// 验证内容类型
    fn validate_content_type(&self, content_type: &str, metadata: &RouteTypeMetadata) -> Result<(), ValidationError> {
        if metadata.supported_content_types.is_empty() {
            // 如果没有指定支持的内容类型，则允许所有类型
            return Ok(());
        }

        if !metadata.supported_content_types.contains(&content_type.to_string()) {
            return Err(ValidationError::InvalidContentType(format!(
                "Content type '{}' is not supported. Supported types: {:?}",
                content_type, metadata.supported_content_types
            )));
        }

        Ok(())
    }

    /// 验证额外数据
    fn validate_extra_data(&self, extra_data: &Option<String>, metadata: &RouteTypeMetadata) -> Result<(), ValidationError> {
        if metadata.requires_extra_data && extra_data.is_none() {
            return Err(ValidationError::ExtraDataValidationError(
                "This route type requires extra_data".to_string(),
            ));
        }

        // 如果存在额外数据，验证是否为有效的 JSON
        if let Some(ref data) = extra_data {
            serde_json::from_str::<serde_json::Value>(data)
                .map_err(|e| ValidationError::ExtraDataValidationError(format!("Invalid JSON: {}", e)))?;
        }

        Ok(())
    }

    /// 验证路由
    pub fn validate_route(&self, path: &str, route: &SerializableRoute) -> Result<(), ValidationError> {
        // 验证路径
        self.validate_path(path)?;

        // 检查路由类型是否已注册
        if !RouteRegistry::list_types().contains(&route.route_type) {
            return Err(ValidationError::UnknownRouteType(route.route_type.clone()));
        }

        // 获取路由类型元数据
        let metadata = self
            .metadata_registry
            .get_metadata(&route.route_type)
            .ok_or_else(|| ValidationError::UnknownRouteType(route.route_type.clone()))?;

        // 验证内容类型
        self.validate_content_type(&route.content_type, metadata)?;

        // 验证额外数据
        self.validate_extra_data(&route.extra_data, metadata)?;

        Ok(())
    }

    /// 验证路由集合
    pub fn validate_routes(&self, routes: &HashMap<String, SerializableRoute>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (path, route) in routes {
            if let Err(e) = self.validate_route(path, route) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 检查是否需要迁移
    pub fn needs_migration(&self, type_name: &str, current_version: &str) -> bool {
        let migrations = self.metadata_registry.get_migrations(type_name);
        migrations.iter().any(|m| m.from_version == current_version)
    }

    /// 获取可用的迁移规则
    pub fn get_available_migrations(&self, type_name: &str, from_version: &str) -> Vec<&MigrationRule> {
        self.metadata_registry
            .get_migrations(type_name)
            .iter()
            .filter(|m| m.from_version == from_version)
            .collect()
    }
}

impl Default for RouteValidator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_valid() {
        let validator = RouteValidator::new();
        assert!(validator.validate_path("/test").is_ok());
        assert!(validator.validate_path("/api/v1/users").is_ok());
        assert!(validator.validate_path("/path/with spaces").is_ok());
    }

    #[test]
    fn test_validate_path_invalid() {
        let validator = RouteValidator::new();
        assert!(validator.validate_path("").is_err());
        assert!(validator.validate_path("test").is_err()); // Missing leading /
        assert!(validator.validate_path("/../test").is_err()); // Contains ..
    }

    #[test]
    fn test_route_type_registry() {
        let mut registry = RouteTypeRegistry::new();

        let metadata = RouteTypeMetadata {
            type_name: "TestRoute".to_string(),
            version: "1.0.0".to_string(),
            description: "Test route".to_string(),
            required_fields: vec!["route_type".to_string(), "body".to_string()],
            optional_fields: vec![],
            supported_content_types: vec!["text/plain".to_string()],
            requires_extra_data: false,
        };

        assert!(registry.register_metadata(metadata).is_ok());
        assert!(registry.is_type_registered("TestRoute"));

        let retrieved = registry.get_metadata("TestRoute");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().type_name, "TestRoute");
    }

    #[test]
    fn test_route_validation() {
        let validator = RouteValidator::with_defaults();

        let route = SerializableRoute {
            route_type: "SimpleRoute".to_string(),
            body: "test body".to_string(),
            content_type: "text/plain".to_string(),
            extra_data: None,
        };

        // 有效路由
        assert!(validator.validate_route("/test", &route).is_ok());

        // 无效路径
        assert!(validator.validate_route("test", &route).is_err());

        // 无效内容类型
        let invalid_route = SerializableRoute {
            content_type: "invalid/type".to_string(),
            ..route.clone()
        };
        assert!(validator.validate_route("/test", &invalid_route).is_err());

        // 未知路由类型
        let unknown_route = SerializableRoute {
            route_type: "UnknownRoute".to_string(),
            ..route
        };
        assert!(validator.validate_route("/test", &unknown_route).is_err());
    }

    #[test]
    fn test_route_collection_validation() {
        let validator = RouteValidator::with_defaults();

        let mut routes = HashMap::new();
        routes.insert(
            "/valid1".to_string(),
            SerializableRoute {
                route_type: "SimpleRoute".to_string(),
                body: "body1".to_string(),
                content_type: "text/plain".to_string(),
                extra_data: None,
            },
        );
        routes.insert(
            "/valid2".to_string(),
            SerializableRoute {
                route_type: "SimpleRoute".to_string(),
                body: "body2".to_string(),
                content_type: "application/json".to_string(),
                extra_data: None,
            },
        );

        // 所有路由都有效
        assert!(validator.validate_routes(&routes).is_ok());

        // 添加无效路由
        routes.insert(
            "invalid".to_string(), // 缺少前导 /
            SerializableRoute {
                route_type: "SimpleRoute".to_string(),
                body: "body3".to_string(),
                content_type: "text/plain".to_string(),
                extra_data: None,
            },
        );

        let result = validator.validate_routes(&routes);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_extra_data_validation() {
        let validator = RouteValidator::with_defaults();

        // 有效的 JSON 额外数据
        let route = SerializableRoute {
            route_type: "SimpleRoute".to_string(),
            body: "body".to_string(),
            content_type: "text/plain".to_string(),
            extra_data: Some(r#"{"key":"value"}"#.to_string()),
        };
        assert!(validator.validate_route("/test", &route).is_ok());

        // 无效的 JSON 额外数据
        let invalid_route = SerializableRoute {
            extra_data: Some("{invalid json}".to_string()),
            ..route
        };
        assert!(validator.validate_route("/test", &invalid_route).is_err());
    }

    #[test]
    fn test_migration_rules() {
        let mut registry = RouteTypeRegistry::new();

        let rule = MigrationRule {
            from_version: "1.0.0".to_string(),
            to_version: "2.0.0".to_string(),
            description: "Upgrade to version 2.0".to_string(),
            transformation: "transformation_function".to_string(),
        };

        registry.register_migration("TestRoute", rule);

        assert_eq!(registry.get_migrations("TestRoute").len(), 1);
        assert!(registry.get_migrations("UnknownRoute").is_empty());
    }
}