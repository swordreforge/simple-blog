### 项目修改计划

#### 1.支持批量增删改查

#### 2.边界情况保证

#### 2.1关于界面预先配置模板

#### 3.添加文章封面用户可选择是否开启，在归档中展示

#### 4.前端性能优化

#### 5.总体性能优化

#### 6.限流中间件实现

#### 7.类似embed的实现



ECC加密登录功能修复总结

  问题描述
  登录和注册功能的ECC加密解密一直失败，返回"密码解密失败: Decryption
  failed: aead::Error"错误。

  根本原因分析

  问题源于前后端ECDH密钥派生方式不一致：

   1. 公钥解析问题：
      - 前端发送PEM格式的公钥（PKIX/DER编码）
      - 后端最初只支持SEC1格式解析，导致解析失败

   2. 密钥派生方式差异：
      - 前端Web Crypto API使用deriveBits返回压缩格式（32字节X坐标）
      - 后端Rust使用to_bytes()返回SEC1未压缩格式（65字节：0x04 + X + Y）
      - 导致共享密钥不匹配，AES-GCM解密失败

  修复步骤

  1. 后端修复

  文件: src/handlers/api_handlers/crypto.rs

  修复1: 添加PKIX格式支持
   # Cargo.toml
   spki = "0.7"  # 用于解析PKIX格式公钥

   use p256::elliptic_curve::sec1::ToEncodedPoint;
   use spki::DecodePublicKey;

   // 支持PKIX/DER和SEC1两种格式
   let client_public_key = match PublicKey::from_public_key_der
   (client_public_key_bytes) {
       Ok(key) => key,
       Err(_) => PublicKey::from_sec1_bytes
   (client_public_key_bytes)?
   };

  修复2: 正确提取X坐标
   // 计算共享密钥
   let shared_point = *client_point * *server_scalar.as_ref();

   // 获取未压缩格式的点
   let encoded_point = shared_point.to_encoded_point(false);
   let point_bytes = encoded_point.as_bytes();

   // 提取X坐标（跳过0x04字节）
   let x_coordinate = &point_bytes[1..33];
   let mut key = [0u8; 32];
   key.copy_from_slice(x_coordinate);

  2. 前端修复

  文件: templates/js/ecc-encrypt.js

  修复: 直接使用deriveBits返回的32字节作为AES密钥
   async deriveSharedKey() {
     // 使用deriveBits获取32字节的X坐标
     const sharedSecretBits = await window.crypto.subtle.deriveBits
   (
       { name: 'ECDH', public: this.serverPublicKey },
       this.clientKeyPair.privateKey,
       256  // 返回32字节
     );

     const sharedSecretBytes = new Uint8Array(sharedSecretBits);
    
     // 直接使用全部32字节作为AES-GCM密钥
     return await window.crypto.subtle.importKey(
       'raw',
       sharedSecretBytes,
       { name: 'AES-GCM', length: 256 },
       true,
       ['encrypt', 'decrypt']
     );
   }

  技术要点

   1. ECDH密钥派生一致性：
      - Go版本: sharedX.Bytes() → 32字节X坐标
      - Rust版本（修复后）: point_bytes[1..33] → 32字节X坐标
      - 前端（修复后）: deriveBits(..., 256) → 32字节X坐标

   2. 加密数据格式：
      - 格式：nonce(12字节) + ciphertext + auth_tag(16字节)
      - 示例：8字节明文 → 24字节密文(8+16) → 36字节组合 → 48字节base64

   3. AES-GCM解密流程：
      - Base64解码 → 分离nonce和密文 → AES-GCM解密 → UTF-8解码

  验证结果

  修复后，前后端共享密钥完全匹配：
   - 前端:
     42396b6455ac67c603e978af2686c9ff0dacafd3dd64f1e2efe51ef05610d183
   - 后端:
     42396b6455ac67c603e978af2686c9ff0dacafd3dd64f1e2efe51ef05610d183
   - 解密成功: ✅ Plaintext length: 8
   - 登录成功: ✅ POST /api/login -> 200

  修改的文件

   1. Cargo.toml - 添加spki依赖
   2. src/handlers/api_handlers/crypto.rs - 修复ECDH密钥派生和公钥解析
   3. src/handlers/api_handlers/auth.rs - 添加调试日志
   4. templates/js/ecc-encrypt.js - 修复密钥派生方法

  经验教训

   1. Web Crypto 
      API的deriveBits行为：当指定256位时，返回压缩格式（仅X坐标）
   2. p256库的to_bytes()：返回SEC1编码格式，需要手动提取X坐标
   3. 跨平台密钥派生：必须确保使用完全相同的密钥派生方式
   4. 调试的重要性：详细的hex输出对比是定位问题的关键





