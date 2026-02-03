// 模态框动画控制脚本

// 带动画的模态框关闭函数
function closeModalWithAnimation(modal) {
  if (!modal) return;
  
  modal.classList.add('closing');
  
  setTimeout(() => {
    modal.classList.remove('active', 'closing');
  }, 300);
}

// 通用模态框关闭函数 - 关闭所有打开的模态框
function closeAllModals() {
  document.querySelectorAll('.modal.active').forEach(modal => {
    closeModalWithAnimation(modal);
  });
}

// 初始化模态框事件监听器
function initModalAnimations() {
  // ESC键关闭所有模态框
  document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
      closeAllModals();
    }
  });
  
  // 为所有模态框添加点击外部关闭功能
  document.querySelectorAll('.modal').forEach(modal => {
    const modalContent = modal.querySelector('.modal-content');
    
    if (modalContent) {
      // 点击模态框内容区域不关闭
      modalContent.addEventListener('click', function(e) {
        e.stopPropagation();
      });
    }
    
    // 点击模态框外部关闭
    modal.addEventListener('click', function() {
      closeModalWithAnimation(modal);
    });
  });
  
  // 为所有模态框关闭按钮添加事件
  document.querySelectorAll('.modal-close').forEach(closeBtn => {
    closeBtn.addEventListener('click', function() {
      const modalId = this.getAttribute('data-modal');
      const modal = modalId ? document.getElementById(modalId) : this.closest('.modal');
      if (modal) {
        closeModalWithAnimation(modal);
      }
    });
  });
}

// 页面加载完成后初始化
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initModalAnimations);
} else {
  initModalAnimations();
}

// 导出函数供其他脚本使用
window.closeModalWithAnimation = closeModalWithAnimation;
window.closeAllModals = closeAllModals;