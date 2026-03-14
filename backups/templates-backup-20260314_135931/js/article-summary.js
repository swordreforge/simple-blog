/**
 * 文章摘要显示和隐藏控制
 * 根据 passage_summarize_enabled 设置项决定是否显示文章摘要
 */

(function() {
  'use strict';

  let passageSummarizeEnabled = true; // 默认启用
  let isLoaded = false;

  /**
   * 初始化文章摘要功能
   */
  async function initArticleSummary() {
    if (isLoaded) {
      return;
    }

    try {
      // 从服务器获取 passage_summarize_enabled 设置
      const response = await fetch('/api/settings/template');
      if (response.ok) {
        const settings = await response.json();
        passageSummarizeEnabled = settings.passage_summarize_enabled !== false;
      }
    } catch (error) {
      // 保持默认值 true
      passageSummarizeEnabled = true;
    }

    isLoaded = true;
  }

  /**
   * 处理单篇文章的摘要显示
   * @param {HTMLElement} article - 文章元素
   * @param {Object|null} articleData - 文章数据（可选）
   */
  function processArticle(article, articleData = null) {
    // 检查是否已经处理过
    if (article.hasAttribute('data-summary-processed')) {
      return;
    }

    const articleHeader = article.querySelector('.article-header');
    const articleContent = article.querySelector('.article-content');
    const articleTitle = article.querySelector('.article-title');

    if (!articleHeader || !articleContent) {
      return;
    }

    // 标记为已处理
    article.setAttribute('data-summary-processed', 'true');

    // 如果未启用摘要功能，直接返回
    if (!passageSummarizeEnabled) {
      return;
    }

    // 获取摘要
    let summary = null;
    
    // 如果直接提供了 articleData，优先使用
    if (articleData && articleData.summary) {
      summary = articleData.summary;
    }

    if (!summary) {
      return;
    }

    // 创建摘要元素
    const summaryElement = createSummaryElement(summary);
    
    // 将摘要插入到标题和内容之间
    if (articleTitle) {
      articleHeader.insertAdjacentElement('afterend', summaryElement);
    } else {
      articleHeader.insertAdjacentElement('afterend', summaryElement);
    }
  }

  /**
   * 创建摘要元素
   */
  function createSummaryElement(summaryText) {
    const summaryDiv = document.createElement('div');
    summaryDiv.className = 'article-summary';
    summaryDiv.innerHTML = `
      <div class="summary-content">
        <div class="summary-header">
          <span class="summary-label">摘要</span>
          <button class="summary-toggle" title="展开/收起摘要">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </button>
        </div>
        <div class="summary-text">${summaryText}</div>
      </div>
    `;

    // 添加切换功能
    const toggleBtn = summaryDiv.querySelector('.summary-toggle');
    
    // 默认展开
    summaryDiv.classList.add('expanded');

    toggleBtn.addEventListener('click', () => {
      summaryDiv.classList.toggle('expanded');
      // 旋转图标
      if (summaryDiv.classList.contains('expanded')) {
        toggleBtn.style.transform = 'rotate(0deg)';
      } else {
        toggleBtn.style.transform = 'rotate(180deg)';
      }
    });

    return summaryDiv;
  }

  // 导出函数供外部调用
  window.ArticleSummary = {
    init: initArticleSummary,
    processArticle: processArticle,
    isEnabled: () => passageSummarizeEnabled
  };

  // DOM 加载完成后自动初始化
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initArticleSummary);
  } else {
    initArticleSummary();
  }

})();