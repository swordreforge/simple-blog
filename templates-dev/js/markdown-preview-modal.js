!(function () {
  let n = null,
    e = null,
    t = null,
    o = null,
    i = null,
    r = !1;
  function a() {
    var r, a;
    n ||
      (((n = document.createElement('div')).id = 'markdown-preview-modal'),
      (n.className = 'markdown-preview-modal'),
      (n.style.cssText =
        '\n      position: fixed;\n      top: 0;\n      left: 0;\n      width: 100%;\n      height: 100%;\n      background: rgba(0, 0, 0, 0.5);\n      backdrop-filter: blur(5px);\n      z-index: 10000;\n      display: none;\n      align-items: center;\n      justify-content: center;\n      opacity: 0;\n      transition: opacity 0.3s ease;\n    '),
      ((e = document.createElement('div')).className = 'markdown-preview-content'),
      (e.style.cssText =
        '\n      background: white;\n      width: 90%;\n      max-width: 800px;\n      max-height: 80vh;\n      border-radius: 12px;\n      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);\n      overflow: hidden;\n      transform: scale(0.9);\n      transition: transform 0.3s ease;\n      display: flex;\n      flex-direction: column;\n    '),
      ((r = document.createElement('div')).className = 'markdown-preview-header'),
      (r.style.cssText =
        '\n      padding: 16px 20px;\n      border-bottom: 1px solid #e0e0e0;\n      display: flex;\n      justify-content: space-between;\n      align-items: center;\n      background: #f5f5f5;\n    '),
      ((o = document.createElement('h3')).className = 'markdown-preview-title'),
      (o.style.cssText =
        '\n      margin: 0;\n      font-size: 18px;\n      font-weight: 600;\n      color: #333;\n    '),
      ((t = document.createElement('button')).className = 'markdown-preview-close'),
      (t.innerHTML = '×'),
      (t.style.cssText =
        '\n      background: none;\n      border: none;\n      font-size: 24px;\n      color: #666;\n      cursor: pointer;\n      width: 32px;\n      height: 32px;\n      display: flex;\n      align-items: center;\n      justify-content: center;\n      border-radius: 50%;\n      transition: all 0.2s ease;\n    '),
      t.addEventListener('mouseenter', () => {
        ((t.style.background = '#e0e0e0'), (t.style.color = '#333'));
      }),
      t.addEventListener('mouseleave', () => {
        ((t.style.background = 'none'), (t.style.color = '#666'));
      }),
      r.appendChild(o),
      r.appendChild(t),
      ((i = document.createElement('div')).className = 'markdown-preview-body'),
      (i.style.cssText =
        "\n      padding: 20px;\n      overflow-y: auto;\n      flex: 1;\n      font-family: 'Segoe UI', 'Helvetica Neue', 'PingFang SC', 'Microsoft YaHei', sans-serif;\n      line-height: 1.6;\n      color: #333;\n    "),
      ((a = document.createElement('div')).className = 'markdown-preview-loading'),
      (a.innerHTML =
        '\n      <div style="\n        display: flex;\n        flex-direction: column;\n        align-items: center;\n        justify-content: center;\n        height: 200px;\n        color: #666;\n      ">\n        <div style="\n          width: 40px;\n          height: 40px;\n          border: 3px solid #f3f3f3;\n          border-top: 3px solid #3498db;\n          border-radius: 50%;\n          animation: spin 1s linear infinite;\n          margin-bottom: 12px;\n        "></div>\n        <div style="font-size: 14px;">加载中...</div>\n      </div>\n      <style>\n        @keyframes spin {\n          0% { transform: rotate(0deg); }\n          100% { transform: rotate(360deg); }\n        }\n      </style>\n    '),
      e.appendChild(r),
      e.appendChild(i),
      n.appendChild(e),
      document.body.appendChild(n),
      t.addEventListener('click', d),
      n.addEventListener('click', e => {
        e.target === n && d();
      }),
      document.addEventListener('keydown', e => {
        'Escape' === e.key && 'flex' === n.style.display && d();
      }));
  }
  function d() {
    n &&
      ((n.style.opacity = '0'),
      (e.style.transform = 'scale(0.9)'),
      setTimeout(() => {
        ((n.style.display = 'none'), (i.innerHTML = ''));
      }, 300));
  }
  ((window.MarkdownPreviewModal = {
    open: async function (t) {
      if (!r) {
        (a(), (r = !0), (i.innerHTML = ''));
        var d = document.createElement('div');
        ((d.className = 'markdown-preview-loading'),
          (d.innerHTML =
            '\n      <div style="\n        display: flex;\n        flex-direction: column;\n        align-items: center;\n        justify-content: center;\n        height: 200px;\n        color: #666;\n      ">\n        <div style="\n          width: 40px;\n          height: 40px;\n          border: 3px solid #f3f3f3;\n          border-top: 3px solid #3498db;\n          border-radius: 50%;\n          animation: spin 1s linear infinite;\n          margin-bottom: 12px;\n        "></div>\n        <div style="font-size: 14px;">加载中...</div>\n      </div>\n      <style>\n        @keyframes spin {\n          0% { transform: rotate(0deg); }\n          100% { transform: rotate(360deg); }\n        }\n      </style>\n    '),
          i.appendChild(d),
          (n.style.display = 'flex'),
          requestAnimationFrame(() => {
            ((n.style.opacity = '1'), (e.style.transform = 'scale(1)'));
          }));
        try {
          var l = await (await fetch('/api/markdown/preview?path=' + encodeURIComponent(t))).json();
          if (!l.success) throw new Error(l.message || '加载失败');
          var s,
            p = l.data,
            c =
              ((o.textContent = p.title),
              (function () {
                let n = p.content;
                return (n = (n =
                  (n = `<p style="margin: 10px 0;">${(n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = (n = n.replace(/&/g, '&amp;')).replace(/</g, '&lt;')).replace(/>/g, '&gt;')).replace(/^### (.*$)/gim, '<h3>$1</h3>')).replace(/^## (.*$)/gim, '<h2>$1</h2>')).replace(/^# (.*$)/gim, '<h1>$1</h1>')).replace(/\*\*(.*?)\*\*/gim, '<strong>$1</strong>')).replace(/\*(.*?)\*/gim, '<em>$1</em>')).replace(/\[([^\]]+)\]\(([^)]+)\)/gim, '<a href="$2" target="_blank">$1</a>')).replace(/!\[([^\]]*)\]\(([^)]+)\)/gim, '<img src="$2" alt="$1" style="max-width: 100%; height: auto; border-radius: 4px; margin: 10px 0;">')).replace(/```(\w+)?\n([\s\S]*?)```/gim, '<pre><code>$2</code></pre>')).replace(/`([^`]+)`/gim, '<code style="background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: monospace;">$1</code>')).replace(/^> (.*$)/gim, '<blockquote style="border-left: 4px solid #ddd; padding-left: 16px; margin: 10px 0; color: #666;">$1</blockquote>')).replace(/^---$/gim, '<hr style="border: none; border-top: 1px solid #ddd; margin: 20px 0;">')).replace(/^\- (.*$)/gim, '<li style="margin: 4px 0;">$1</li>')).replace(/^(\d+)\. (.*$)/gim, '<li style="margin: 4px 0;">$2</li>')).replace(/\n\n/g, '</p><p style="margin: 10px 0;">')).replace(/\n/g, '<br>'))}</p>`).replace(
                    /<li>/g,
                    '<ul style="margin: 10px 0; padding-left: 20px;"><li>'
                  )).replace(/<\/li>/g, '</li></ul>')).replace(/<\/ul><ul>/g, '');
              })());
          ((i.innerHTML = c),
            document.getElementById('markdown-preview-styles') ||
              (((s = document.createElement('style')).id = 'markdown-preview-styles'),
              (s.textContent =
                "\n        .markdown-preview-body h1,\n        .markdown-preview-body h2,\n        .markdown-preview-body h3 {\n          margin-top: 20px;\n          margin-bottom: 10px;\n          color: #333;\n          font-weight: 600;\n        }\n\n        .markdown-preview-body h1 {\n          font-size: 24px;\n          border-bottom: 2px solid #e0e0e0;\n          padding-bottom: 10px;\n        }\n\n        .markdown-preview-body h2 {\n          font-size: 20px;\n        }\n\n        .markdown-preview-body h3 {\n          font-size: 18px;\n        }\n\n        .markdown-preview-body p {\n          margin: 10px 0;\n          line-height: 1.6;\n        }\n\n        .markdown-preview-body a {\n          color: #007bff;\n          text-decoration: none;\n        }\n\n        .markdown-preview-body a:hover {\n          text-decoration: underline;\n        }\n\n        .markdown-preview-body pre {\n          background: #f4f4f4;\n          padding: 16px;\n          border-radius: 4px;\n          overflow-x: auto;\n          margin: 10px 0;\n        }\n\n        .markdown-preview-body code {\n          font-family: 'Consolas', 'Monaco', monospace;\n          font-size: 14px;\n        }\n\n        .markdown-preview-body blockquote {\n          border-left: 4px solid #007bff;\n          padding-left: 16px;\n          margin: 10px 0;\n          color: #666;\n          font-style: italic;\n        }\n\n        .markdown-preview-body ul,\n        .markdown-preview-body ol {\n          margin: 10px 0;\n          padding-left: 20px;\n        }\n\n        .markdown-preview-body li {\n          margin: 4px 0;\n        }\n\n        .markdown-preview-body hr {\n          border: none;\n          border-top: 1px solid #e0e0e0;\n          margin: 20px 0;\n        }\n\n        .markdown-preview-body img {\n          max-width: 100%;\n          height: auto;\n          border-radius: 4px;\n          margin: 10px 0;\n        }\n      "),
              document.head.appendChild(s)));
        } catch (t) {
          (console.error('Failed to load markdown:', t),
            (i.innerHTML = `\n        <div style="\n          display: flex;\n          flex-direction: column;\n          align-items: center;\n          justify-content: center;\n          height: 200px;\n          color: #e74c3c;\n        ">\n          <div style="font-size: 48px; margin-bottom: 12px;">⚠️</div>\n          <div style="font-size: 14px; font-weight: 600;">加载失败</div>\n          <div style="font-size: 12px; color: #666; margin-top: 4px;">${t.message}</div>\n        </div>\n      `));
        } finally {
          r = !1;
        }
      }
    },
    close: d,
  }),
    'loading' === document.readyState
      ? document.addEventListener('DOMContentLoaded', () => {
          a();
        })
      : a());
})();
