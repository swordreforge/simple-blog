// 文章页面专用快捷键扩展
// 在通用快捷键系统基础上添加文章页面的特殊功能快捷键

class PassageShortcuts {
  constructor() {
    this.shortcuts = {
      // 文章页面专用快捷键
      's': { action: 'toggleSidebar', label: '切换侧边栏' },
      't': { action: 'toggleReadingMode', label: '切换阅读模式' },
      'h': { action: 'toggleHeader', label: '切换标题栏和底栏' },
      'f': { action: 'toggleFullscreen', label: '全屏模式' },
      'Escape': { action: 'exitFullscreen', label: '退出全屏' }
    };

    this.enabled = true;
    this.init();
  }

  init() {
    // 等待页面加载完成
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => this.setupShortcuts());
    } else {
      this.setupShortcuts();
    }
  }

  setupShortcuts() {
    // 监听键盘事件
    document.addEventListener('keydown', (e) => this.handleKeyPress(e));

    // 显示快捷键提示
    this.showShortcutHints();

    // 更新帮助模态框
    this.updateHelpModal();
  }

  handleKeyPress(e) {
    // 如果快捷键功能被禁用,不处理
    if (!this.enabled) return;

    // 如果图片查看器或代码查看器打开，不处理快捷键
    const imageViewer = document.getElementById('imageViewer');
    const codeViewer = document.getElementById('codeViewer');
    if ((imageViewer && imageViewer.classList.contains('active')) ||
        (codeViewer && codeViewer.classList.contains('active'))) {
      return;
    }

    // 如果用户正在输入框中输入,不触发快捷键
    const activeElement = document.activeElement;
    if (activeElement && (
      activeElement.tagName === 'INPUT' ||
      activeElement.tagName === 'TEXTAREA' ||
      activeElement.isContentEditable
    )) {
      return;
    }

    const key = e.key;

    // 检查是否有对应的快捷键
    if (this.shortcuts[key]) {
      e.preventDefault();
      const shortcut = this.shortcuts[key];

      // 执行对应的操作
      this.executeAction(shortcut);
    }
  }

  executeAction(shortcut) {
    switch (shortcut.action) {
      case 'toggleSidebar':
        this.toggleSidebar();
        break;

      case 'toggleReadingMode':
        this.toggleReadingMode();
        break;

      case 'toggleHeader':
        this.toggleHeader();
        break;

      case 'toggleFullscreen':
        this.toggleFullscreen();
        break;

      case 'exitFullscreen':
        this.exitFullscreen();
        break;
    }
  }

  toggleSidebar() {
    const sidebarToggle = document.getElementById('sidebarToggle');
    if (sidebarToggle) {
      sidebarToggle.click();
      const isOpen = !document.getElementById('sidebar').classList.contains('hidden');
      this.showToast(isOpen ? '侧边栏已显示' : '侧边栏已隐藏', 'success');
    }
  }

  toggleReadingMode() {
    const readingModeToggle = document.getElementById('readingModeToggle');
    if (readingModeToggle) {
      readingModeToggle.click();
      const mode = document.getElementById('readingModeText').textContent;
      this.showToast(`已切换到${mode}模式`, 'success');
    }
  }

  toggleHeader() {
    const nav = document.querySelector('nav');
    const sidebarToggleFixed = document.getElementById('sidebarToggleFixed');

    if (nav && sidebarToggleFixed) {
      // 记录当前标题栏状态
      const wasNavHidden = nav.style.display === 'none';

      // 切换标题栏和底栏显示/隐藏
      sidebarToggleFixed.click();

      // 显示提示信息
      const isNowHidden = !wasNavHidden;
      this.showToast(isNowHidden ? '标题栏和底栏已隐藏' : '标题栏和底栏已显示', 'success');
    }
  }

  toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().then(() => {
        this.showToast('已进入全屏模式', 'success');
      }).catch(err => {
        this.showToast('无法进入全屏模式', 'error');
      });
    } else {
      this.exitFullscreen();
    }
  }

  exitFullscreen() {
    if (document.fullscreenElement) {
      document.exitFullscreen().then(() => {
        this.showToast('已退出全屏模式', 'success');
      });
    }
  }

  showShortcutHints() {
    // 检测移动端，如果是移动端则不显示快捷键提示
    const isMobile = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) || window.innerWidth <= 768;
    if (isMobile) {
      return; // 移动端不显示快捷键提示
    }

    // 为按钮添加快捷键提示
    const buttons = [
      { id: 'sidebarToggle', key: 's' },
      { id: 'readingModeToggle', key: 't' },
      { id: 'sidebarToggleFixed', key: 'h' }
    ];

    buttons.forEach(({ id, key }) => {
      const button = document.getElementById(id);
      if (button) {
        // 检查是否已经存在快捷键提示
        let hint = button.querySelector('.shortcut-hint');
        if (!hint) {
          hint = document.createElement('span');
          hint.className = 'shortcut-hint';
          hint.textContent = key;
          button.appendChild(hint);
        }
      }
    });
  }

  updateHelpModal() {
    // 更新通用快捷键系统的帮助模态框
    if (window.keyboardShortcuts) {
      const originalShowHelpModal = window.keyboardShortcuts.showHelpModal.bind(window.keyboardShortcuts);

      window.keyboardShortcuts.showHelpModal = () => {
        originalShowHelpModal();

        // 等待模态框创建后添加文章页面快捷键
        setTimeout(() => {
          const modal = document.querySelector('.shortcuts-help-modal');
          if (modal) {
            const shortcutsList = modal.querySelector('.shortcuts-list');
            if (shortcutsList) {
              // 添加文章页面快捷键部分
              const passageSection = document.createElement('div');
              passageSection.innerHTML = `
                <h4>文章页面快捷键</h4>
                ${this.renderShortcutList(['s', 't', 'h', 'f', 'Escape'])}
              `;
              shortcutsList.appendChild(passageSection);
            }
          }
        }, 100);
      };
    }
  }

  renderShortcutList(keys) {
    return keys.map(key => {
      const shortcut = this.shortcuts[key];
      if (!shortcut) return '';

      return `
        <div class="shortcut-item">
          <kbd class="shortcut-key">${key}</kbd>
          <span class="shortcut-label">${shortcut.label}</span>
        </div>
      `;
    }).join('');
  }

  showToast(message, type = 'info') {
    // 使用通用快捷键系统的 showToast 方法
    if (window.keyboardShortcuts && window.keyboardShortcuts.showToast) {
      window.keyboardShortcuts.showToast(message, type);
    } else {
      // 如果通用系统不可用,创建简单的提示
      console.log(`[${type.toUpperCase()}] ${message}`);
    }
  }

  enable() {
    this.enabled = true;
  }

  disable() {
    this.enabled = false;
  }
}

// 页面加载完成后初始化文章页面快捷键
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.passageShortcuts = new PassageShortcuts();
  });
} else {
  window.passageShortcuts = new PassageShortcuts();
}