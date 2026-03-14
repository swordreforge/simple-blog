/**
 * BlackMagic 自动注入暗色模式
 * 使用BlackMagic库自动应用暗色模式，一加载就自动生效
 */

(function() {
    'use strict';

    // BlackMagic 配置对象
    const BlackMagicConfig = {
        autoSwitch: true,  // 自动切换，一加载就应用暗色模式
        theme: 'dark',     // 强制使用暗色主题
        preserveColors: {
            // 保留某些颜色的原始值
            '--accent-color': true,
            '--brand-color': true
        },
        overrideColors: {
            // 强制覆盖某些颜色
            '--bg-primary': '#1a1a1a',
            '--bg-secondary': '#252525',
            '--text-primary': '#e0e0e0',
            '--text-secondary': '#a0a0a0',
            '--border-color': '#404040'
        }
    };

    // BlackMagic 核心类
    class BlackMagic {
        constructor(config) {
            this.config = config || {};
            this.originalTheme = document.documentElement.getAttribute('data-theme') || 'light';
            this.currentTheme = this.originalTheme;
            this.observers = [];
            this.init();
        }

        init() {
            // 如果配置了自动切换，立即应用主题
            if (this.config.autoSwitch) {
                this.applyTheme(this.config.theme || 'dark');
            }

            // 监听DOM变化，动态应用样式
            this.observeDOMChanges();
        }

        applyTheme(theme) {
            this.currentTheme = theme;
            
            // 设置 color-scheme
            document.documentElement.style.setProperty('color-scheme', theme);
            
            // 添加/移除 dark-mode 类
            if (theme === 'dark') {
                document.documentElement.classList.add('dark-mode');
                document.documentElement.classList.remove('light-mode');
            } else {
                document.documentElement.classList.add('light-mode');
                document.documentElement.classList.remove('dark-mode');
            }

            // 设置 theme-color meta 标签
            this.updateThemeColor(theme);

            // 应用自定义颜色覆盖
            if (this.config.overrideColors) {
                this.applyColorOverrides();
            }

            // 触发主题变化事件
            this.dispatchThemeChangeEvent(theme);
        }

        updateThemeColor(theme) {
            const themeColor = document.querySelector('meta[name="theme-color"]');
            if (themeColor) {
                themeColor.setAttribute('content', theme === 'dark' ? '#000000' : '#ffffff');
            }
        }

        applyColorOverrides() {
            const overrides = this.config.overrideColors;
            if (!overrides) return;

            for (const [property, value] of Object.entries(overrides)) {
                // 检查是否需要保留原始颜色
                if (this.config.preserveColors && this.config.preserveColors[property]) {
                    continue;
                }
                
                // 检查是否已经设置了该属性
                if (!getComputedStyle(document.documentElement).getPropertyValue(property).trim()) {
                    document.documentElement.style.setProperty(property, value);
                }
            }
        }

        observeDOMChanges() {
            const observer = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    if (mutation.type === 'childList') {
                        // 处理新增的元素
                        mutation.addedNodes.forEach((node) => {
                            if (node.nodeType === Node.ELEMENT_NODE) {
                                this.applyThemeToElement(node);
                            }
                        });
                    }
                });
            });

            observer.observe(document.body, {
                childList: true,
                subtree: true
            });

            this.observers.push(observer);
        }

        applyThemeToElement(element) {
            // 对特定元素应用暗色模式样式
            if (this.currentTheme === 'dark') {
                // 处理表格
                if (element.tagName === 'TABLE') {
                    element.style.setProperty('--table-bg', '#252525');
                    element.style.setProperty('--table-header-bg', '#333333');
                    element.style.setProperty('--table-border', '#404040');
                }
                
                // 处理输入框
                if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
                    element.style.setProperty('--input-bg', '#252525');
                    element.style.setProperty('--input-text', '#e0e0e0');
                    element.style.setProperty('--input-border', '#404040');
                }
                
                // 处理按钮
                if (element.tagName === 'BUTTON') {
                    element.style.setProperty('--button-bg', '#404040');
                    element.style.setProperty('--button-text', '#e0e0e0');
                }
            }
        }

        dispatchThemeChangeEvent(theme) {
            const event = new CustomEvent('blackmagic-theme-change', {
                detail: { theme: theme }
            });
            document.dispatchEvent(event);
        }

        destroy() {
            // 清理观察器
            this.observers.forEach(observer => observer.disconnect());
            this.observers = [];

            // 恢复原始主题
            document.documentElement.classList.remove('dark-mode', 'light-mode');
            document.documentElement.removeAttribute('style');
        }
    }

    // 初始化 BlackMagic
    let blackMagic = null;

    // 立即初始化（在DOMContentLoaded之前）
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            blackMagic = new BlackMagic(BlackMagicConfig);
        });
    } else {
        blackMagic = new BlackMagic(BlackMagicConfig);
    }

    // 暴露到全局，方便调试和手动控制
    window.BlackMagic = BlackMagic;
    window.blackMagic = blackMagic;

    // 提供手动控制API
    window.applyDarkMode = function() {
        if (blackMagic) {
            blackMagic.applyTheme('dark');
        }
    };

    window.applyLightMode = function() {
        if (blackMagic) {
            blackMagic.applyTheme('light');
        }
    };

    console.log('[BlackMagic] 自动注入暗色模式已初始化');
})();