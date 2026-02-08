/**
 * 关于页面聚焦模式和键盘导航
 */
class AboutFocusMode {
  constructor() {
    this.focusMode = false;
    this.selectedIndex = 0;
    this.items = [];

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

    // ESC 键：暂时退出聚焦模式（允许关闭模态框等操作）
    if (e.key === 'Escape') {
      e.preventDefault();
      this.temporarilyExitFocusMode();
      return;
    }

    // 上下键导航卡片
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
      // Enter 键可以用于滚动到卡片内容
      this.scrollToCard();
    }
  }

  enterFocusMode() {
    this.focusMode = true;
    this.selectedIndex = 0;

    // 添加聚焦模式样式
    document.body.classList.add('about-focus-mode');

    // 收集所有可导航的卡片
    this.collectItems();

    // 显示初始选中项
    this.updateSelection();

    // 显示提示
    this.showToast('已进入关于页面聚焦模式 (按 q 退出，上下键导航)');
  }

  exitFocusMode() {
    this.focusMode = false;

    // 移除聚焦模式样式
    document.body.classList.remove('about-focus-mode');

    // 清除所有选中样式
    this.clearSelection();

    // 显示提示
    this.showToast('已退出关于页面聚焦模式');
  }

  temporarilyExitFocusMode() {
    // 暂时移除聚焦模式样式，但不清除状态
    document.body.classList.remove('about-focus-mode');
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

  collectItems() {
    this.items = [];

    // 收集所有主卡片（about-card）
    const mainCards = document.querySelectorAll('.about-card');
    mainCards.forEach(card => {
      this.items.push({
        element: card,
        type: 'main-card',
        title: card.querySelector('h2')?.textContent || '卡片'
      });
    });

    // 收集特性卡片（feature-item）
    const featureItems = document.querySelectorAll('.feature-item');
    featureItems.forEach(item => {
      this.items.push({
        element: item,
        type: 'feature',
        title: item.querySelector('h3')?.textContent || '特性'
      });
    });

    // 收集团队成员卡片（team-member）
    const teamMembers = document.querySelectorAll('.team-member');
    teamMembers.forEach(member => {
      this.items.push({
        element: member,
        type: 'team',
        title: member.querySelector('h3')?.textContent || '团队成员'
      });
    });

    // 收集联系信息卡片（contact-item）
    const contactItems = document.querySelectorAll('.contact-item');
    contactItems.forEach(item => {
      this.items.push({
        element: item,
        type: 'contact',
        title: item.querySelector('h3')?.textContent || '联系方式'
      });
    });
  }

  updateSelection() {
    // 清除之前的选中样式
    this.clearSelection();

    if (this.items[this.selectedIndex]) {
      const item = this.items[this.selectedIndex];
      item.element.classList.add('about-focus-selected');
      item.element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  clearSelection() {
    document.querySelectorAll('.about-focus-selected').forEach(el => {
      el.classList.remove('about-focus-selected');
    });
  }

  scrollToCard() {
    if (!this.items[this.selectedIndex]) return;

    const item = this.items[this.selectedIndex];
    item.element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    this.showToast(`查看: ${item.title}`);
  }

  showToast(message) {
    const toast = document.createElement('div');
    toast.className = 'about-focus-toast';
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
    window.aboutFocusMode = new AboutFocusMode();
  });
} else {
  window.aboutFocusMode = new AboutFocusMode();
}