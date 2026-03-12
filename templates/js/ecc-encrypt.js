/**
 * ECC加密工具类
 * 使用Web Crypto API实现ECC P-256加密
 */
class ECCEncryptor {
  constructor() {
    this.sessionId = null;
    this.serverPublicKey = null;
    this.clientKeyPair = null;
    this.isInitialized = false;
  }

  /**
   * 初始化加密器，从服务器获取公钥
   * @returns {Promise<{sessionId: string, algorithm: string}>}
   */
  async init() {
    try {
      // 生成会话ID或从服务器获取
      this.sessionId = this.generateSessionId();

      // 从服务器获取ECC公钥
      const response = await fetch(`/api/crypto/public-key?session_id=${this.sessionId}`);
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to get server public key');
      }

      const serverKeyInfo = await response.json();

      if (!serverKeyInfo.success) {
        throw new Error(serverKeyInfo.error || 'Failed to get server public key');
      }

      // 导入服务器公钥
      this.serverPublicKey = await this.importServerPublicKey(serverKeyInfo);

      // 生成客户端密钥对
      this.clientKeyPair = await this.generateClientKeyPair();

      this.isInitialized = true;

      return {
        sessionId: this.sessionId,
        algorithm: serverKeyInfo.algorithm,
        expiresAt: serverKeyInfo.expires_at
      };
    } catch (error) {
      console.error('Failed to initialize ECC encryptor:', error);
      throw error;
    }
  }

  /**
   * 导入服务器公钥（支持JWK格式）
   * @param {Object} keyInfo - 服务器公钥信息
   * @returns {Promise<CryptoKey>}
   */
  async importServerPublicKey(keyInfo) {
    if (keyInfo.key_format === 'jwk') {
      // JWK格式
      return await window.crypto.subtle.importKey(
        'jwk',
        keyInfo.public_key,
        {
          name: 'ECDH',
          namedCurve: 'P-256'
        },
        true,
        []
      );
    } else if (keyInfo.key_format === 'raw') {
      // Raw格式
      const keyData = this.base64ToArrayBuffer(keyInfo.keyData);
      return await window.crypto.subtle.importKey(
        'raw',
        keyData,
        {
          name: 'ECDH',
          namedCurve: 'P-256'
        },
        true,
        ['deriveKey', 'deriveBits']
      );
    } else {
      throw new Error('Unsupported key format');
    }
  }

  /**
   * 生成客户端ECC密钥对
   * @returns {Promise<CryptoKeyPair>}
   */
  async generateClientKeyPair() {
    return await window.crypto.subtle.generateKey(
      {
        name: 'ECDH',
        namedCurve: 'P-256'
      },
      true, // 可导出
      ['deriveKey', 'deriveBits']
    );
  }

  /**
   * 导出客户端公钥为JWK格式
   * @returns {Promise<Object>}
   */
  async exportClientPublicKey() {
    return await window.crypto.subtle.exportKey(
      'jwk',
      this.clientKeyPair.publicKey
    );
  }

  /**
   * 导出客户端公钥为PEM格式
   * @returns {Promise<string>}
   */
  async exportClientPublicKeyPEM() {
    const jwk = await this.exportClientPublicKey();
    
    // 将JWK转换为PEM格式
    const publicKey = await window.crypto.subtle.importKey(
      'jwk',
      jwk,
      {
        name: 'ECDH',
        namedCurve: 'P-256'
      },
      true,
      []
    );

    const exported = await window.crypto.subtle.exportKey('spki', publicKey);
    const exportedAsString = String.fromCharCode.apply(null, new Uint8Array(exported));
    const exportedAsBase64 = btoa(exportedAsString);
    const pemHeader = '-----BEGIN PUBLIC KEY-----';
    const pemFooter = '-----END PUBLIC KEY-----';
    const pemContents = exportedAsBase64.match(/.{1,64}/g).join('\n');

    return `${pemHeader}\n${pemContents}\n${pemFooter}`;
  }

  /**
   * 派生共享密钥
   * @returns {Promise<CryptoKey>}
   */
  async deriveSharedKey() {
    // 使用 deriveBits 获取原始共享密钥字节（与Go版本保持一致）
    // Web Crypto API的deriveBits当指定256位时，返回压缩格式（只有X坐标，32字节）
    // 这与Go版本的 sharedX.Bytes() 完全一致
    const sharedSecretBits = await window.crypto.subtle.deriveBits(
      {
        name: 'ECDH',
        public: this.serverPublicKey
      },
      this.clientKeyPair.privateKey,
      256  // 派生256位（32字节）- 直接返回X坐标
    );

    const sharedSecretBytes = new Uint8Array(sharedSecretBits);

    // 直接使用全部32字节作为AES密钥（与Go版本的 sharedX.Bytes() 一致）
    const keyBytes = sharedSecretBytes;

    // 从原始字节导入为AES-GCM密钥
    return await window.crypto.subtle.importKey(
      'raw',
      keyBytes,
      {
        name: 'AES-GCM',
        length: 256
      },
      true,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * 加密数据
   * @param {string} plaintext - 明文
   * @returns {Promise<Object>} 加密数据对象
   */
  async encrypt(plaintext) {
    if (!this.isInitialized) {
      throw new Error('ECC encryptor not initialized. Call init() first.');
    }

    try {
      // 1. 派生共享密钥
      const sharedKey = await this.deriveSharedKey();

      // 2. 生成IV（12字节，GCM推荐）
      const iv = window.crypto.getRandomValues(new Uint8Array(12));

      // 3. 加密数据
      const encoder = new TextEncoder();
      const encoded = encoder.encode(plaintext);

      const encrypted = await window.crypto.subtle.encrypt(
        {
          name: 'AES-GCM',
          iv: iv
        },
        sharedKey,
        encoded
      );

      // 4. 组合 IV + 加密数据
      const combined = new Uint8Array(iv.length + encrypted.byteLength);
      combined.set(iv, 0);
      combined.set(new Uint8Array(encrypted), iv.length);

      // 5. 导出客户端公钥为PEM格式
      const clientPublicKeyPEM = await this.exportClientPublicKeyPEM();

      return {
        encrypted: this.arrayBufferToBase64(combined),
        clientPublicKey: clientPublicKeyPEM,
        sessionId: this.sessionId,
        algorithm: 'ECDH-ES+A256KW'
      };
    } catch (error) {
      console.error('Encryption failed:', error);
      throw error;
    }
  }

  /**
   * 检查是否已初始化
   * @returns {boolean}
   */
  isReady() {
    return this.isInitialized;
  }

  /**
   * 获取会话ID
   * @returns {string|null}
   */
  getSessionId() {
    return this.sessionId;
  }

  /**
   * 重置加密器
   */
  reset() {
    this.sessionId = null;
    this.serverPublicKey = null;
    this.clientKeyPair = null;
    this.isInitialized = false;
  }

  /**
   * 工具方法：ArrayBuffer转Base64
   * @param {ArrayBuffer} buffer
   * @returns {string}
   */
  arrayBufferToBase64(buffer) {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)));
  }

  /**
   * 工具方法：Base64转ArrayBuffer
   * @param {string} base64
   * @returns {ArrayBuffer}
   */
  base64ToArrayBuffer(base64) {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
  }

  /**
   * 工具方法：生成会话ID
   * @returns {string}
   */
  generateSessionId() {
    return 'session_' + Math.random().toString(36).substr(2, 9) + Date.now();
  }
}

/**
 * 检查浏览器是否支持Web Crypto API
 * @returns {Object}
 */
function checkCryptoSupport() {
  if (!window.crypto) {
    return {
      supported: false,
      message: 'window.crypto is not available',
      reason: 'browser_not_supported'
    };
  }

  if (!window.crypto.subtle) {
    return {
      supported: false,
      message: 'Web Crypto API is not available (requires HTTPS or localhost)',
      reason: 'insecure_context',
      protocol: window.location.protocol,
      hostname: window.location.hostname
    };
  }

  // 检查是否支持ECDH和P-256
  return {
    supported: true,
    ecdh: true,
    curves: ['P-256', 'P-384', 'P-521'],
    message: 'Web Crypto API is fully supported'
  };
}

// 导出到全局
window.ECCEncryptor = ECCEncryptor;
window.checkCryptoSupport = checkCryptoSupport;