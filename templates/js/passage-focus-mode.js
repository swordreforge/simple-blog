/**
 * 文章页面文本聚焦模式和键盘导航
 */
class PassageFocusMode {
  constructor() {
    this.focusMode = false;
    this.currentPanel = 'left'; // 'left' 或 'right'
    this.selectedIndex = 0;
    this.items = [];
    this.articleItems = [];
    this.articleScrollPosition = 0;

    this.init();
  }

  init() {
    // 监听键盘事件
    document.addEventListener('keydown', (e) => this.handleKeyPress(e));
  }

  handleKeyPress(e) {
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

    // 进入聚焦模式
    if (e.key === 'i' && !this.focusMode) {
      e.preventDefault();
      this.enterFocusMode();
      return;
    }

    // 退出聚焦模式
    if (e.key === 'q' && this.focusMode) {
      e.preventDefault();
      this.exitFocusMode();
      return;
    }

    // 如果不在聚焦模式，不处理其他按键
    if (!this.focusMode) return;

    // ESC 键：暂时退出聚焦模式（允许关闭模态框等操作）
    if (e.key === 'Escape') {
      e.preventDefault();
      this.temporarilyExitFocusMode();
      return;
    }

    // 左右键切换面板
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      this.switchPanel('left');
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      this.switchPanel('right');
    }

    // 根据当前面板处理按键
    if (this.currentPanel === 'left') {
      this.handleLeftPanel(e);
    } else {
      this.handleRightPanel(e);
    }
  }

  enterFocusMode() {
    this.focusMode = true;
    this.currentPanel = 'left';
    this.selectedIndex = 0;
    this.articleScrollPosition = 0;

    // 添加聚焦模式样式
    document.body.classList.add('focus-mode');

    // 收集左侧面板的可导航项
    this.collectLeftPanelItems();

    // 收集右侧文章的所有可滚动元素
    this.collectRightPanelItems();

    // 显示初始选中项
    this.updateSelection();

    // 显示提示
    this.showToast('已进入文本聚焦模式 (按 q 退出)');
  }

  exitFocusMode() {
    this.focusMode = false;

    // 移除聚焦模式样式
    document.body.classList.remove('focus-mode');
    document.body.classList.remove('focus-mode-left');
    document.body.classList.remove('focus-mode-right');

    // 清除所有选中样式
    this.clearSelection();

    // 显示提示
    this.showToast('已退出文本聚焦模式');
  }

  temporarilyExitFocusMode() {
    // 暂时移除聚焦模式样式，但不清除状态
    document.body.classList.remove('focus-mode');
    document.body.classList.remove('focus-mode-left');
    document.body.classList.remove('focus-mode-right');
    this.clearSelection();

    this.showToast('聚焦模式已暂停 (按 i 重新进入)');

    // 监听下次按键，如果是 i 则重新进入聚焦模式
    const reEnterHandler = (e) => {
      if (e.key === 'i') {
        e.preventDefault();
        document.removeEventListener('keydown', reEnterHandler);
        this.enterFocusMode();
      }
    };
    document.addEventListener('keydown', reEnterHandler);
  }

  switchPanel(panel) {
    this.currentPanel = panel;
    this.selectedIndex = 0;

    if (panel === 'left') {
      document.body.classList.add('focus-mode-left');
      document.body.classList.remove('focus-mode-right');
      this.collectLeftPanelItems();
      this.updateSelection();
    } else {
      document.body.classList.add('focus-mode-right');
      document.body.classList.remove('focus-mode-left');
      this.collectRightPanelItems();
      this.updateSelection();
    }
  }

  collectLeftPanelItems() {
    this.items = [];

    // 收集所有可点击的文件项和文件夹
    const fileItems = document.querySelectorAll('.file-item, .folder-header');
    fileItems.forEach((item, index) => {
      this.items.push({
        element: item,
        type: item.classList.contains('folder-header') ? 'folder' : 'file'
      });
    });
  }

  collectRightPanelItems() {
    this.articleItems = [];

    // 收集文章中的所有标题和段落
    const article = document.querySelector('.article-content');
    if (article) {
      const headings = article.querySelectorAll('h2, h3, h4, h5, h6');
      const paragraphs = article.querySelectorAll('p');

      headings.forEach(h => {
        this.articleItems.push({ element: h, type: 'heading' });
      });

      paragraphs.forEach(p => {
        this.articleItems.push({ element: p, type: 'paragraph' });
      });
    }
  }

  handleLeftPanel(e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (this.selectedIndex < this.items.length - 1) {
        this.selectedIndex++;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (this.selectedIndex > 0) {
        this.selectedIndex--;
        this.updateSelection();
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      this.activateLeftItem();
    } else if (e.key === 'u') {
      e.preventDefault();
      this.toggleFolder();
    }
  }

  handleRightPanel(e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      this.scrollArticle('down');
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      this.scrollArticle('up');
    } else if (e.key === 'Enter') {
      e.preventDefault();
      // 在右侧面板，Enter 可以用于打开链接或执行其他操作
      this.showToast('文章浏览模式');
    }
  }

  updateSelection() {
    // 清除之前的选中样式
    this.clearSelection();

    if (this.currentPanel === 'left' && this.items[this.selectedIndex]) {
      const item = this.items[this.selectedIndex];
      item.element.classList.add('focus-selected');
      item.element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    } else if (this.currentPanel === 'right' && this.articleItems[this.selectedIndex]) {
      const item = this.articleItems[this.selectedIndex];
      item.element.classList.add('focus-selected');
      item.element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  clearSelection() {
    document.querySelectorAll('.focus-selected').forEach(el => {
      el.classList.remove('focus-selected');
    });
  }

  activateLeftItem() {
    if (!this.items[this.selectedIndex]) return;

    const item = this.items[this.selectedIndex];

    if (item.type === 'folder') {
      // 点击文件夹，切换展开/折叠
      this.toggleFolder();
    } else if (item.type === 'file') {
      // 点击文件，打开文章
      item.element.click();
      this.switchPanel('right');
    }
  }

  toggleFolder() {
    if (!this.items[this.selectedIndex]) return;

    const item = this.items[this.selectedIndex];
    if (item.type === 'folder') {
      const folder = item.element.closest('.folder');
      if (folder) {
        folder.classList.toggle('open');
        // 重新收集项目
        setTimeout(() => {
          this.collectLeftPanelItems();
          this.updateSelection();
        }, 100);
      }
    }
  }

  scrollArticle(direction) {
    const articleContainer = document.querySelector('.article-container');
    if (!articleContainer) return;

    const scrollAmount = 200;

    if (direction === 'down') {
      articleContainer.scrollTop += scrollAmount;
    } else if (direction === 'up') {
      articleContainer.scrollTop -= scrollAmount;
    }
  }

  showToast(message) {
    const toast = document.createElement('div');
    toast.className = 'focus-mode-toast';
    toast.textContent = message;
    document.body.appendChild(toast);

    setTimeout(() => {
      toast.classList.add('show');
    }, 10);

    setTimeout(() => {
      toast.classList.remove('show');
      setTimeout(() => toast.remove(), 300);
    }, 2000);
  }
}

// 初始化
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.passageFocusMode = new PassageFocusMode();
  });
} else {
  window.passageFocusMode = new PassageFocusMode();
}