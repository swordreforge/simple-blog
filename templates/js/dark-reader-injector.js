! function() {
  "use strict";
  // 全局捕获并忽略 Dark Reader 的图片加载错误
  window.addEventListener('unhandledrejection', function(event) {
    if (event.reason && event.reason.toString().includes('Unable to load image')) {
      event.preventDefault(); // 阻止错误显示在控制台
    }
  });
  const n = {
    brightness: 100,
    contrast: 100,
    grayscale: 0,
    sepia: 0,
    useFont: !1,
    textStroke: 0,
    scrollbarColor: "auto",
    selectionColor: "auto",
    styleSystemControls: !0,
    lightSchemeMatches: !1,
    darkSchemeMatches: !0,
    immediateFetch: !1,
    ignoreInlineStyle: ["*"],
    ignoreImageAnalysis: [],
    disableStyleSheetsProxy: !0,
    ignoreInlineAnalysis: ["*"],
    disablePDFViewer: !1,
    disableStyleSheets: !1,
    ignoreSelectors: [".nav-icon", ".nav-item", ".navigation", ".navbar", "nav", '[class*="nav"]', '[class*="menu"]', ".shortcut-hint", "svg", "img"],
    mode: 0
  };

  function t() {
    return "undefined" != typeof DarkReader && DarkReader.enable
  }

  function e() {
    if (t()) try {
      DarkReader.setFetchMethod(window.fetch), DarkReader.enable(n), console.log("[DarkReader] 暗色模式已启用"), a(), l("dark")
    } catch (n) {
      console.error("[DarkReader] 启用失败:", n), r()
    } else console.warn("[DarkReader] Dark Reader库未加载，使用备用方案"), r()
  }

  function o() {
    if (t()) try {
      DarkReader.disable(), console.log("[DarkReader] 暗色模式已禁用"), l("light")
    } catch (n) {
      console.error("[DarkReader] 禁用失败:", n)
    }
  }

  function r() {
    document.documentElement.style.setProperty("color-scheme", "dark"), document.documentElement.classList.add("dark-mode");
    const n = document.querySelector('meta[name="theme-color"]');
    n && n.setAttribute("content", "#000000"),
      function() {
        const n = "dark-reader-fallback-styles";
        let t = document.getElementById(n);
        t || (t = document.createElement("style"), t.id = n, document.head.appendChild(t)), t.textContent = a.toString().match(/\/[\*\s\S]*?\`\`/)[0].replace(/\/[\*\s\S]*?\`\`/, a.toString().match(/styleElement\.textContent = \`([\s\S]*)\`;/)[1])
      }(), console.log("[DarkReader] 已切换到备用暗色模式"), l("dark")
  }

  function a() {
    const n = "dark-reader-custom-styles";
    let t = document.getElementById(n);
    t || (t = document.createElement("style"), t.id = n, document.head.appendChild(t)), t.textContent = '\n            /* 增强暗色效果的自定义样式 */\n            html {\n                color-scheme: dark !important;\n            }\n            \n            body {\n                background-color: #1a1a1a !important;\n                color: #e0e0e0 !important;\n            }\n            \n            /* 保护导航元素的颜色，不被强制修改 */\n            nav, .navbar, .nav-item, .nav-icon,\n            [class*="nav-"], [class*="menu-"],\n            .navigation, .shortcut-hint {\n                color: inherit !important;\n                stroke: currentColor !important;\n                fill: none !important;\n            }\n\n            /* 为导航栏按钮应用毛玻璃效果 */\n            nav button, .navbar button,\n            .nav button, [class*="nav-"] button,\n            #loginBtn, #userCenterToggle,\n            .shortcuts-help-btn,\n            .user-center-item, .logout-item {\n                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;\n                backdrop-filter: blur(10px) !important;\n                -webkit-backdrop-filter: blur(10px) !important;\n                color: inherit !important;\n                border: 1px solid rgba(255, 255, 255, 0.1) !important;\n            }\n\n            /* 用户中心按钮悬停效果 */\n            .user-center-item:hover, .logout-item:hover {\n                background-color: rgba(80, 80, 80, 0.8) !important;\n                border-color: rgba(255, 255, 255, 0.3) !important;\n            }\n\n            /* 强制所有输入框使用暗色 */\n            input, textarea, select {\n                background-color: #2d2d2d !important;\n                color: #e0e0e0 !important;\n                border-color: #404040 !important;\n            }\n\n            /* 覆盖浏览器自动填充的黄色背景 */\n            input:-webkit-autofill,\n            input:-webkit-autofill:hover,\n            input:-webkit-autofill:focus,\n            textarea:-webkit-autofill,\n            textarea:-webkit-autofill:hover,\n            textarea:-webkit-autofill:focus,\n            select:-webkit-autofill,\n            select:-webkit-autofill:hover,\n            select:-webkit-autofill:focus {\n                -webkit-box-shadow: 0 0 0 30px #2d2d2d inset !important;\n                -webkit-text-fill-color: #e0e0e0 !important;\n                background-color: #2d2d2d !important;\n                color: #e0e0e0 !important;\n                transition: background-color 5000s ease-in-out 0s !important;\n            }\n\n            /* 为所有按钮应用透明毛玻璃效果 */\n            button {\n                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;\n                backdrop-filter: blur(10px) !important;\n                -webkit-backdrop-filter: blur(10px) !important;\n                color: inherit !important;\n                border: 1px solid rgba(255, 255, 255, 0.1) !important;\n                transition: all 0.3s ease !important;\n            }\n\n            /* 按钮悬停效果 */\n            button:hover {\n                background-color: rgba(80, 80, 80, 0.8) !important;\n                border-color: rgba(255, 255, 255, 0.3) !important;\n            }\n            \n            /* 强制所有表格使用暗色 */\n            table {\n                background-color: #252525 !important;\n                color: #e0e0e0 !important;\n            }\n            \n            table th {\n                background-color: #3d3d3d !important;\n                color: #e0e0e0 !important;\n            }\n            \n            table td {\n                border-color: #404040 !important;\n            }\n            \n            /* 只为实际内容区域的链接设置颜色，不影响导航 */\n            .article-content a, .content a, .post-content a {\n                color: #4a9eff !important;\n            }\n            \n            .article-content a:hover, .content a:hover, .post-content a:hover {\n                color: #3a8eef !important;\n            }\n            \n            /* 强制所有代码块使用暗色 */\n            pre, code {\n                background-color: #2d2d2d !important;\n                color: #e0e0e0 !important;\n            }\n            \n            /* 强制所有卡片使用暗色 */\n            .card, .panel {\n                background-color: #252525 !important;\n                color: #e0e0e0 !important;\n                border-color: #404040 !important;\n            }\n\n            /* 保留模态框的毛玻璃效果 */\n            .modal {\n                background-color: rgba(0, 0, 0, 0.5) !important;\n                color: #e0e0e0 !important;\n            }\n\n            /* 保留模态框内容的毛玻璃效果 */\n            .modal-content {\n                background: rgba(0, 0, 0, 0) !important;\n                backdrop-filter: blur(40px) saturate(200%) !important;\n                -webkit-backdrop-filter: blur(40px) saturate(200%) !important;\n                border: 1px solid rgba(255, 255, 255, 0.5) !important;\n                color: #e0e0e0 !important;\n            }\n            \n            /* 强制导航栏使用暗色背景，但保留毛玻璃效果 */\n            nav, .navbar, header {\n                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;\n                backdrop-filter: blur(10px) !important;\n                -webkit-backdrop-filter: blur(10px) !important;\n                border-color: rgba(255, 255, 255, 0.1) !important;\n            }\n            \n            /* 强制侧边栏使用暗色 */\n            aside, .sidebar {\n                background-color: #252525 !important;\n                color: #e0e0e0 !important;\n            }\n            \n            /* 强制页脚使用暗色但保留毛玻璃效果 */\n            footer {\n                background-color: var(--footer-glass-color, rgba(45, 45, 45, 0.6)) !important;\n                backdrop-filter: blur(10px) !important;\n                -webkit-backdrop-filter: blur(10px) !important;\n                color: #ffffff !important;\n                text-shadow: 0 0 10px rgba(255, 255, 255, 0.5),\n                             0 0 20px rgba(255, 255, 255, 0.3),\n                             0 0 30px rgba(255, 255, 255, 0.1) !important;\n                border-color: rgba(255, 255, 255, 0.1) !important;\n            }\n            \n            /* 强制滚动条使用暗色 */\n            ::-webkit-scrollbar {\n                background-color: #2d2d2d !important;\n            }\n            \n            ::-webkit-scrollbar-thumb {\n                background-color: #4a4a4a !important;\n            }\n        '
  }

  function l(n) {
    const t = new CustomEvent("dark-reader-theme-change", {
      detail: {
        theme: n
      }
    });
    document.dispatchEvent(t)
  }
  async function i() {
    try {
      await new Promise((n, e) => {
        if (t()) return void n();
        const o = document.createElement("script");
        o.src = "/js/npm/darkreader@4.9.92/darkreader.min.js", o.onload = n, o.onerror = e, document.head.appendChild(o)
      }), e(), new MutationObserver(e => {
        e.forEach(e => {
          "childList" === e.type && e.addedNodes.forEach(e => {
            e.nodeType === Node.ELEMENT_NODE && t() && (DarkReader.setFetchMethod(window.fetch), DarkReader.enable(n))
          })
        })
      }).observe(document.body, {
        childList: !0,
        subtree: !0
      }), window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", n => {
        console.log("[DarkReader] 系统主题变为: " + (n.matches ? "dark" : "light"))
      })
    } catch (n) {
      console.error("[DarkReader] 初始化失败:", n), r()
    }
  }
  window.DarkReaderInjector = {
    enable: e,
    disable: o,
    toggle: () => {
      t() && DarkReader.isEnabled() ? o() : e()
    },
    isAvailable: t,
    fallback: r
  }, "loading" === document.readyState ? document.addEventListener("DOMContentLoaded", i) : i(), console.log("[DarkReader] 专业暗色模式注入器已加载")
}();