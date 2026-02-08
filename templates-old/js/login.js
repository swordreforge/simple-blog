// 登录状态管理
const AuthManager = {
  isLoggedIn: false,
  currentUser: null,
  token: null,
  eccEncryptor: null,

  // 初始化
  init() {
    this.checkLoginStatus();
    this.setupEventListeners();
    this.updateUI();
    this.initECCEncryption();
  },

  // 初始化ECC加密
  async initECCEncryption() {
    try {
      // 检查浏览器支持
      const support = checkCryptoSupport();
      if (!support.supported) {
        let message = 'Web Crypto API not supported, falling back to plain text';
        if (support.reason === 'insecure_context') {
          message += ` (${support.protocol}://${support.hostname} is not a secure context - requires HTTPS or localhost)`;
        }
        console.warn(message);
        return;
      }

      // 创建ECC加密器
      this.eccEncryptor = new ECCEncryptor();
      await this.eccEncryptor.init();
      console.log('ECC encryption initialized successfully');
    } catch (error) {
      console.error('Failed to initialize ECC encryption:', error);
      this.eccEncryptor = null;
    }
  },

  // 检查登录状态
  checkLoginStatus() {
    const token = localStorage.getItem('auth_token');
    const user = localStorage.getItem('auth_user');
    
    if (token && user) {
      this.isLoggedIn = true;
      this.token = token;
      this.currentUser = JSON.parse(user);
    } else {
      this.isLoggedIn = false;
      this.token = null;
      this.currentUser = null;
    }
  },

  // 设置事件监听器
  setupEventListeners() {
    // 登录按钮点击
    const loginBtn = document.getElementById('loginBtn');
    if (loginBtn) {
      loginBtn.addEventListener('click', () => this.openLoginModal());
    }

    // 登录表单提交
    const loginForm = document.getElementById('loginForm');
    if (loginForm) {
      loginForm.addEventListener('submit', (e) => this.handleLogin(e));
    }

    // 关闭登录模态框
    const loginCloseBtn = document.querySelector('.modal-close[data-modal="loginModal"]');
    if (loginCloseBtn) {
      loginCloseBtn.addEventListener('click', () => this.closeLoginModal());
    }

    // 点击登录模态框外部关闭
    const loginModal = document.getElementById('loginModal');
    if (loginModal) {
      loginModal.addEventListener('click', (e) => {
        if (e.target === loginModal) {
          this.closeLoginModal();
        }
      });
    }

    // 打开注册模态框
    const openRegisterBtn = document.getElementById('openRegisterModal');
    if (openRegisterBtn) {
      openRegisterBtn.addEventListener('click', (e) => {
        e.preventDefault();
        this.closeLoginModal();
        this.openRegisterModal();
      });
    }

    // 注册表单提交
    const registerForm = document.getElementById('registerForm');
    if (registerForm) {
      registerForm.addEventListener('submit', (e) => this.handleRegister(e));
    }

    // 关闭注册模态框
    const registerCloseBtn = document.querySelector('.modal-close[data-modal="registerModal"]');
    if (registerCloseBtn) {
      registerCloseBtn.addEventListener('click', () => this.closeRegisterModal());
    }

    // 点击注册模态框外部关闭
    const registerModal = document.getElementById('registerModal');
    if (registerModal) {
      registerModal.addEventListener('click', (e) => {
        if (e.target === registerModal) {
          this.closeRegisterModal();
        }
      });
    }

    // 从注册模态框打开登录模态框
    const openLoginFromRegisterBtn = document.getElementById('openLoginModal');
    if (openLoginFromRegisterBtn) {
      openLoginFromRegisterBtn.addEventListener('click', (e) => {
        e.preventDefault();
        this.closeRegisterModal();
        this.openLoginModal();
      });
    }

    // 关闭个人中心模态框
    const userCenterCloseBtn = document.querySelector('.modal-close[data-modal="userCenterModal"]');
    if (userCenterCloseBtn) {
      userCenterCloseBtn.addEventListener('click', () => this.closeUserCenterModal());
    }

    // 点击个人中心模态框外部关闭
    const userCenterModal = document.getElementById('userCenterModal');
    if (userCenterModal) {
      userCenterModal.addEventListener('click', (e) => {
        if (e.target === userCenterModal) {
          this.closeUserCenterModal();
        }
      });
    }

    // 退出登录
    const logoutBtn = document.getElementById('logoutBtn');
    if (logoutBtn) {
      logoutBtn.addEventListener('click', () => this.handleLogout());
    }

    // 个人中心切换
    const userCenterToggle = document.getElementById('userCenterToggle');
    if (userCenterToggle) {
      userCenterToggle.addEventListener('click', () => this.openUserCenterModal());
    }

    // ESC键关闭所有模态框
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        this.closeLoginModal();
        this.closeRegisterModal();
        this.closeUserCenterModal();
      }
    });
  },

  // 打开登录模态框
  openLoginModal() {
    const modal = document.getElementById('loginModal');
    if (modal) {
      modal.classList.add('active');
      document.body.style.overflow = 'hidden';
      
      // 聚焦用户名输入框
      setTimeout(() => {
        const usernameInput = document.getElementById('loginUsername');
        if (usernameInput) {
          usernameInput.focus();
        }
      }, 100);
    }
  },

  // 关闭登录模态框
  closeLoginModal() {
    const modal = document.getElementById('loginModal');
    if (modal) {
      modal.classList.add('closing');
      setTimeout(() => {
        modal.classList.remove('active', 'closing');
        document.body.style.overflow = '';
        
        // 清空表单
        const loginForm = document.getElementById('loginForm');
        if (loginForm) {
          loginForm.reset();
        }
        
        // 清除错误信息
        const errorMessage = document.getElementById('loginError');
        if (errorMessage) {
          errorMessage.textContent = '';
          errorMessage.style.display = 'none';
        }
      }, 300);
    }
  },

  // 打开个人中心模态框
  openUserCenterModal() {
    const modal = document.getElementById('userCenterModal');
    if (modal) {
      // 确保用户名已更新
      const userCenterUsername = document.getElementById('userCenterUsername');
      if (userCenterUsername && this.currentUser) {
        userCenterUsername.textContent = this.currentUser.username;
      }
      modal.classList.add('active');
      document.body.style.overflow = 'hidden';
    }
  },

  // 关闭个人中心模态框
  closeUserCenterModal() {
    const modal = document.getElementById('userCenterModal');
    if (modal) {
      modal.classList.add('closing');
      setTimeout(() => {
        modal.classList.remove('active', 'closing');
        document.body.style.overflow = '';
      }, 300);
    }
  },

  // 打开注册模态框
  openRegisterModal() {
    const modal = document.getElementById('registerModal');
    if (modal) {
      modal.classList.add('active');
      document.body.style.overflow = 'hidden';
      
      // 聚焦用户名输入框
      setTimeout(() => {
        const usernameInput = document.getElementById('registerUsername');
        if (usernameInput) {
          usernameInput.focus();
        }
      }, 100);
    }
  },

  // 关闭注册模态框
  closeRegisterModal() {
    const modal = document.getElementById('registerModal');
    if (modal) {
      modal.classList.add('closing');
      setTimeout(() => {
        modal.classList.remove('active', 'closing');
        document.body.style.overflow = '';
        
        // 清空表单
        const registerForm = document.getElementById('registerForm');
        if (registerForm) {
          registerForm.reset();
        }
        
        // 清除错误信息
        const errorMessage = document.getElementById('registerError');
        if (errorMessage) {
          errorMessage.textContent = '';
          errorMessage.style.display = 'none';
        }
      }, 300);
    }
  },

  // 处理登录
  async handleLogin(e) {
    e.preventDefault();
    
    const usernameInput = document.getElementById('loginUsername');
    const passwordInput = document.getElementById('loginPassword');
    const errorMessage = document.getElementById('loginError');
    const submitBtn = document.getElementById('loginSubmitBtn');
    
    if (!usernameInput || !passwordInput) return;
    
    const username = usernameInput.value.trim();
    const password = passwordInput.value.trim();
    
    if (!username || !password) {
      if (errorMessage) {
        errorMessage.textContent = '请输入用户名和密码';
        errorMessage.style.display = 'block';
      }
      return;
    }
    
    // 禁用提交按钮
    if (submitBtn) {
      submitBtn.disabled = true;
      submitBtn.textContent = '登录中...';
    }
    
    try {
      // 准备登录数据
      let loginData = {
        username: username,
        password: password
      };

      // 如果ECC加密器可用，使用加密传输
      if (this.eccEncryptor && this.eccEncryptor.isReady()) {
        try {
          // 加密密码
          const encryptedData = await this.eccEncryptor.encrypt(password);
          
          // 使用加密数据
          loginData = {
            username: username,
            encrypted_password: encryptedData.encrypted,
            session_id: encryptedData.sessionId,
            client_public_key: encryptedData.clientPublicKey,
            algorithm: encryptedData.algorithm
          };
        } catch (encryptError) {
          console.warn('ECC encryption failed, falling back to plain text:', encryptError);
          // 加密失败时使用明文（不推荐）
        }
      }

      const response = await fetch('/api/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(loginData)
      });
      
      const result = await response.json();
      
      if (response.ok && result.success) {
        // 保存登录信息
        localStorage.setItem('auth_token', result.token);
        localStorage.setItem('auth_user', JSON.stringify(result.user));
        
        // 更新状态
        this.isLoggedIn = true;
        this.token = result.token;
        this.currentUser = result.user;
        
        // 更新UI
        this.updateUI();
        
        // 关闭登录模态框
        this.closeLoginModal();
        
        // 打开个人中心模态框
        this.openUserCenterModal();
        
        // 显示成功提示
        this.showNotification('登录成功！', 'success');
      } else {
        if (errorMessage) {
          errorMessage.textContent = result.message || '登录失败，请检查用户名和密码';
          errorMessage.style.display = 'block';
        }
      }
    } catch (error) {
      console.error('登录错误:', error);
      if (errorMessage) {
        errorMessage.textContent = '网络错误，请稍后重试';
        errorMessage.style.display = 'block';
      }
    } finally {
      // 恢复提交按钮
      if (submitBtn) {
        submitBtn.disabled = false;
        submitBtn.textContent = '登录';
      }
    }
  },

  // 处理注册
  async handleRegister(e) {
    e.preventDefault();
    
    const usernameInput = document.getElementById('registerUsername');
    const emailInput = document.getElementById('registerEmail');
    const passwordInput = document.getElementById('registerPassword');
    const errorMessage = document.getElementById('registerError');
    const submitBtn = document.getElementById('registerSubmitBtn');
    
    if (!usernameInput || !emailInput || !passwordInput) return;
    
    const username = usernameInput.value.trim();
    const email = emailInput.value.trim();
    const password = passwordInput.value.trim();
    
    if (!username || !email || !password) {
      if (errorMessage) {
        errorMessage.textContent = '请填写所有必填字段';
        errorMessage.style.display = 'block';
      }
      return;
    }
    
    // 禁用提交按钮
    if (submitBtn) {
      submitBtn.disabled = true;
      submitBtn.textContent = '注册中...';
    }
    
    try {
      // 准备注册数据
      let registerData = {
        username: username,
        email: email,
        password: password
      };

      // 如果ECC加密器可用，使用加密传输
      if (this.eccEncryptor && this.eccEncryptor.isReady()) {
        try {
          // 加密密码
          const encryptedData = await this.eccEncryptor.encrypt(password);
          
          // 使用加密数据
          registerData = {
            username: username,
            email: email,
            encrypted_password: encryptedData.encrypted,
            session_id: encryptedData.sessionId,
            client_public_key: encryptedData.clientPublicKey,
            algorithm: encryptedData.algorithm
          };
        } catch (encryptError) {
          console.warn('ECC encryption failed, falling back to plain text:', encryptError);
          // 加密失败时使用明文（不推荐）
        }
      }

      const response = await fetch('/api/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(registerData)
      });
      
      const result = await response.json();
      
      if (response.ok && result.success) {
        // 注册成功，关闭注册模态框
        this.closeRegisterModal();
        
        // 显示成功提示
        this.showNotification('注册成功！请登录', 'success');
        
        // 自动打开登录模态框
        setTimeout(() => {
          this.openLoginModal();
          // 预填用户名
          const loginUsernameInput = document.getElementById('loginUsername');
          if (loginUsernameInput) {
            loginUsernameInput.value = username;
          }
        }, 500);
      } else {
        if (errorMessage) {
          errorMessage.textContent = result.message || '注册失败，请稍后重试';
          errorMessage.style.display = 'block';
        }
      }
    } catch (error) {
      console.error('注册错误:', error);
      if (errorMessage) {
        errorMessage.textContent = '网络错误，请稍后重试';
        errorMessage.style.display = 'block';
      }
    } finally {
      // 恢复提交按钮
      if (submitBtn) {
        submitBtn.disabled = false;
        submitBtn.textContent = '注册';
      }
    }
  },

  // 处理退出登录
  async handleLogout() {
    try {
      // 发送退出登录请求（带token）
      const token = this.getToken();
      if (token) {
        await fetch('/api/logout', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
          }
        });
      }
    } catch (error) {
      console.error('退出登录请求失败:', error);
    } finally {
      // 无论请求成功与否，都清除本地存储
      localStorage.removeItem('auth_token');
      localStorage.removeItem('auth_user');
      
      // 清除 cookie
      document.cookie = 'auth_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;';
      
      // 更新状态
      this.isLoggedIn = false;
      this.token = null;
      this.currentUser = null;
      
      // 更新UI
      this.updateUI();
      
      // 关闭个人中心模态框
      this.closeUserCenterModal();
      
      // 显示提示
      this.showNotification('已退出登录', 'info');
    }
  },

  // 切换个人中心（已废弃，现在使用模态框）
  toggleUserCenter() {
    // 此方法已废弃，现在使用模态框
    this.openUserCenterModal();
  },

  // 更新UI
  updateUI() {
    const loginBtn = document.getElementById('loginBtn');
    const userCenter = document.getElementById('userCenter');
    const userCenterToggle = document.getElementById('userCenterToggle');
    const usernameDisplay = document.getElementById('usernameDisplay');
    const userCenterUsername = document.getElementById('userCenterUsername');
    const adminOnlyElements = document.querySelectorAll('.admin-only');

    if (this.isLoggedIn && this.currentUser) {
      // 显示个人中心，隐藏登录按钮
      if (loginBtn) {
        loginBtn.style.display = 'none';
      }
      if (userCenter) {
        userCenter.style.display = 'block';
      }
      if (userCenterToggle) {
        userCenterToggle.style.display = 'flex';
      }
      if (usernameDisplay) {
        usernameDisplay.textContent = this.currentUser.username;
      }
      if (userCenterUsername) {
        userCenterUsername.textContent = this.currentUser.username;
      }

      // 检查是否为管理员，显示或隐藏所有管理员专用元素
      adminOnlyElements.forEach((element) => {
        if (this.currentUser.role === 'admin') {
          // 使用!important来覆盖CSS中的!important规则
          // 根据元素类型设置合适的display值
          if (element.tagName === 'A' && !element.classList.contains('user-center-item')) {
            // 导航栏链接使用 inline-block
            element.style.setProperty('display', 'inline-block', 'important');
          } else if (element.classList.contains('user-center-item')) {
            // 模态框菜单项使用 flex
            element.style.setProperty('display', 'flex', 'important');
          } else {
            // 其他元素使用 block
            element.style.setProperty('display', 'block', 'important');
          }
        } else {
          element.style.setProperty('display', 'none', 'important');
        }
      });
    } else {
      // 显示登录按钮，隐藏个人中心
      if (loginBtn) {
        loginBtn.style.display = 'flex';
      }
      if (userCenter) {
        userCenter.style.display = 'none';
      }
      if (userCenterToggle) {
        userCenterToggle.style.display = 'none';
      }
      if (userCenterUsername) {
        userCenterUsername.textContent = '用户名';
      }

      // 隐藏所有管理员专用元素
      adminOnlyElements.forEach(element => {
        element.style.display = 'none';
      });
    }
  },

  // 显示通知
  showNotification(message, type = 'info') {
    // 创建通知元素
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    
    // 添加样式
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 15px 25px;
      background: ${type === 'success' ? '#00b894' : type === 'error' ? '#e74c3c' : '#007bff'};
      color: white;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      z-index: 10000;
      animation: slideInRight 0.3s ease;
      font-weight: 500;
    `;
    
    // 添加动画样式
    if (!document.getElementById('notification-styles')) {
      const style = document.createElement('style');
      style.id = 'notification-styles';
      style.textContent = `
        @keyframes slideInRight {
          from {
            transform: translateX(100%);
            opacity: 0;
          }
          to {
            transform: translateX(0);
            opacity: 1;
          }
        }
        @keyframes slideOutRight {
          from {
            transform: translateX(0);
            opacity: 1;
          }
          to {
            transform: translateX(100%);
            opacity: 0;
          }
        }
      `;
      document.head.appendChild(style);
    }
    
    // 添加到页面
    document.body.appendChild(notification);
    
    // 3秒后移除
    setTimeout(() => {
      notification.style.animation = 'slideOutRight 0.3s ease';
      setTimeout(() => {
        notification.remove();
      }, 300);
    }, 3000);
  },

  // 获取认证令牌
  getToken() {
    return this.token;
  },

  // 获取当前用户
  getCurrentUser() {
    return this.currentUser;
  },

  // 检查是否已登录
  isAuthenticated() {
    return this.isLoggedIn;
  },

  // 发送带认证的请求
  async authenticatedFetch(url, options = {}) {
    const token = this.getToken();
    if (!token) {
      throw new Error('未登录');
    }
    
    const headers = {
      ...options.headers,
      'Authorization': `Bearer ${token}`
    };
    
    const response = await fetch(url, {
      ...options,
      headers
    });
    
    // 如果返回401，说明token过期或无效
    if (response.status === 401) {
      this.handleLogout();
      throw new Error('登录已过期，请重新登录');
    }
    
    return response;
  }
};

// 页面加载时初始化
document.addEventListener('DOMContentLoaded', () => {
  AuthManager.init();
});

// 导出到全局
window.AuthManager = AuthManager;