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
    console.log('[ArticleSummary] 开始初始化...');
    
    if (isLoaded) {
      console.log('[ArticleSummary] 已经初始化，跳过');
      return;
    }

    try {
      // 从服务器获取 passage_summarize_enabled 设置
      console.log('[ArticleSummary] 正在获取文章摘要设置...');
      const response = await fetch('/api/settings/template');
      if (response.ok) {
        const settings = await response.json();
        passageSummarizeEnabled = settings.passage_summarize_enabled !== false;
        console.log('[ArticleSummary] 文章摘要设置:', passageSummarizeEnabled);
      } else {
        console.error('[ArticleSummary] 获取设置失败，响应状态:', response.status);
      }
    } catch (error) {
      console.error('[ArticleSummary] 获取文章摘要设置失败:', error);
      // 保持默认值 true
      passageSummarizeEnabled = true;
    }

    console.log('[ArticleSummary] 摘要功能启用状态:', passageSummarizeEnabled);

    // 监听文章加载事件
    observeArticleChanges();
    
    // 监听文章数据加载完成事件
    document.addEventListener('articleLoaded', handleArticleLoaded);
    
    // 处理已存在的文章
    processExistingArticles();

    isLoaded = true;
    console.log('[ArticleSummary] 初始化完成');
  }

  /**
   * 处理文章加载完成事件
   */
  function handleArticleLoaded(event) {
    console.log('[ArticleSummary] 收到 articleLoaded 事件:', event.detail);
    const articleId = event.detail.articleId;
    const article = document.getElementById(articleId) || document.querySelector('.article.active');
    console.log('[ArticleSummary] 找到的文章元素:', article);
    if (article) {
      console.log('[ArticleSummary] 移除处理标记并重新处理文章:', articleId);
      // 移除处理标记，重新处理文章
      article.removeAttribute('data-summary-processed');
      processArticle(article);
    } else {
      console.log('[ArticleSummary] 未找到文章元素');
    }
  }

  /**
   * 观察文章内容变化
   */
  function observeArticleChanges() {
    console.log('[ArticleSummary] 设置 MutationObserver...');
    // 监听文章内容的变化
    const observer = new MutationObserver((mutations) => {
      console.log('[ArticleSummary] 检测到 DOM 变化，变更数量:', mutations.length);
      mutations.forEach((mutation, index) => {
        console.log('[ArticleSummary] 变更', index, '类型:', mutation.type, '添加的节点数:', mutation.addedNodes.length);
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            console.log('[ArticleSummary] 添加的元素:', node.tagName, '类名:', node.className);
            const article = node.closest && node.closest('.article');
            if (article) {
              console.log('[ArticleSummary] 找到文章元素，开始处理:', article.id);
              processArticle(article);
            }
          }
        });
      });
    });

    // 观察文章容器
    const articleContainer = document.getElementById('articleContent');
    console.log('[ArticleSummary] 文章容器:', articleContainer);
    if (articleContainer) {
      observer.observe(articleContainer, {
        childList: true,
        subtree: true
      });
      console.log('[ArticleSummary] MutationObserver 已设置');
    } else {
      console.log('[ArticleSummary] 未找到文章容器');
    }
  }

  /**
   * 处理已存在的文章
   */
  function processExistingArticles() {
    console.log('[ArticleSummary] 处理已存在的文章...');
    const articles = document.querySelectorAll('.article.active');
    console.log('[ArticleSummary] 找到', articles.length, '个活动文章');
    
    // 调试：打印所有文章元素的详细信息
    articles.forEach((article, index) => {
      console.log('[ArticleSummary] 文章', index, '详情:');
      console.log('  - ID:', article.id);
      console.log('  - data-article-id:', article.getAttribute('data-article-id'));
      console.log('  - data-article:', article.getAttribute('data-article'));
      console.log('  - classList:', Array.from(article.classList));
      
      // 检查文章内部结构
      const articleHeader = article.querySelector('.article-header');
      const articleContent = article.querySelector('.article-content');
      const articleTitle = article.querySelector('.article-title');
      console.log('  - 有 header:', !!articleHeader);
      console.log('  - 有 content:', !!articleContent);
      console.log('  - 有 title:', !!articleTitle);
      
      // 检查是否已经有摘要元素
      const existingSummary = article.querySelector('.article-summary');
      console.log('  - 已有摘要:', !!existingSummary);
    });
    
    articles.forEach((article, index) => {
      console.log('[ArticleSummary] 处理文章', index, 'ID:', article.id);
      processArticle(article);
    });
  }

  /**
   * 处理单篇文章的摘要显示
   */
  async function processArticle(article) {
    const articleId = article.id || article.getAttribute('data-article-id');
    console.log('[ArticleSummary] 开始处理文章:', articleId);
    
    // 检查是否已经处理过
    if (article.hasAttribute('data-summary-processed')) {
      console.log('[ArticleSummary] 文章已处理过，跳过:', articleId);
      return;
    }

    const articleHeader = article.querySelector('.article-header');
    const articleContent = article.querySelector('.article-content');
    const articleTitle = article.querySelector('.article-title');

    console.log('[ArticleSummary] 文章元素检查 - Header:', !!articleHeader, 'Content:', !!articleContent, 'Title:', !!articleTitle);

    if (!articleHeader || !articleContent) {
      console.log('[ArticleSummary] 文章元素不完整，跳过处理');
      return;
    }

    // 标记为已处理
    article.setAttribute('data-summary-processed', 'true');

    // 如果未启用摘要功能，直接返回
    if (!passageSummarizeEnabled) {
      console.log('[ArticleSummary] 摘要功能未启用，跳过');
      return;
    }

    // 从文章数据中获取摘要
    console.log('[ArticleSummary] 开始获取文章数据...');
    let summary = null;
    const articleData = getArticleData(article);
    
    console.log('[ArticleSummary] 获取到的文章数据:', articleData);
    
    if (articleData && articleData.summary) {
      summary = articleData.summary;
      console.log('[ArticleSummary] 从文章数据中获取到摘要:', summary);
    } else {
      console.log('[ArticleSummary] 文章数据中没有摘要，尝试从 API 获取...');
      // 备用方案：尝试从 API 获取摘要
      const apiArticleId = article.id || article.getAttribute('data-article-id');
      if (apiArticleId) {
        summary = await fetchSummaryFromAPI(apiArticleId);
        console.log('[ArticleSummary] 从 API 获取到的摘要:', summary);
      }
    }

    if (!summary) {
      console.log('[ArticleSummary] 未能获取到摘要，跳过显示');
      return;
    }

    console.log('[ArticleSummary] 创建摘要元素...');
    // 创建摘要元素
    const summaryElement = createSummaryElement(summary);
    
    console.log('[ArticleSummary] 插入摘要元素到文章中...');
    // 将摘要插入到标题和内容之间
    if (articleTitle) {
      // 如果有标题，插入到标题之后
      articleHeader.insertAdjacentElement('afterend', summaryElement);
      console.log('[ArticleSummary] 摘要已插入到标题之后');
    } else {
      // 如果没有标题，插入到 header 之后
      articleHeader.insertAdjacentElement('afterend', summaryElement);
      console.log('[ArticleSummary] 摘要已插入到 header 之后');
    }
    
    console.log('[ArticleSummary] 文章处理完成:', articleId);
  }

  /**
   * 从 API 获取文章摘要
   */
  async function fetchSummaryFromAPI(articleId) {
    console.log('[ArticleSummary] fetchSummaryFromAPI - articleId:', articleId);
    try {
      // 从 articleId 中提取 UUID
      const uuid = articleId.replace('article-', '');
      console.log('[ArticleSummary] 提取的 UUID:', uuid);
      console.log('[ArticleSummary] 正在请求 API: /api/passages/' + uuid);
      
      const response = await fetch(`/api/passages/${uuid}`);
      console.log('[ArticleSummary] API 响应状态:', response.status);
      
      if (response.ok) {
        const result = await response.json();
        console.log('[ArticleSummary] API 返回数据:', result);
        
        if (result.success && result.data) {
          console.log('[ArticleSummary] result.data.summary:', result.data.summary);
          if (result.data.summary) {
            console.log('[ArticleSummary] 成功从 API 获取到摘要');
            return result.data.summary;
          } else {
            console.log('[ArticleSummary] API 返回的数据中没有摘要字段');
          }
        } else {
          console.log('[ArticleSummary] API 返回失败或没有 data 字段');
        }
      } else {
        console.error('[ArticleSummary] API 请求失败，状态码:', response.status);
      }
    } catch (error) {
      console.error('[ArticleSummary] 从 API 获取摘要失败:', error);
    }
    console.log('[ArticleSummary] 未能从 API 获取摘要');
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
    const articleId = article.id || article.getAttribute('data-article-id');
    console.log('[ArticleSummary] getArticleData - articleId:', articleId);
    console.log('[ArticleSummary] getArticleData - article.id:', article.id);
    console.log('[ArticleSummary] getArticleData - article.getAttribute("data-article-id"):', article.getAttribute('data-article-id'));
    
    // 方法1: 从 articlesData 中查找文章（包含 summary）
    console.log('[ArticleSummary] 方法1: 从 articlesData 查找...');
    console.log('[ArticleSummary] window.articlesData:', window.articlesData);
    if (window.articlesData && window.articlesData.folders) {
      console.log('[ArticleSummary] articlesData 存在，文件夹数量:', window.articlesData.folders.length);
      console.log('[ArticleSummary] articlesData.folders:', window.articlesData.folders);
      
      for (const folder of window.articlesData.folders) {
        console.log('[ArticleSummary] 检查文件夹:', folder);
        const foundArticle = findArticleInFolder(folder, articleId);
        if (foundArticle) {
          console.log('[ArticleSummary] 从 articlesData 找到文章，包含摘要:', !!foundArticle.summary);
          console.log('[ArticleSummary] 找到的文章数据:', foundArticle);
          return foundArticle;
        }
      }
      console.log('[ArticleSummary] 从 articlesData 未找到文章');
    } else {
      console.log('[ArticleSummary] articlesData 不存在或没有文件夹');
    }
    
    // 方法2: 从 findArticleById 函数查找（passage.html 中的函数）
    console.log('[ArticleSummary] 方法2: 从 findArticleById 查找...');
    console.log('[ArticleSummary] typeof window.findArticleById:', typeof window.findArticleById);
    if (typeof window.findArticleById === 'function') {
      const articleData = window.findArticleById(articleId);
      console.log('[ArticleSummary] findArticleById 返回:', articleData);
      if (articleData) {
        console.log('[ArticleSummary] 从 findArticleById 找到文章，包含摘要:', !!articleData.summary);
        return articleData;
      }
      console.log('[ArticleSummary] 从 findArticleById 未找到文章');
    } else {
      console.log('[ArticleSummary] findArticleById 函数不存在');
    }

    // 方法3: 从缓存中获取文章数据
    console.log('[ArticleSummary] 方法3: 从 articleContentCache 查找...');
    console.log('[ArticleSummary] window.articleContentCache:', window.articleContentCache);
    if (window.articleContentCache && articleId) {
      console.log('[ArticleSummary] articleContentCache 存在，缓存大小:', window.articleContentCache.size);
      console.log('[ArticleSummary] 缓存键:', Array.from(window.articleContentCache.keys()));
      const cached = window.articleContentCache.get(articleId);
      console.log('[ArticleSummary] 缓存数据:', cached);
      if (cached) {
        // 如果缓存中有 articleData，直接返回
        if (cached.articleData) {
          console.log('[ArticleSummary] 从缓存找到 articleData，包含摘要:', !!cached.articleData.summary);
          return cached.articleData;
        }
        // 否则尝试从原始数据构建
        if (cached.summary) {
          console.log('[ArticleSummary] 从缓存找到摘要');
          return { summary: cached.summary };
        }
      }
      console.log('[ArticleSummary] 从缓存未找到数据');
    } else {
      console.log('[ArticleSummary] articleContentCache 不存在或没有 articleId');
    }

    // 方法4: 尝试从 data 属性获取
    console.log('[ArticleSummary] 方法4: 从 data-article 属性获取...');
    const dataStr = article.getAttribute('data-article');
    console.log('[ArticleSummary] data-article 属性值:', dataStr);
    if (dataStr) {
      try {
        const parsed = JSON.parse(dataStr);
        console.log('[ArticleSummary] 从 data-article 属性解析数据，包含摘要:', !!parsed.summary);
        return parsed;
      } catch (e) {
        console.error('[ArticleSummary] 解析文章数据失败:', e);
      }
    }
    console.log('[ArticleSummary] data-article 属性不存在');

    // 方法5: 从当前活动的文章数据中获取（passage.html 中的全局变量）
    console.log('[ArticleSummary] 方法5: 从 currentPassageData 获取...');
    console.log('[ArticleSummary] window.currentPassageData:', window.currentPassageData);
    if (window.currentPassageData && window.currentPassageData.summary) {
      console.log('[ArticleSummary] 从 currentPassageData 找到摘要');
      return window.currentPassageData;
    }
    console.log('[ArticleSummary] currentPassageData 不存在或没有摘要');

    console.log('[ArticleSummary] 所有方法都未能获取到文章数据');
    return null;
  }

  /**
   * 在文件夹中递归查找文章
   */
  function findArticleInFolder(folder, articleId) {
    console.log('[ArticleSummary] findArticleInFolder - 检查文件夹，articleId:', articleId);
    console.log('[ArticleSummary] findArticleInFolder - folder.articles:', folder.articles);
    
    // 检查文件夹中的文章
    if (folder.articles) {
      for (const article of folder.articles) {
        console.log('[ArticleSummary] findArticleInFolder - 检查文章:', article.id, '===', articleId, '?', article.id === articleId);
        if (article.id === articleId) {
          console.log('[ArticleSummary] findArticleInFolder - 找到匹配的文章:', article);
          return article;
        }
      }
    }
    
    // 递归检查子文件夹
    if (folder.folders) {
      console.log('[ArticleSummary] findArticleInFolder - 检查子文件夹，数量:', folder.folders.length);
      for (const subFolder of folder.folders) {
        const found = findArticleInFolder(subFolder, articleId);
        if (found) {
          console.log('[ArticleSummary] findArticleInFolder - 从子文件夹找到文章');
          return found;
        }
      }
    }
    
    console.log('[ArticleSummary] findArticleInFolder - 未找到文章');
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