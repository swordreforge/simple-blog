/**
 * 归档页面聚焦模式和键盘导航
 */
class CollectFocusMode {
  constructor() {
    this.focusMode = false;
    this.currentLevel = 'main'; // 'main' 或 'sub'
    this.mainIndex = 0; // 主卡片索引
    this.subIndex = 0; // 子项索引
    this.mainItems = []; // 主卡片列表
    this.subItems = []; // 子项列表

    this.init();
  }

  init() {
    // 监听键盘事件
    document.addEventListener('keydown', (e) => this.handleKeyPress(e));
  }

  handleKeyPress(e) {
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

    // ESC 键：返回上一级或暂时退出
    if (e.key === 'Escape') {
      e.preventDefault();
      if (this.currentLevel === 'sub') {
        this.returnToMain();
      } else {
        this.temporarilyExitFocusMode();
      }
      return;
    }

    // 根据当前层级处理按键
    if (this.currentLevel === 'main') {
      this.handleMainLevel(e);
    } else {
      this.handleSubLevel(e);
    }
  }

  enterFocusMode() {
    this.focusMode = true;
    this.currentLevel = 'main';
    this.mainIndex = 0;
    this.subIndex = 0;

    // 添加聚焦模式样式
    document.body.classList.add('collect-focus-mode');

    // 收集主卡片
    this.collectMainItems();

    // 显示初始选中项
    this.updateSelection();

    // 显示提示
    this.showToast('已进入归档聚焦模式 (按 q 退出)');
  }

  exitFocusMode() {
    this.focusMode = false;

    // 移除聚焦模式样式
    document.body.classList.remove('collect-focus-mode');
    document.body.classList.remove('collect-focus-mode-main');
    document.body.classList.remove('collect-focus-mode-sub');

    // 清除所有选中样式
    this.clearSelection();

    // 显示提示
    this.showToast('已退出归档聚焦模式');
  }

  temporarilyExitFocusMode() {
    // 暂时移除聚焦模式样式，但不清除状态
    document.body.classList.remove('collect-focus-mode');
    document.body.classList.remove('collect-focus-mode-main');
    document.body.classList.remove('collect-focus-mode-sub');
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

  collectMainItems() {
    this.mainItems = [];

    // 收集筛选归档卡片
    const filterCard = document.querySelector('.archive-filter');
    if (filterCard) {
      this.mainItems.push({
        element: filterCard,
        type: 'filter',
        title: '筛选归档'
      });
    }

    // 收集文章卡片
    const documentCards = document.querySelectorAll('.document-card');
    documentCards.forEach(card => {
      this.mainItems.push({
        element: card,
        type: 'article',
        title: card.querySelector('.document-title')?.textContent || '文章'
      });
    });

    // 收集时间线卡片
    const timelineCard = document.querySelector('.timeline-container');
    if (timelineCard) {
      this.mainItems.push({
        element: timelineCard,
        type: 'timeline',
        title: '文章时间线'
      });
    }

    // 收集标签云卡片
    const tagsCard = document.querySelector('.tags-container');
    if (tagsCard) {
      this.mainItems.push({
        element: tagsCard,
        type: 'tags',
        title: '标签云'
      });
    }
  }

  collectSubItems(mainItem) {
    this.subItems = [];

    if (mainItem.type === 'filter') {
      // 收集筛选按钮
      const filterBtns = mainItem.element.querySelectorAll('.filter-btn');
      filterBtns.forEach(btn => {
        this.subItems.push({
          element: btn,
          type: 'filter-btn',
          title: btn.textContent
        });
      });
    } else if (mainItem.type === 'article') {
      // 收集文章的标签
      const tags = mainItem.element.querySelectorAll('.tag');
      tags.forEach(tag => {
        this.subItems.push({
          element: tag,
          type: 'tag',
          title: tag.textContent
        });
      });
    } else if (mainItem.type === 'timeline') {
      // 收集时间线年份
      const years = mainItem.element.querySelectorAll('.timeline-year');
      years.forEach(year => {
        this.subItems.push({
          element: year,
          type: 'year',
          title: year.textContent
        });
      });
    } else if (mainItem.type === 'tags') {
      // 收集标签云标签
      const cloudTags = mainItem.element.querySelectorAll('.cloud-tag');
      cloudTags.forEach(tag => {
        this.subItems.push({
          element: tag,
          type: 'cloud-tag',
          title: tag.textContent
        });
      });
    }
  }

  handleMainLevel(e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (this.mainIndex < this.mainItems.length - 1) {
        this.mainIndex++;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (this.mainIndex > 0) {
        this.mainIndex--;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      // 在主级别，右键可以用于快速跳转到下一个卡片
      if (this.mainIndex < this.mainItems.length - 1) {
        this.mainIndex++;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      // 在主级别，左键可以用于快速跳转到上一个卡片
      if (this.mainIndex > 0) {
        this.mainIndex--;
        this.updateSelection();
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      this.enterSubLevel();
    }
  }

  handleSubLevel(e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (this.subIndex < this.subItems.length - 1) {
        this.subIndex++;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (this.subIndex > 0) {
        this.subIndex--;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      // 在子级别，右键可以用于快速跳转到下一个子项
      if (this.subIndex < this.subItems.length - 1) {
        this.subIndex++;
        this.updateSelection();
      }
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      // 在子级别，左键可以用于快速跳转到上一个子项
      if (this.subIndex > 0) {
        this.subIndex--;
        this.updateSelection();
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      this.activateSubItem();
    }
  }

  enterSubLevel() {
    if (!this.mainItems[this.mainIndex]) return;

    const mainItem = this.mainItems[this.mainIndex];

    // 如果是文章卡片，直接跳转到文章阅读页面
    if (mainItem.type === 'article') {
      const link = mainItem.element.querySelector('.document-link');
      if (link) {
        window.location.href = link.href;
      }
      return;
    }

    // 收集子项
    this.collectSubItems(mainItem);

    if (this.subItems.length === 0) {
      // 如果没有子项，直接点击主卡片
      mainItem.element.click();
      return;
    }

    // 进入子级别
    this.currentLevel = 'sub';
    this.subIndex = 0;

    document.body.classList.add('collect-focus-mode-sub');
    document.body.classList.remove('collect-focus-mode-main');

    this.updateSelection();
    this.showToast(`进入 ${mainItem.title} 子菜单`);
  }

  returnToMain() {
    this.currentLevel = 'main';
    this.subIndex = 0;

    document.body.classList.add('collect-focus-mode-main');
    document.body.classList.remove('collect-focus-mode-sub');

    this.updateSelection();
    this.showToast('返回主菜单');
  }

  activateSubItem() {
    if (!this.subItems[this.subIndex]) return;

    const subItem = this.subItems[this.subIndex];
    subItem.element.click();
  }

  updateSelection() {
    // 清除之前的选中样式
    this.clearSelection();

    if (this.currentLevel === 'main' && this.mainItems[this.mainIndex]) {
      const item = this.mainItems[this.mainIndex];
      item.element.classList.add('collect-focus-selected');
      item.element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    } else if (this.currentLevel === 'sub' && this.subItems[this.subIndex]) {
      const item = this.subItems[this.subIndex];
      item.element.classList.add('collect-focus-selected');
      item.element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  clearSelection() {
    document.querySelectorAll('.collect-focus-selected').forEach(el => {
      el.classList.remove('collect-focus-selected');
    });
  }

  showToast(message) {
    const toast = document.createElement('div');
    toast.className = 'collect-focus-toast';
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
    window.collectFocusMode = new CollectFocusMode();
  });
} else {
  window.collectFocusMode = new CollectFocusMode();
}