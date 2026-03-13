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
    if (isLoaded) return;

    try {
      // 从服务器获取 passage_summarize_enabled 设置
      const response = await fetch('/api/settings/template');
      if (response.ok) {
        const settings = await response.json();
        passageSummarizeEnabled = settings.passage_summarize_enabled !== false;
      }
    } catch (error) {
      console.error('获取文章摘要设置失败:', error);
      // 保持默认值 true
    }

    // 监听文章加载事件
    observeArticleChanges();
    
    // 监听文章数据加载完成事件
    document.addEventListener('articleLoaded', handleArticleLoaded);
    
    // 处理已存在的文章
    processExistingArticles();

    isLoaded = true;
  }

  /**
   * 处理文章加载完成事件
   */
  function handleArticleLoaded(event) {
    const articleId = event.detail.articleId;
    const article = document.getElementById(articleId) || document.querySelector('.article.active');
    if (article) {
      // 移除处理标记，重新处理文章
      article.removeAttribute('data-summary-processed');
      processArticle(article);
    }
  }

  /**
   * 观察文章内容变化
   */
  function observeArticleChanges() {
    // 监听文章内容的变化
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            const article = node.closest && node.closest('.article');
            if (article) {
              processArticle(article);
            }
          }
        });
      });
    });

    // 观察文章容器
    const articleContainer = document.getElementById('articleContent');
    if (articleContainer) {
      observer.observe(articleContainer, {
        childList: true,
        subtree: true
      });
    }
  }

  /**
   * 处理已存在的文章
   */
  function processExistingArticles() {
    const articles = document.querySelectorAll('.article.active');
    articles.forEach(article => {
      processArticle(article);
    });
  }

  /**
   * 处理单篇文章的摘要显示
   */
  async function processArticle(article) {
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

    // 从文章数据中获取摘要
    let summary = null;
    const articleData = getArticleData(article);
    
    if (articleData && articleData.summary) {
      summary = articleData.summary;
    } else {
      // 备用方案：尝试从 API 获取摘要
      const articleId = article.id || article.getAttribute('data-article-id');
      if (articleId) {
        summary = await fetchSummaryFromAPI(articleId);
      }
    }

    if (!summary) {
      return;
    }

    // 创建摘要元素
    const summaryElement = createSummaryElement(summary);
    
    // 将摘要插入到标题和内容之间
    if (articleTitle) {
      // 如果有标题，插入到标题之后
      articleHeader.insertAdjacentElement('afterend', summaryElement);
    } else {
      // 如果没有标题，插入到 header 之后
      articleHeader.insertAdjacentElement('afterend', summaryElement);
    }
  }

  /**
   * 从 API 获取文章摘要
   */
  async function fetchSummaryFromAPI(articleId) {
    try {
      // 从 articleId 中提取 UUID
      const uuid = articleId.replace('article-', '');
      const response = await fetch(`/api/passages/${uuid}`);
      if (response.ok) {
        const result = await response.json();
        if (result.success && result.data && result.data.summary) {
          return result.data.summary;
        }
      }
    } catch (error) {
      console.error('从 API 获取摘要失败:', error);
    }
    return null;
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
    const summaryTextEl = summaryDiv.querySelector('.summary-text');
    
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

  /**
   * 获取文章数据
   */
  function getArticleData(article) {
    // 尝试从全局变量获取文章数据
    const articleId = article.id || article.getAttribute('data-article-id');
    
    // 方法1: 从 articlesData 中查找文章（包含 summary）
    if (window.articlesData && window.articlesData.folders) {
      for (const folder of window.articlesData.folders) {
        const foundArticle = findArticleInFolder(folder, articleId);
        if (foundArticle) {
          return foundArticle;
        }
      }
    }
    
    // 方法2: 从 findArticleById 函数查找（passage.html 中的函数）
    if (typeof window.findArticleById === 'function') {
      const articleData = window.findArticleById(articleId);
      if (articleData) {
        return articleData;
      }
    }

    // 方法3: 从缓存中获取文章数据
    if (window.articleContentCache && articleId) {
      const cached = window.articleContentCache.get(articleId);
      if (cached) {
        // 如果缓存中有 articleData，直接返回
        if (cached.articleData) {
          return cached.articleData;
        }
        // 否则尝试从原始数据构建
        if (cached.summary) {
          return { summary: cached.summary };
        }
      }
    }

    // 方法4: 尝试从 data 属性获取
    const dataStr = article.getAttribute('data-article');
    if (dataStr) {
      try {
        return JSON.parse(dataStr);
      } catch (e) {
        console.error('解析文章数据失败:', e);
      }
    }

    // 方法5: 从当前活动的文章数据中获取（passage.html 中的全局变量）
    if (window.currentPassageData && window.currentPassageData.summary) {
      return window.currentPassageData;
    }

    return null;
  }

  /**
   * 在文件夹中递归查找文章
   */
  function findArticleInFolder(folder, articleId) {
    // 检查文件夹中的文章
    if (folder.articles) {
      for (const article of folder.articles) {
        if (article.id === articleId) {
          return article;
        }
      }
    }
    
    // 递归检查子文件夹
    if (folder.folders) {
      for (const subFolder of folder.folders) {
        const found = findArticleInFolder(subFolder, articleId);
        if (found) {
          return found;
        }
      }
    }
    
    return null;
  }

  /**
   * 更新文章数据
   */
  function updateArticleData(article, data) {
    if (data && data.summary) {
      // 如果文章已经处理过，重新处理
      if (article.hasAttribute('data-summary-processed')) {
        // 移除现有的摘要元素
        const existingSummary = article.querySelector('.article-summary');
        if (existingSummary) {
          existingSummary.remove();
        }
        // 移除处理标记
        article.removeAttribute('data-summary-processed');
        // 重新处理
        processArticle(article);
      }
    }
  }

  // 导出函数供外部调用
  window.ArticleSummary = {
    init: initArticleSummary,
    processArticle: processArticle,
    updateArticleData: updateArticleData,
    isEnabled: () => passageSummarizeEnabled
  };

  // DOM 加载完成后自动初始化
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initArticleSummary);
  } else {
    initArticleSummary();
  }

})();