use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub details: Option<serde_json::Value>,
}

/// 审计日志记录器
#[derive(Clone)]
pub struct AuditLogger {
    log_to_file: bool,
    log_to_stdout: bool,
}

impl AuditLogger {
    pub fn new() -> Self {
        AuditLogger {
            log_to_file: true,
            log_to_stdout: true,
        }
    }

    /// 记录审计事件
    pub fn log(&self, log: AuditLog) {
        let log_line = match serde_json::to_string(&log) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize audit log: {}", e);
                return;
            }
        };

        if self.log_to_stdout {
            println!("AUDIT: {}", log_line);
        }

        if self.log_to_file
            && let Err(e) = self.write_to_file(&log_line) {
                eprintln!("Failed to write audit log to file: {}", e);
            }
    }

    /// 写入日志文件
    fn write_to_file(&self, log_line: &str) -> std::io::Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let log_dir = "data/logs";
        std::fs::create_dir_all(log_dir)?;

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let log_file = format!("{}/audit-{}.log", log_dir, date);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        writeln!(file, "{}", log_line)
    }

    /// 记录登录事件
    #[allow(dead_code)]
    pub fn log_login(
        &self,
        user_id: i64,
        username: &str,
        ip: Option<String>,
        user_agent: Option<String>,
        success: bool,
    ) {
        self.log(AuditLog {
            timestamp: Utc::now(),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            action: "LOGIN".to_string(),
            resource: "AUTH".to_string(),
            resource_id: None,
            ip,
            user_agent,
            success,
            details: None,
        });
    }

    /// 记录文章创建事件
    pub fn log_passage_create(
        &self,
        user_id: i64,
        username: &str,
        passage_id: i64,
        passage_uuid: &str,
        title: &str,
        ip: Option<String>,
    ) {
        self.log(AuditLog {
            timestamp: Utc::now(),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            action: "CREATE".to_string(),
            resource: "PASSAGE".to_string(),
            resource_id: Some(passage_uuid.to_string()),
            ip,
            user_agent: None,
            success: true,
            details: Some(serde_json::json!({
                "passage_id": passage_id,
                "title": title
            })),
        });
    }

    /// 记录文章更新事件
    pub fn log_passage_update(
        &self,
        user_id: i64,
        username: &str,
        passage_id: i64,
        passage_uuid: &str,
        ip: Option<String>,
    ) {
        self.log(AuditLog {
            timestamp: Utc::now(),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            action: "UPDATE".to_string(),
            resource: "PASSAGE".to_string(),
            resource_id: Some(passage_uuid.to_string()),
            ip,
            user_agent: None,
            success: true,
            details: Some(serde_json::json!({
                "passage_id": passage_id
            })),
        });
    }

    /// 记录文章删除事件
    pub fn log_passage_delete(
        &self,
        user_id: i64,
        username: &str,
        passage_uuid: &str,
        title: &str,
        ip: Option<String>,
    ) {
        self.log(AuditLog {
            timestamp: Utc::now(),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            action: "DELETE".to_string(),
            resource: "PASSAGE".to_string(),
            resource_id: Some(passage_uuid.to_string()),
            ip,
            user_agent: None,
            success: true,
            details: Some(serde_json::json!({
                "title": title
            })),
        });
    }

    /// 记录权限变更事件
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn log_permission_change(
        &self,
        admin_user_id: i64,
        admin_username: &str,
        target_user_id: i64,
        target_username: &str,
        old_role: &str,
        new_role: &str,
        ip: Option<String>,
    ) {
        self.log(AuditLog {
            timestamp: Utc::now(),
            user_id: Some(admin_user_id),
            username: Some(admin_username.to_string()),
            action: "PERMISSION_CHANGE".to_string(),
            resource: "USER".to_string(),
            resource_id: Some(target_user_id.to_string()),
            ip,
            user_agent: None,
            success: true,
            details: Some(serde_json::json!({
                "target_username": target_username,
                "old_role": old_role,
                "new_role": new_role
            })),
        });
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref AUDIT_LOGGER: AuditLogger = AuditLogger::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog {
            timestamp: Utc::now(),
            user_id: Some(1),
            username: Some("test_user".to_string()),
            action: "LOGIN".to_string(),
            resource: "AUTH".to_string(),
            resource_id: None,
            ip: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            success: true,
            details: None,
        };

        assert_eq!(log.action, "LOGIN");
        assert_eq!(log.user_id, Some(1));
        assert_eq!(log.success, true);
    }

    #[test]
    fn test_audit_log_serialization() {
        let log = AuditLog {
            timestamp: Utc::now(),
            user_id: Some(1),
            username: Some("test_user".to_string()),
            action: "CREATE".to_string(),
            resource: "PASSAGE".to_string(),
            resource_id: Some("test-uuid".to_string()),
            ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            success: true,
            details: Some(serde_json::json!({"title": "Test"})),
        };

        // 测试序列化
        let json = serde_json::to_string(&log);
        assert!(json.is_ok());

        // 测试反序列化
        let deserialized: Result<AuditLog, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_audit_logger_default() {
        let _logger = AuditLogger::default();
        // 测试 logger 可以创建
        // 实际的日志写入测试需要文件系统
    }
}
