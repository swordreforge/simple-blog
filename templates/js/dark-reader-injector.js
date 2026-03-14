/**
 * Dark Reader 专业暗色模式注入
 * 使用Dark Reader库的API来提供完整的暗色模式支持
 */

(function() {
    'use strict';

    // Dark Reader 配置
    const darkReaderConfig = {
        brightness: 100,
        contrast: 100, // 提高对比度到100，避免颜色偏差
        grayscale: 0,
        sepia: 0,
        useFont: false,
        textStroke: 0,
        scrollbarColor: 'auto',
        selectionColor: 'auto',
        styleSystemControls: true,
        lightSchemeMatches: false, // 始终使用暗色模式
        darkSchemeMatches: true,
        immediateFetch: true,
        ignoreInlineStyle: ['*'], // 忽略所有内联样式，避免强制颜色修改
        ignoreImageAnalysis: [],
        disableStyleSheetsProxy: true, // 禁用样式代理，保留原始CSS
        ignoreInlineAnalysis: ['*'], // 忽略内联分析
        disablePDFViewer: false,
        // 保留原始颜色，不要强制修改
        disableStyleSheets: false,
        // 忽略特定元素的颜色修改
        ignoreSelectors: [
            '.nav-icon',
            '.nav-item',
            '.navigation',
            '.navbar',
            'nav',
            '[class*="nav"]',
            '[class*="menu"]',
            '.shortcut-hint',
            'svg'
        ]
    };

    // 检查Dark Reader是否已加载
    function isDarkReaderAvailable() {
        return typeof DarkReader !== 'undefined' && DarkReader.enable;
    }

    // 应用暗色模式
    function applyDarkMode() {
        if (isDarkReaderAvailable()) {
            try {
                DarkReader.enable(darkReaderConfig);
                console.log('[DarkReader] 暗色模式已启用');
                
                // 添加自定义CSS以增强暗色效果
                addCustomDarkStyles();
                
                // 触发事件通知
                dispatchThemeEvent('dark');
            } catch (error) {
                console.error('[DarkReader] 启用失败:', error);
                fallbackToDarkMode();
            }
        } else {
            console.warn('[DarkReader] Dark Reader库未加载，使用备用方案');
            fallbackToDarkMode();
        }
    }

    // 移除暗色模式
    function removeDarkMode() {
        if (isDarkReaderAvailable()) {
            try {
                DarkReader.disable();
                console.log('[DarkReader] 暗色模式已禁用');
                dispatchThemeEvent('light');
            } catch (error) {
                console.error('[DarkReader] 禁用失败:', error);
            }
        }
    }

    // 备用暗色模式方案
    function fallbackToDarkMode() {
        // 强制设置颜色方案
        document.documentElement.style.setProperty('color-scheme', 'dark');
        
        // 添加dark-mode类
        document.documentElement.classList.add('dark-mode');
        
        // 设置meta标签
        const themeColor = document.querySelector('meta[name="theme-color"]');
        if (themeColor) {
            themeColor.setAttribute('content', '#000000');
        }
        
        // 添加备用样式
        addFallbackStyles();
        
        console.log('[DarkReader] 已切换到备用暗色模式');
        dispatchThemeEvent('dark');
    }

    // 添加自定义暗色样式
    function addCustomDarkStyles() {
        const styleId = 'dark-reader-custom-styles';
        let styleElement = document.getElementById(styleId);
        
        if (!styleElement) {
            styleElement = document.createElement('style');
            styleElement.id = styleId;
            document.head.appendChild(styleElement);
        }
        
        styleElement.textContent = `
            /* 增强暗色效果的自定义样式 */
            html {
                color-scheme: dark !important;
            }
            
            body {
                background-color: #1a1a1a !important;
                color: #e0e0e0 !important;
            }
            
            /* 保护导航元素的颜色，不被强制修改 */
            nav, .navbar, .nav-item, .nav-icon,
            [class*="nav-"], [class*="menu-"],
            .navigation, .shortcut-hint {
                color: inherit !important;
                stroke: currentColor !important;
                fill: none !important;
            }

            /* 为导航栏按钮应用毛玻璃效果 */
            nav button, .navbar button,
            .nav button, [class*="nav-"] button,
            #loginBtn, #userCenterToggle,
            .shortcuts-help-btn {
                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;
                backdrop-filter: blur(10px) !important;
                -webkit-backdrop-filter: blur(10px) !important;
                color: inherit !important;
                border: 1px solid rgba(255, 255, 255, 0.1) !important;
            }

            /* 强制所有输入框使用暗色 */
            input, textarea, select {
                background-color: #2d2d2d !important;
                color: #e0e0e0 !important;
                border-color: #404040 !important;
            }

            /* 强制非导航栏按钮使用暗色 */
            button:not(nav button):not(.navbar button):not(.nav button):not([class*="nav-"] button):not(#loginBtn):not(#userCenterToggle):not(.shortcuts-help-btn) {
                background-color: #3d3d3d !important;
                color: #e0e0e0 !important;
            }
            
            /* 强制所有表格使用暗色 */
            table {
                background-color: #252525 !important;
                color: #e0e0e0 !important;
            }
            
            table th {
                background-color: #3d3d3d !important;
                color: #e0e0e0 !important;
            }
            
            table td {
                border-color: #404040 !important;
            }
            
            /* 只为实际内容区域的链接设置颜色，不影响导航 */
            .article-content a, .content a, .post-content a {
                color: #4a9eff !important;
            }
            
            .article-content a:hover, .content a:hover, .post-content a:hover {
                color: #3a8eef !important;
            }
            
            /* 强制所有代码块使用暗色 */
            pre, code {
                background-color: #2d2d2d !important;
                color: #e0e0e0 !important;
            }
            
            /* 强制所有卡片使用暗色 */
            .card, .panel {
                background-color: #252525 !important;
                color: #e0e0e0 !important;
                border-color: #404040 !important;
            }

            /* 保留模态框的毛玻璃效果 */
            .modal {
                background-color: rgba(0, 0, 0, 0.5) !important;
                color: #e0e0e0 !important;
            }

            /* 保留模态框内容的毛玻璃效果 */
            .modal-content {
                background: rgba(0, 0, 0, 0) !important;
                backdrop-filter: blur(40px) saturate(200%) !important;
                -webkit-backdrop-filter: blur(40px) saturate(200%) !important;
                border: 1px solid rgba(255, 255, 255, 0.5) !important;
                color: #e0e0e0 !important;
            }
            
            /* 强制导航栏使用暗色背景，但不改变文字颜色 */
            nav, .navbar, header {
                background-color: rgba(26, 26, 26, 0.95) !important;
                border-color: #404040 !important;
            }
            
            /* 强制侧边栏使用暗色 */
            aside, .sidebar {
                background-color: #252525 !important;
                color: #e0e0e0 !important;
            }
            
            /* 强制页脚使用暗色但保留毛玻璃效果 */
            footer {
                background-color: var(--footer-glass-color, rgba(45, 45, 45, 0.6)) !important;
                backdrop-filter: blur(10px) !important;
                -webkit-backdrop-filter: blur(10px) !important;
                color: #e0e0e0 !important;
                border-color: rgba(255, 255, 255, 0.1) !important;
            }
            
            /* 强制滚动条使用暗色 */
            ::-webkit-scrollbar {
                background-color: #2d2d2d !important;
            }
            
            ::-webkit-scrollbar-thumb {
                background-color: #4a4a4a !important;
            }
        `;
    }

    // 添加备用样式
    function addFallbackStyles() {
        const styleId = 'dark-reader-fallback-styles';
        let styleElement = document.getElementById(styleId);
        
        if (!styleElement) {
            styleElement = document.createElement('style');
            styleElement.id = styleId;
            document.head.appendChild(styleElement);
        }
        
        styleElement.textContent = addCustomDarkStyles.toString().match(/\/[\*\s\S]*?\`\`/)[0].replace(/\/[\*\s\S]*?\`\`/, addCustomDarkStyles.toString().match(/styleElement\.textContent = \`([\s\S]*)\`;/)[1]);
    }

    // 触发主题变化事件
    function dispatchThemeEvent(theme) {
        const event = new CustomEvent('dark-reader-theme-change', {
            detail: { theme: theme }
        });
        document.dispatchEvent(event);
    }

    // 动态加载Dark Reader库
    function loadDarkReaderLibrary() {
        return new Promise((resolve, reject) => {
            if (isDarkReaderAvailable()) {
                resolve();
                return;
            }
            
            const script = document.createElement('script');
            script.src = '/js/npm/darkreader@4.9.92/darkreader.min.js';
            script.onload = resolve;
            script.onerror = reject;
            document.head.appendChild(script);
        });
    }

    // 初始化函数
    async function init() {
        try {
            // 加载Dark Reader库
            await loadDarkReaderLibrary();
            
            // 应用暗色模式
            applyDarkMode();
            
            // 监听DOM变化
            observeDOMChanges();
            
            // 监听系统主题变化
            observeSystemTheme();
            
        } catch (error) {
            console.error('[DarkReader] 初始化失败:', error);
            fallbackToDarkMode();
        }
    }

    // 监听DOM变化
    function observeDOMChanges() {
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'childList') {
                    mutation.addedNodes.forEach((node) => {
                        if (node.nodeType === Node.ELEMENT_NODE) {
                            // 确保新元素也应用暗色样式
                            if (isDarkReaderAvailable()) {
                                DarkReader.enable(darkReaderConfig);
                            }
                        }
                    });
                }
            });
        });
        
        observer.observe(document.body, {
            childList: true,
            subtree: true
        });
    }

    // 监听系统主题变化
    function observeSystemTheme() {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        
        mediaQuery.addEventListener('change', (e) => {
            // 我们始终强制使用暗色模式，所以这里只是记录
            console.log(`[DarkReader] 系统主题变为: ${e.matches ? 'dark' : 'light'}`);
        });
    }

    // 暴露全局API
    window.DarkReaderInjector = {
        enable: applyDarkMode,
        disable: removeDarkMode,
        toggle: () => {
            if (isDarkReaderAvailable() && DarkReader.isEnabled()) {
                removeDarkMode();
            } else {
                applyDarkMode();
            }
        },
        isAvailable: isDarkReaderAvailable,
        fallback: fallbackToDarkMode
    };

    // 立即初始化
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    console.log('[DarkReader] 专业暗色模式注入器已加载');
})();
