// 快捷操作面板功能
(function() {
  // 等待页面加载完成
  document.addEventListener('DOMContentLoaded', function() {
    const quickActionsBtn = document.getElementById('quickActionsBtn');
    const quickActionsPanel = document.getElementById('quickActionsPanel');
    let quickActionsClosing = false;

    // 点击按钮切换面板
    if (quickActionsBtn) {
      quickActionsBtn.addEventListener('click', function(e) {
        e.stopPropagation();
        const isShown = quickActionsPanel.classList.contains('show');

        if (isShown) {
          // 关闭时先让内容淡出
          quickActionsClosing = true;
          quickActionsPanel.classList.remove('show');
          setTimeout(() => {
            quickActionsClosing = false;
          }, 500);
        } else {
          quickActionsPanel.classList.add('show');
        }
      });
    }

    // 点击其他地方关闭面板
    document.addEventListener('click', function(e) {
      if (!quickActionsBtn.contains(e.target) && !quickActionsPanel.contains(e.target)) {
        if (quickActionsPanel.classList.contains('show') && !quickActionsClosing) {
          quickActionsClosing = true;
          quickActionsPanel.classList.remove('show');
          setTimeout(() => {
            quickActionsClosing = false;
          }, 500);
        }
      }
    });
  });
})();