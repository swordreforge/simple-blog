/* ESBuild compressed */
var p=(s,o,a)=>new Promise((b,n)=>{var d=r=>{try{l(a.next(r))}catch(c){n(c)}},m=r=>{try{l(a.throw(r))}catch(c){n(c)}},l=r=>r.done?b(r.value):Promise.resolve(r.value).then(d,m);l((a=a.apply(s,o)).next())});(function(){"use strict";const s={brightness:100,contrast:100,grayscale:0,sepia:0,useFont:!1,textStroke:0,scrollbarColor:"auto",selectionColor:"auto",styleSystemControls:!0,lightSchemeMatches:!1,darkSchemeMatches:!0,immediateFetch:!0,ignoreInlineStyle:["*"],ignoreImageAnalysis:[],disableStyleSheetsProxy:!0,ignoreInlineAnalysis:["*"],disablePDFViewer:!1,disableStyleSheets:!1,ignoreSelectors:[".nav-icon",".nav-item",".navigation",".navbar","nav",'[class*="nav"]','[class*="menu"]',".shortcut-hint","svg"]};function o(){return typeof DarkReader!="undefined"&&DarkReader.enable}function a(){if(o())try{DarkReader.enable(s),console.log("[DarkReader] \u6697\u8272\u6A21\u5F0F\u5DF2\u542F\u7528"),d(),l("dark")}catch(e){console.error("[DarkReader] \u542F\u7528\u5931\u8D25:",e),n()}else console.warn("[DarkReader] Dark Reader\u5E93\u672A\u52A0\u8F7D\uFF0C\u4F7F\u7528\u5907\u7528\u65B9\u6848"),n()}function b(){if(o())try{DarkReader.disable(),console.log("[DarkReader] \u6697\u8272\u6A21\u5F0F\u5DF2\u7981\u7528"),l("light")}catch(e){console.error("[DarkReader] \u7981\u7528\u5931\u8D25:",e)}}function n(){document.documentElement.style.setProperty("color-scheme","dark"),document.documentElement.classList.add("dark-mode");const e=document.querySelector('meta[name="theme-color"]');e&&e.setAttribute("content","#000000"),m(),console.log("[DarkReader] \u5DF2\u5207\u6362\u5230\u5907\u7528\u6697\u8272\u6A21\u5F0F"),l("dark")}function d(){const e="dark-reader-custom-styles";let t=document.getElementById(e);t||(t=document.createElement("style"),t.id=e,document.head.appendChild(t)),t.textContent=`
            /* \u589E\u5F3A\u6697\u8272\u6548\u679C\u7684\u81EA\u5B9A\u4E49\u6837\u5F0F */
            html {
                color-scheme: dark !important;
            }
            
            body {
                background-color: #1a1a1a !important;
                color: #e0e0e0 !important;
            }
            
            /* \u4FDD\u62A4\u5BFC\u822A\u5143\u7D20\u7684\u989C\u8272\uFF0C\u4E0D\u88AB\u5F3A\u5236\u4FEE\u6539 */
            nav, .navbar, .nav-item, .nav-icon,
            [class*="nav-"], [class*="menu-"],
            .navigation, .shortcut-hint {
                color: inherit !important;
                stroke: currentColor !important;
                fill: none !important;
            }

            /* \u4E3A\u5BFC\u822A\u680F\u6309\u94AE\u5E94\u7528\u6BDB\u73BB\u7483\u6548\u679C */
            nav button, .navbar button,
            .nav button, [class*="nav-"] button,
            #loginBtn, #userCenterToggle,
            .shortcuts-help-btn,
            .user-center-item, .logout-item {
                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;
                backdrop-filter: blur(10px) !important;
                -webkit-backdrop-filter: blur(10px) !important;
                color: inherit !important;
                border: 1px solid rgba(255, 255, 255, 0.1) !important;
            }

            /* \u7528\u6237\u4E2D\u5FC3\u6309\u94AE\u60AC\u505C\u6548\u679C */
            .user-center-item:hover, .logout-item:hover {
                background-color: rgba(80, 80, 80, 0.8) !important;
                border-color: rgba(255, 255, 255, 0.3) !important;
            }

            /* \u5F3A\u5236\u6240\u6709\u8F93\u5165\u6846\u4F7F\u7528\u6697\u8272 */
            input, textarea, select {
                background-color: #2d2d2d !important;
                color: #e0e0e0 !important;
                border-color: #404040 !important;
            }

            /* \u8986\u76D6\u6D4F\u89C8\u5668\u81EA\u52A8\u586B\u5145\u7684\u9EC4\u8272\u80CC\u666F */
            input:-webkit-autofill,
            input:-webkit-autofill:hover,
            input:-webkit-autofill:focus,
            textarea:-webkit-autofill,
            textarea:-webkit-autofill:hover,
            textarea:-webkit-autofill:focus,
            select:-webkit-autofill,
            select:-webkit-autofill:hover,
            select:-webkit-autofill:focus {
                -webkit-box-shadow: 0 0 0 30px #2d2d2d inset !important;
                -webkit-text-fill-color: #e0e0e0 !important;
                background-color: #2d2d2d !important;
                color: #e0e0e0 !important;
                transition: background-color 5000s ease-in-out 0s !important;
            }

            /* \u4E3A\u6240\u6709\u6309\u94AE\u5E94\u7528\u900F\u660E\u6BDB\u73BB\u7483\u6548\u679C */
            button {
                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;
                backdrop-filter: blur(10px) !important;
                -webkit-backdrop-filter: blur(10px) !important;
                color: inherit !important;
                border: 1px solid rgba(255, 255, 255, 0.1) !important;
                transition: all 0.3s ease !important;
            }

            /* \u6309\u94AE\u60AC\u505C\u6548\u679C */
            button:hover {
                background-color: rgba(80, 80, 80, 0.8) !important;
                border-color: rgba(255, 255, 255, 0.3) !important;
            }
            
            /* \u5F3A\u5236\u6240\u6709\u8868\u683C\u4F7F\u7528\u6697\u8272 */
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
            
            /* \u53EA\u4E3A\u5B9E\u9645\u5185\u5BB9\u533A\u57DF\u7684\u94FE\u63A5\u8BBE\u7F6E\u989C\u8272\uFF0C\u4E0D\u5F71\u54CD\u5BFC\u822A */
            .article-content a, .content a, .post-content a {
                color: #4a9eff !important;
            }
            
            .article-content a:hover, .content a:hover, .post-content a:hover {
                color: #3a8eef !important;
            }
            
            /* \u5F3A\u5236\u6240\u6709\u4EE3\u7801\u5757\u4F7F\u7528\u6697\u8272 */
            pre, code {
                background-color: #2d2d2d !important;
                color: #e0e0e0 !important;
            }
            
            /* \u5F3A\u5236\u6240\u6709\u5361\u7247\u4F7F\u7528\u6697\u8272 */
            .card, .panel {
                background-color: #252525 !important;
                color: #e0e0e0 !important;
                border-color: #404040 !important;
            }

            /* \u4FDD\u7559\u6A21\u6001\u6846\u7684\u6BDB\u73BB\u7483\u6548\u679C */
            .modal {
                background-color: rgba(0, 0, 0, 0.5) !important;
                color: #e0e0e0 !important;
            }

            /* \u4FDD\u7559\u6A21\u6001\u6846\u5185\u5BB9\u7684\u6BDB\u73BB\u7483\u6548\u679C */
            .modal-content {
                background: rgba(0, 0, 0, 0) !important;
                backdrop-filter: blur(40px) saturate(200%) !important;
                -webkit-backdrop-filter: blur(40px) saturate(200%) !important;
                border: 1px solid rgba(255, 255, 255, 0.5) !important;
                color: #e0e0e0 !important;
            }
            
            /* \u5F3A\u5236\u5BFC\u822A\u680F\u4F7F\u7528\u6697\u8272\u80CC\u666F\uFF0C\u4F46\u4FDD\u7559\u6BDB\u73BB\u7483\u6548\u679C */
            nav, .navbar, header {
                background-color: var(--navbar-glass-color, rgba(60, 60, 60, 0.6)) !important;
                backdrop-filter: blur(10px) !important;
                -webkit-backdrop-filter: blur(10px) !important;
                border-color: rgba(255, 255, 255, 0.1) !important;
            }
            
            /* \u5F3A\u5236\u4FA7\u8FB9\u680F\u4F7F\u7528\u6697\u8272 */
            aside, .sidebar {
                background-color: #252525 !important;
                color: #e0e0e0 !important;
            }
            
            /* \u5F3A\u5236\u9875\u811A\u4F7F\u7528\u6697\u8272\u4F46\u4FDD\u7559\u6BDB\u73BB\u7483\u6548\u679C */
            footer {
                background-color: var(--footer-glass-color, rgba(45, 45, 45, 0.6)) !important;
                backdrop-filter: blur(10px) !important;
                -webkit-backdrop-filter: blur(10px) !important;
                color: #ffffff !important;
                text-shadow: 0 0 10px rgba(255, 255, 255, 0.5),
                             0 0 20px rgba(255, 255, 255, 0.3),
                             0 0 30px rgba(255, 255, 255, 0.1) !important;
                border-color: rgba(255, 255, 255, 0.1) !important;
            }
            
            /* \u5F3A\u5236\u6EDA\u52A8\u6761\u4F7F\u7528\u6697\u8272 */
            ::-webkit-scrollbar {
                background-color: #2d2d2d !important;
            }
            
            ::-webkit-scrollbar-thumb {
                background-color: #4a4a4a !important;
            }
        `}function m(){const e="dark-reader-fallback-styles";let t=document.getElementById(e);t||(t=document.createElement("style"),t.id=e,document.head.appendChild(t)),t.textContent=d.toString().match(/\/[\*\s\S]*?\`\`/)[0].replace(/\/[\*\s\S]*?\`\`/,d.toString().match(/styleElement\.textContent = \`([\s\S]*)\`;/)[1])}function l(e){const t=new CustomEvent("dark-reader-theme-change",{detail:{theme:e}});document.dispatchEvent(t)}function r(){return new Promise((e,t)=>{if(o()){e();return}const i=document.createElement("script");i.src="/js/npm/darkreader@4.9.92/darkreader.min.js",i.onload=e,i.onerror=t,document.head.appendChild(i)})}function c(){return p(this,null,function*(){try{yield r(),a(),u(),k()}catch(e){console.error("[DarkReader] \u521D\u59CB\u5316\u5931\u8D25:",e),n()}})}function u(){new MutationObserver(t=>{t.forEach(i=>{i.type==="childList"&&i.addedNodes.forEach(g=>{g.nodeType===Node.ELEMENT_NODE&&o()&&DarkReader.enable(s)})})}).observe(document.body,{childList:!0,subtree:!0})}function k(){window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change",t=>{console.log(`[DarkReader] \u7CFB\u7EDF\u4E3B\u9898\u53D8\u4E3A: ${t.matches?"dark":"light"}`)})}window.DarkReaderInjector={enable:a,disable:b,toggle:()=>{o()&&DarkReader.isEnabled()?b():a()},isAvailable:o,fallback:n},document.readyState==="loading"?document.addEventListener("DOMContentLoaded",c):c(),console.log("[DarkReader] \u4E13\u4E1A\u6697\u8272\u6A21\u5F0F\u6CE8\u5165\u5668\u5DF2\u52A0\u8F7D")})();
