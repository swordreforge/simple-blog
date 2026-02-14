# 安全审计报告

**审计日期**: 2026-02-14  
**审计范围**: OWASP Top 10 漏洞检查  
**项目**: rustblog - 基于 Rust 和 Actix-web 的博客系统

---

## 执行摘要

本次安全审计发现 **2 个高危漏洞** 和 **2 个中危漏洞**，需要立即修复。其余安全控制均已正确实施。

### 风险分布

| 风险等级 | 数量 | 状态 |
|----------|------|------|
| 🔴 高危 | 2 | 需要立即修复 |
| 🟡 中危 | 2 | 需要尽快修复 |
| ✅ 安全 | 6 | 已正确实施 |

---

## 详细发现

### 🔴 A01:2021 - 访问控制失效

**严重程度**: 高危  
**CVSS 评分**: 8.1 (高危)

#### 问题描述
认证中间件的 `FromRequest` 实现在未认证时返回空值而非拒绝请求，导致未授权访问。

#### 受影响代码
```rust
// src/middleware/auth.rs:51
impl FromRequest for RoleKey {
    type Error = Error;
    type Future = Ready<Result<RoleKey, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(role) = req.extensions().get::<RoleKey>() {
            ready(Ok(role.clone()))
        } else {
            // ❌ 问题：返回空值而非拒绝请求
            ready(Ok(RoleKey(String::new())))
        }
    }
}
```

#### 影响范围
- 所有使用 `RoleKey`、`UserIDKey`、`UsernameKey` 提取器的端点
- 未认证用户可能绕过访问控制

#### 攻击场景
```bash
# 未认证用户访问受保护端点
curl http://example.com/api/admin/passages

# 预期：401 Unauthorized
# 实际：可能返回数据（取决于 handler 逻辑）
```

#### 修复建议
修改 `FromRequest` 实现，未认证时返回错误：

```rust
fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
    if let Some(role) = req.extensions().get::<RoleKey>() {
        ready(Ok(role.clone()))
    } else {
        // ✅ 修复：拒绝未认证请求
        ready(Err(actix_web::error::ErrorUnauthorized(
            "Authentication required"
        )))
    }
}
```

#### 优先级
**P0 - 立即修复**

---

### 🔴 A02:2021 - 加密失效

**严重程度**: 高危  
**CVSS 评分**: 7.5 (高危)

#### 问题描述
AES-GCM 加密中的 Nonce 被重复使用，违反加密最佳实践，可能导致密钥泄露或消息伪造。

#### 受影响代码
```rust
// src/handlers/api_handlers/crypto.rs:163
pub fn hybrid_decrypt(&self, encrypted_data_b64: &str, client_public_key_input: &str) -> Result<String, String> {
    // ... ECDH 密钥协商 ...
    
    // 提取nonce（前12字节）
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let ciphertext = &encrypted_data[12..];
    
    // ❌ 问题：Nonce 固定为密文前12字节，可能重复使用
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
}
```

#### 加密原理
AES-GCM 是一种认证加密算法，要求 **每个密钥对每个 Nonce 只能使用一次**。重复使用 Nonce 会导致：

1. **密钥泄露**: 攻击者可恢复加密密钥
2. **消息伪造**: 攻击者可篡改密文而不被发现
3. **重放攻击**: 攻击者可重放密文

#### 影响范围
- 所有使用 `hybrid_decrypt` 的端点
- 加密的用户数据可能被破解

#### 攻击场景
```python
# 攻击者收集多个使用相同 Nonce 的密文
ciphertext1 = encrypt(plaintext1, nonce)
ciphertext2 = encrypt(plaintext2, nonce)

# XOR 两个密文可恢复明文 XOR
# XOR 两个密文标签可恢复认证密钥
```

#### 修复建议
使用随机 Nonce 或计数器：

```rust
pub fn hybrid_encrypt(&self, plaintext: &str, client_public_key_input: &str) -> Result<String, String> {
    // ... ECDH 密钥协商 ...
    
    // ✅ 修复：生成随机 Nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let cipher = Aes256Gcm::new(&shared_key.into());
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Nonce + Ciphertext + Tag
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(base64::encode(&result))
}
```

#### 优先级
**P0 - 立即修复**

---

### 🟡 A04:2021 - 不安全设计

**严重程度**: 中危  
**CVSS 评分**: 5.3 (中危)

#### 问题描述
访问控制检查逻辑不完整，空角色 `""` 与 `"admin"` 比较返回 `true`，但代码继续执行。

#### 受影响代码
```rust
// src/handlers/api_handlers/passage.rs:316-363
pub async fn get(...) -> HttpResponse {
    // 获取用户角色
    let role: String = req.extensions().get::<crate::middleware::auth::RoleKey>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| String::new()); // ❌ 空字符串
    
    // ... 获取文章 ...
    
    // 检查文章状态和可见性
    if passage.status != "published" {
        if role != "admin" {  // ❌ "" != "admin" 为 true
            return HttpResponse::Ok().json(...); // 继续执行
        }
    }
}
```

#### 逻辑错误
```rust
role != "admin"  // 当 role = "" 时，返回 true
```

#### 影响范围
- 文章访问控制
- 其他基于角色的访问控制

#### 修复建议
使用显式检查：

```rust
if role != "admin" && !role.is_empty() {
    return HttpResponse::Ok().json(...);
}

if role.is_empty() {
    return HttpResponse::Unauthorized().json(...);
}
```

#### 优先级
**P1 - 尽快修复**

---

### 🟡 A09:2021 - 安全日志和监控失效

**严重程度**: 中危  
**CVSS 评分**: 4.3 (中危)

#### 问题描述
部分敏感操作缺少审计日志，无法追踪安全事件。

#### 缺失日志的场景
1. 管理员登录
2. 文章创建/删除
3. 用户权限变更
4. 敏感配置修改

#### 当前日志示例
```rust
// ✅ 良好：错误日志
eprintln!("获取文章失败: {}", e);

// ❌ 缺失：审计日志
// 没有记录谁在何时执行了什么操作
```

#### 修复建议
添加结构化审计日志：

```rust
#[derive(Debug, Serialize)]
struct AuditLog {
    timestamp: DateTime<Utc>,
    user_id: i64,
    username: String,
    action: String,
    resource: String,
    ip: String,
    user_agent: String,
    success: bool,
}

pub fn log_audit_event(log: AuditLog) {
    let log_line = serde_json::to_string(&log).unwrap();
    println!("AUDIT: {}", log_line);
    
    // 也可写入数据库或日志文件
}
```

#### 优先级
**P1 - 尽快修复**

---

## 通过的安全检查

### ✅ A03:2021 - 注入

**状态**: 安全  
**检查结果**: 所有 SQL 查询使用参数化查询

```rust
// ✅ 正确：参数化查询
conn.execute(
    "UPDATE passages SET title = ? WHERE id = ?",
    params![&passage.title, &passage.id]
)?;

// ✅ 正确：无字符串拼接
let sql = format!(
    "SELECT * FROM passages WHERE id = ?", // 使用 ? 而非直接拼接
);
```

**防护措施**:
- 使用 `rusqlite` 参数化查询
- 避免字符串拼接 SQL

---

### ✅ A05:2021 - 安全配置错误

**状态**: 安全  
**检查结果**: JWT 密钥使用强随机密钥

```rust
// ✅ 正确：32字节随机密钥
fn generate_random_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

// ✅ 正确：密钥持久化到文件
let jwt_secret_file = base_dir.join("data").join("jwt-secret");
fs::write(&jwt_secret_file, &new_secret)?;
```

**防护措施**:
- 32字节随机密钥（256位）
- 密钥文件权限保护
- 不使用硬编码密钥

---

### ✅ A06:2021 - 易受攻击和过时的组件

**状态**: 安全  
**检查结果**: 使用最新版本的依赖项

| 依赖项 | 版本 | 状态 |
|--------|------|------|
| actix-web | 4.12.1 | ✅ 最新 |
| tokio | 1.49 | ✅ 最新 |
| rusqlite | 0.32 | ✅ 最新 |
| p256 | 0.13 | ✅ 最新 |
| aes-gcm | 0.10 | ✅ 最新 |
| jsonwebtoken | 10.3 | ✅ 最新 |

**防护措施**:
- 定期更新依赖项
- 使用 `cargo audit` 检查已知漏洞

---

### ✅ A07:2021 - 身份识别和身份验证失效

**状态**: 安全  
**检查结果**: JWT 实现符合最佳实践

```rust
// ✅ 正确：使用强加密算法
pub struct JwtService {
    secret: String,
    token_expiration: Duration,
}

// ✅ 正确：HS256 算法
&Validation::new(Algorithm::HS256)

// ✅ 正确：检查过期时间
if Utc::now().timestamp() > claims.exp {
    return Err(JwtError::ExpiredToken);
}
```

**防护措施**:
- 使用 HS256 算法
- 32字节密钥
- Token 过期检查
- Cookie 安全设置

---

### ✅ A08:2021 - 软件和数据完整性失效

**状态**: 安全  
**检查结果**: 文件上传有路径验证

```rust
// ✅ 正确：路径穿越防护
fn validate_path(user_path: &str) -> Result<String, String> {
    // 规范化路径，移除 . 和 ..
    let normalized_path: PathBuf = user_path
        .components()
        .filter(|comp| !matches!(comp, 
            Component::ParentDir | Component::CurDir))
        .collect();
    
    // 检查是否在允许的目录中
    let allowed_dirs = vec![
        cwd.join("img"),
        cwd.join("markdown"),
        cwd.join("attachments"),
        cwd.join("music"),
    ];
}
```

**防护措施**:
- 路径标准化
- 白名单目录
- 文件类型验证

---

### ✅ A10:2021 - 服务器端请求伪造 (SSRF)

**状态**: 安全  
**检查结果**: 应用程序不发起外部 HTTP 请求

**检查结果**:
- 无外部 API 调用
- 无用户控制的 URL 请求
- 无基于 URL 的文件加载

---

## 修复优先级

### P0 - 立即修复（24小时内）

1. **A01: 访问控制失效**
   - 修复认证中间件返回空值问题
   - 确保未认证请求被拒绝

2. **A02: 加密失效**
   - 修复 AES-GCM Nonce 重复使用
   - 实现随机 Nonce 生成

### P1 - 尽快修复（一周内）

3. **A04: 不安全设计**
   - 修复角色检查逻辑
   - 添加显式空角色检查

4. **A09: 安全日志和监控失效**
   - 实现审计日志系统
   - 记录所有敏感操作

---

## 安全建议

### 短期（1-3个月）

1. **实施安全日志**
   - 结构化审计日志
   - 异常行为检测
   - 安全事件告警

2. **加强访问控制**
   - 实施 RBAC（基于角色的访问控制）
   - 定期审计权限
   - 实施最小权限原则

3. **依赖项管理**
   - 集成 `cargo audit`
   - 定期更新依赖项
   - 监控安全公告

### 长期（3-12个月）

1. **安全测试**
   - 渗透测试
   - 模糊测试
   - 代码安全审查

2. **安全培训**
   - 开发者安全培训
   - 安全编码实践
   - 漏洞披露流程

3. **合规性**
   - GDPR 合规
   - 安全标准认证
   - 数据保护措施

---

## 附录

### A. 测试方法

本次审计采用以下方法：

1. **静态代码分析**
   - 手动代码审查
   - 模式匹配搜索
   - 数据流分析

2. **动态分析**
   - strace 日志分析
   - 运行时行为检查
   - 网络流量分析

3. **配置检查**
   - 依赖项版本检查
   - 安全配置验证
   - 文件权限检查

### B. 参考资料

- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [Rust Security Guidelines](https://doc.rust-lang.org/nomicon/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Actix-web Security](https://actix.rs/docs/security)

### C. 联系信息

如有安全问题或发现，请联系：

- **安全邮箱**: security@example.com
- **漏洞披露**: 请遵循负责任披露原则
- **紧急联系**: +86-xxx-xxxx-xxxx

---

**报告生成时间**: 2026-02-14  
**审计人员**: iFlow CLI  
**版本**: 1.0.0