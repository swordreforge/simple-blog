// Markdown Preview Modal
(function() {
  // Modal element
  let modal = null;
  let modalContent = null;
  let modalClose = null;
  let modalTitle = null;
  let modalBody = null;
  let isLoading = false;

  // Initialize modal
  function initModal() {
    if (modal) return;

    // Create modal structure
    modal = document.createElement('div');
    modal.id = 'markdown-preview-modal';
    modal.className = 'markdown-preview-modal';
    modal.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(5px);
      z-index: 10000;
      display: none;
      align-items: center;
      justify-content: center;
      opacity: 0;
      transition: opacity 0.3s ease;
    `;

    modalContent = document.createElement('div');
    modalContent.className = 'markdown-preview-content';
    modalContent.style.cssText = `
      background: white;
      width: 90%;
      max-width: 800px;
      max-height: 80vh;
      border-radius: 12px;
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
      overflow: hidden;
      transform: scale(0.9);
      transition: transform 0.3s ease;
      display: flex;
      flex-direction: column;
    `;

    // Modal header
    const modalHeader = document.createElement('div');
    modalHeader.className = 'markdown-preview-header';
    modalHeader.style.cssText = `
      padding: 16px 20px;
      border-bottom: 1px solid #e0e0e0;
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: #f5f5f5;
    `;

    modalTitle = document.createElement('h3');
    modalTitle.className = 'markdown-preview-title';
    modalTitle.style.cssText = `
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      color: #333;
    `;

    modalClose = document.createElement('button');
    modalClose.className = 'markdown-preview-close';
    modalClose.innerHTML = '×';
    modalClose.style.cssText = `
      background: none;
      border: none;
      font-size: 24px;
      color: #666;
      cursor: pointer;
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 50%;
      transition: all 0.2s ease;
    `;

    modalClose.addEventListener('mouseenter', () => {
      modalClose.style.background = '#e0e0e0';
      modalClose.style.color = '#333';
    });

    modalClose.addEventListener('mouseleave', () => {
      modalClose.style.background = 'none';
      modalClose.style.color = '#666';
    });

    modalHeader.appendChild(modalTitle);
    modalHeader.appendChild(modalClose);

    // Modal body
    modalBody = document.createElement('div');
    modalBody.className = 'markdown-preview-body';
    modalBody.style.cssText = `
      padding: 20px;
      overflow-y: auto;
      flex: 1;
      font-family: 'Segoe UI', 'Helvetica Neue', 'PingFang SC', 'Microsoft YaHei', sans-serif;
      line-height: 1.6;
      color: #333;
    `;

    // Loading indicator
    const loadingIndicator = document.createElement('div');
    loadingIndicator.className = 'markdown-preview-loading';
    loadingIndicator.innerHTML = `
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 200px;
        color: #666;
      ">
        <div style="
          width: 40px;
          height: 40px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #3498db;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 12px;
        "></div>
        <div style="font-size: 14px;">加载中...</div>
      </div>
      <style>
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      </style>
    `;

    modalContent.appendChild(modalHeader);
    modalContent.appendChild(modalBody);
    modal.appendChild(modalContent);
    document.body.appendChild(modal);

    // Event listeners
    modalClose.addEventListener('click', closeModal);
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        closeModal();
      }
    });

    // Escape key to close
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && modal.style.display === 'flex') {
        closeModal();
      }
    });
  }

  // Open modal with markdown content
  async function openModal(markdownPath) {
    if (isLoading) return;

    initModal();
    isLoading = true;

    // Show loading state
    modalBody.innerHTML = '';
    const loadingIndicator = document.createElement('div');
    loadingIndicator.className = 'markdown-preview-loading';
    loadingIndicator.innerHTML = `
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 200px;
        color: #666;
      ">
        <div style="
          width: 40px;
          height: 40px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #3498db;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 12px;
        "></div>
        <div style="font-size: 14px;">加载中...</div>
      </div>
      <style>
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      </style>
    `;
    modalBody.appendChild(loadingIndicator);

    // Show modal
    modal.style.display = 'flex';
    requestAnimationFrame(() => {
      modal.style.opacity = '1';
      modalContent.style.transform = 'scale(1)';
    });

    try {
      // Fetch markdown content
      const response = await fetch(`/api/markdown/preview?path=${encodeURIComponent(markdownPath)}`);
      const result = await response.json();

      if (!result.success) {
        throw new Error(result.message || '加载失败');
      }

      const data = result.data;

      // Update title
      modalTitle.textContent = data.title;

      // Parse markdown to HTML (simple parser)
      const htmlContent = parseMarkdown(data.content);

      // Update content
      modalBody.innerHTML = htmlContent;

      // Add styles for markdown content
      addMarkdownStyles();

    } catch (error) {
      console.error('Failed to load markdown:', error);
      modalBody.innerHTML = `
        <div style="
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
          color: #e74c3c;
        ">
          <div style="font-size: 48px; margin-bottom: 12px;">⚠️</div>
          <div style="font-size: 14px; font-weight: 600;">加载失败</div>
          <div style="font-size: 12px; color: #666; margin-top: 4px;">${error.message}</div>
        </div>
      `;
    } finally {
      isLoading = false;
    }
  }

  // Close modal
  function closeModal() {
    if (!modal) return;

    modal.style.opacity = '0';
    modalContent.style.transform = 'scale(0.9)';

    setTimeout(() => {
      modal.style.display = 'none';
      modalBody.innerHTML = '';
    }, 300);
  }

  // Simple markdown parser
  function parseMarkdown(markdown) {
    let html = markdown;

    // Escape HTML
    html = html.replace(/&/g, '&amp;');
    html = html.replace(/</g, '&lt;');
    html = html.replace(/>/g, '&gt;');

    // Headers
    html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
    html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
    html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');

    // Bold
    html = html.replace(/\*\*(.*?)\*\*/gim, '<strong>$1</strong>');

    // Italic
    html = html.replace(/\*(.*?)\*/gim, '<em>$1</em>');

    // Links
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/gim, '<a href="$2" target="_blank">$1</a>');

    // Images
    html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/gim, '<img src="$2" alt="$1" style="max-width: 100%; height: auto; border-radius: 4px; margin: 10px 0;">');

    // Code blocks
    html = html.replace(/```(\w+)?\n([\s\S]*?)```/gim, '<pre><code>$2</code></pre>');

    // Inline code
    html = html.replace(/`([^`]+)`/gim, '<code style="background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: monospace;">$1</code>');

    // Blockquotes
    html = html.replace(/^> (.*$)/gim, '<blockquote style="border-left: 4px solid #ddd; padding-left: 16px; margin: 10px 0; color: #666;">$1</blockquote>');

    // Horizontal rules
    html = html.replace(/^---$/gim, '<hr style="border: none; border-top: 1px solid #ddd; margin: 20px 0;">');

    // Lists
    html = html.replace(/^\- (.*$)/gim, '<li style="margin: 4px 0;">$1</li>');
    html = html.replace(/^(\d+)\. (.*$)/gim, '<li style="margin: 4px 0;">$2</li>');

    // Paragraphs (double line breaks)
    html = html.replace(/\n\n/g, '</p><p style="margin: 10px 0;">');

    // Single line breaks
    html = html.replace(/\n/g, '<br>');

    // Wrap in paragraph
    html = `<p style="margin: 10px 0;">${html}</p>`;

    // Fix list styling
    html = html.replace(/<li>/g, '<ul style="margin: 10px 0; padding-left: 20px;"><li>');
    html = html.replace(/<\/li>/g, '</li></ul>');
    html = html.replace(/<\/ul><ul>/g, '');

    return html;
  }

  // Add markdown styles
  function addMarkdownStyles() {
    if (!document.getElementById('markdown-preview-styles')) {
      const style = document.createElement('style');
      style.id = 'markdown-preview-styles';
      style.textContent = `
        .markdown-preview-body h1,
        .markdown-preview-body h2,
        .markdown-preview-body h3 {
          margin-top: 20px;
          margin-bottom: 10px;
          color: #333;
          font-weight: 600;
        }

        .markdown-preview-body h1 {
          font-size: 24px;
          border-bottom: 2px solid #e0e0e0;
          padding-bottom: 10px;
        }

        .markdown-preview-body h2 {
          font-size: 20px;
        }

        .markdown-preview-body h3 {
          font-size: 18px;
        }

        .markdown-preview-body p {
          margin: 10px 0;
          line-height: 1.6;
        }

        .markdown-preview-body a {
          color: #007bff;
          text-decoration: none;
        }

        .markdown-preview-body a:hover {
          text-decoration: underline;
        }

        .markdown-preview-body pre {
          background: #f4f4f4;
          padding: 16px;
          border-radius: 4px;
          overflow-x: auto;
          margin: 10px 0;
        }

        .markdown-preview-body code {
          font-family: 'Consolas', 'Monaco', monospace;
          font-size: 14px;
        }

        .markdown-preview-body blockquote {
          border-left: 4px solid #007bff;
          padding-left: 16px;
          margin: 10px 0;
          color: #666;
          font-style: italic;
        }

        .markdown-preview-body ul,
        .markdown-preview-body ol {
          margin: 10px 0;
          padding-left: 20px;
        }

        .markdown-preview-body li {
          margin: 4px 0;
        }

        .markdown-preview-body hr {
          border: none;
          border-top: 1px solid #e0e0e0;
          margin: 20px 0;
        }

        .markdown-preview-body img {
          max-width: 100%;
          height: auto;
          border-radius: 4px;
          margin: 10px 0;
        }
      `;
      document.head.appendChild(style);
    }
  }

  // Export to global scope
  window.MarkdownPreviewModal = {
    open: openModal,
    close: closeModal
  };

  // Auto-initialize when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      initModal();
    });
  } else {
    initModal();
  }
})();