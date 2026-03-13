class KeyboardShortcuts{constructor(){this.shortcuts={1:{action:"navigate",url:"/",label:"主页"},2:{action:"navigate",url:"/passage",label:"文章"},3:{action:"navigate",url:"/collect",label:"归档"},4:{action:"navigate",url:"/about",label:"关于"},5:{action:"openModal",modalId:"userCenterModal",label:"个人中心"},6:{action:"navigate",url:"/markdown-editor",label:"编辑器"},f:{action:"navigate",url:"/friends",label:"友链"},l:{action:"openModal",modalId:"loginModal",label:"登录"},"/":{action:"showHelp",label:"快捷键帮助"},Escape:{action:"closeAllModals",label:"关闭模态框"}," ":{action:"music",musicAction:"togglePlay",label:"播放/暂停"},ArrowLeft:{action:"music",musicAction:"previous",label:"上一首"},ArrowRight:{action:"music",musicAction:"next",label:"下一首"},ArrowUp:{action:"music",musicAction:"volumeUp",label:"音量+"},ArrowDown:{action:"music",musicAction:"volumeDown",label:"音量-"},m:{action:"music",musicAction:"mute",label:"静音"},p:{action:"music",musicAction:"playlist",label:"播放列表"},a:{action:"navigate",url:"/admin",label:"管理员设置",adminOnly:!0}},this.enabled=!0,this.init()}init(){document.addEventListener("keydown",e=>this.handleKeyPress(e)),this.showShortcutHints(),this.addHelpButton()}handleKeyPress(t){if(this.enabled){var s=document.activeElement;if(!s||"INPUT"!==s.tagName&&"TEXTAREA"!==s.tagName&&!s.isContentEditable){s=document.getElementById("musicPlaylist"),s=s&&s.classList.contains("show");if(!s||"ArrowUp"!==t.key&&"ArrowDown"!==t.key)if(!document.body.classList.contains("focus-mode")||"ArrowUp"!==t.key&&"ArrowDown"!==t.key&&"ArrowLeft"!==t.key&&"ArrowRight"!==t.key){s=document.body.classList.contains("collect-focus-mode");if(!s||"ArrowUp"!==t.key&&"ArrowDown"!==t.key&&"ArrowLeft"!==t.key&&"ArrowRight"!==t.key)if(!document.body.classList.contains("about-focus-mode")||"ArrowUp"!==t.key&&"ArrowDown"!==t.key){var s=t.key;let e=null;this.shortcuts[s]?e=this.shortcuts[s]:(s={49:"1",50:"2",51:"3",52:"4",53:"5",54:"6",76:"l"}[t.keyCode])&&this.shortcuts[s]&&(e=this.shortcuts[s]),e&&(t.preventDefault(),t.stopPropagation(),e.adminOnly&&!this.isAdmin()?this.showToast("此快捷键仅管理员可用","warning"):this.executeAction(e))}}}}}executeAction(e){switch(e.action){case"navigate":window.location.pathname!==e.url&&(window.location.href=e.url);break;case"openModal":var t=document.getElementById(e.modalId);t&&(t.classList.add("active"),this.showToast("已打开: "+e.label,"success"));break;case"closeAllModals":document.querySelectorAll(".modal.active").forEach(e=>{e.classList.remove("active")}),this.showToast("已关闭所有模态框","success");break;case"showHelp":this.showHelpModal(),this.showToast("快捷键帮助","success");break;case"music":this.executeMusicAction(e.musicAction,e.label)}}executeMusicAction(e,t){if(window.musicPlayer&&window.musicPlayer.settings&&window.musicPlayer.settings.enabled){var s,a,i,r,l=window.musicPlayer;try{switch(e){case"togglePlay":l.togglePlay(),this.showToast(l.isPlaying?"正在播放":"已暂停","success");break;case"previous":l.playPrevious(),this.showToast("上一首","success");break;case"next":l.playNext(),this.showToast("下一首","success");break;case"volumeUp":l.audio&&(s=Math.min(100,100*l.audio.volume+10),l.audio.volume=s/100,(a=document.querySelector("#volumeBar"))&&(a.value=s),l.saveState(),this.showToast(`音量: ${Math.round(s)}%`,"success"));break;case"volumeDown":l.audio&&(i=Math.max(0,100*l.audio.volume-10),l.audio.volume=i/100,(r=document.querySelector("#volumeBar"))&&(r.value=i),l.saveState(),this.showToast(`音量: ${Math.round(i)}%`,"success"));break;case"mute":l.toggleMute(),this.showToast(l.audio.muted?"已静音":"已取消静音","success");break;case"playlist":l.togglePlaylist(),this.showToast("播放列表","success")}}catch(e){console.error("[音乐播放器快捷键错误]",e),this.showToast("操作失败: "+e.message,"error")}}else this.showToast("音乐播放器未启用","warning")}isAdmin(){var e=document.querySelectorAll(".admin-only");return Array.from(e).some(e=>"none"!==window.getComputedStyle(e).display)}isPassagePage(){return"/passage"===window.location.pathname||window.location.pathname.startsWith("/passage/")}isCollectPage(){return"/collect"===window.location.pathname||window.location.pathname.startsWith("/collect/")}isAboutPage(){return"/about"===window.location.pathname}showShortcutHints(){/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)||window.innerWidth<=768||document.querySelectorAll("nav a, nav button").forEach(e=>{const s=e.getAttribute("href"),a=e.getAttribute("id");let i=null,r=null;var t;Object.entries(this.shortcuts).forEach(([e,t])=>{("navigate"===t.action&&t.url===s||"openModal"===t.action&&t.modalId===a)&&(i=e,r=t.label)}),i&&r&&((t=e.querySelector(".shortcut-hint"))||((t=document.createElement("span")).className="shortcut-hint",t.textContent=i,e.appendChild(t)))})}addHelpButton(){var e=document.createElement("button"),t=(e.className="shortcuts-help-btn",e.innerHTML=`
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"></circle>
        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
        <line x1="12" y1="17" x2="12.01" y2="17"></line>
      </svg>
      快捷键
      <span class="shortcut-hint">/</span>
    `,e.addEventListener("click",()=>this.showHelpModal()),document.querySelector("nav"));t&&t.appendChild(e)}showHelpModal(){const t=document.createElement("div");t.className="modal shortcuts-help-modal active",t.innerHTML=`
      <div class="modal-content">
        <div class="modal-header">
          <h3>键盘快捷键</h3>
          <button class="modal-close">&times;</button>
        </div>
        <div class="modal-body">
          <div class="shortcuts-list">
            <h4>导航快捷键</h4>
            ${this.renderShortcutList(["1","2","3","4","6"])}

            <h4>功能快捷键</h4>
            ${this.renderShortcutList(["5","f","l","/","Escape"])}

            ${this.isPassagePage()?`
            <h4>文章页面 - 文本聚焦模式</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">i</kbd>
              <span class="shortcut-label">进入文本聚焦模式</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">退出文本聚焦模式</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">ESC</kbd>
              <span class="shortcut-label">暂时退出聚焦模式（可关闭模态框）</span>
            </div>
            <div class="shortcut-description">
              聚焦模式下：← → 切换面板，↑ ↓ 导航，Enter 激活，u 展开/折叠
            </div>
            `:""}

            ${this.isCollectPage()?`
            <h4>归档页面 - 聚焦模式</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">i</kbd>
              <span class="shortcut-label">进入聚焦模式</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">退出聚焦模式</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">ESC</kbd>
              <span class="shortcut-label">返回上一级或暂时退出</span>
            </div>
            <div class="shortcut-description">
              聚焦模式下：↑ ↓ ← → 导航，Enter 进入子菜单/激活，ESC 返回
            </div>
            `:""}

            ${this.isAboutPage()?`
            <h4>关于页面 - 聚焦模式</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">i</kbd>
              <span class="shortcut-label">进入聚焦模式</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">退出聚焦模式</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">ESC</kbd>
              <span class="shortcut-label">暂时退出聚焦模式（可关闭模态框）</span>
            </div>
            <div class="shortcut-description">
              聚焦模式下：↑ ↓ 导航卡片，Enter 查看卡片内容
            </div>
            `:""}

            <h4>音乐播放器快捷键</h4>
            ${this.renderShortcutList([" ","ArrowLeft","ArrowRight","ArrowUp","ArrowDown","m","p"])}

            ${this.isAdmin()?"<h4>管理员快捷键</h4>":""}
            ${this.isAdmin()?this.renderShortcutList(["a"]):""}
          </div>
        </div>
      </div>
    `,document.body.appendChild(t),t.querySelector(".modal-close").addEventListener("click",()=>{t.classList.remove("active"),setTimeout(()=>t.remove(),300)}),t.addEventListener("click",e=>{e.target===t&&(t.classList.remove("active"),setTimeout(()=>t.remove(),300))});const s=e=>{"Escape"===e.key&&(t.classList.remove("active"),setTimeout(()=>t.remove(),300),document.removeEventListener("keydown",s))};document.addEventListener("keydown",s)}renderShortcutList(e){return e.map(e=>{var t=this.shortcuts[e];if(!t)return"";let s=e;return" "===e?s="Space":"ArrowLeft"===e?s="←":"ArrowRight"===e?s="→":"ArrowUp"===e?s="↑":"ArrowDown"===e&&(s="↓"),`
        <div class="shortcut-item">
          <kbd class="shortcut-key">${s}</kbd>
          <span class="shortcut-label">${t.label}</span>
        </div>
      `}).join("")}showToast(e,t="info"){const s=document.createElement("div");s.className="toast "+t,s.innerHTML=`
      <span class="toast-icon">${this.getToastIcon(t)}</span>
      <span class="toast-message">${e}</span>
      <button class="toast-close">&times;</button>
    `;t=document.getElementById("toastContainer");(t||((e=document.createElement("div")).id="toastContainer",e.className="toast-container",document.body.appendChild(e),e)).appendChild(s),setTimeout(()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)},2e3),s.querySelector(".toast-close").addEventListener("click",()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)})}getToastIcon(e){var t={success:"✓",error:"✕",warning:"⚠",info:"ℹ"};return t[e]||t.info}enable(){this.enabled=!0}disable(){this.enabled=!1}}class AdminKeyboardManager{constructor(){this.isFocusMode=!1,this.currentTab="articles",this.activeModal=null,this.selectedRows=new Set,this.selectedFile=null,this.currentPath="/",this.aboutCurrentTable="main",this.tabs=["articles","users","comments","categories","tags","analytics","about","filemanager","attachments","settings"],this.init()}init(){this.isAdminPage()&&(document.addEventListener("keydown",this.handleKeyDown.bind(this)),document.addEventListener("keyup",this.handleKeyUp.bind(this)),this.observeModals(),this.observeTabs())}isAdminPage(){return"/admin"===window.location.pathname||window.location.pathname.startsWith("/admin")}handleKeyDown(e){if(this.isInputElement(e.target))return this.handleInputShortcuts(e);var t=this.getKeyString(e);this.handleGlobalShortcuts(t,e)||!this.isFocusMode||this.activeModal&&this.handleModalShortcuts(t,e)||this.handleTabShortcuts(t,e)||this.handleFocusModeShortcuts(t,e)}handleKeyUp(e){}isInputElement(e){return e&&("INPUT"===e.tagName||"TEXTAREA"===e.tagName||"SELECT"===e.tagName||e.isContentEditable)}handleInputShortcuts(e){const s=this.getKeyString(e);if("Escape"===s){if(this.activeModal)return this.closeCurrentModal(),e.preventDefault(),!0;if(this.isFocusMode)return this.exitFocusMode(),e.preventDefault(),!0}if("settings"===this.currentTab&&this.isFocusMode){if("q"===s)return e.target.blur(),this.exitFocusMode(),e.preventDefault(),!0;if("s"===s)return e.target.blur(),setTimeout(()=>{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t){t=this.getVisibleSettingsSection(t);if(t){let e=null;t=t.querySelector("h4")?.textContent||"";t.includes("外观")?e=document.getElementById("saveSettingsBtn"):t.includes("音乐")?e=document.getElementById("saveMusicSettingsBtn"):(t.includes("模板")||t.includes("文章标题")||t.includes("切换界面")||t.includes("外部链接")||t.includes("赞助"))&&(e=document.getElementById("saveTemplateSettingsBtn")),e&&e.click()}}},100),e.preventDefault(),!0;if("1"<=s&&s<="7")return e.target.blur(),setTimeout(()=>{var e,t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);t&&(e=parseInt(s),t=t.querySelector(`.settings-section:nth-of-type(${e})`))&&(t.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(t),this.showToast("已跳转到设置区块 "+s,"success"))},100),e.preventDefault(),!0}return!1}getKeyString(e){let t=e.key;return t=" "===t?"Space":t}handleGlobalShortcuts(e,t){switch(e){case"i":if(this.activeModal)break;return this.enterFocusMode(),t.preventDefault(),!0;case"q":return this.exitFocusMode(),t.preventDefault(),!0;case"Escape":return this.activeModal?this.closeCurrentModal():this.isFocusMode&&this.exitFocusMode(),t.preventDefault(),!0}return!1}handleModalShortcuts(e,t){if(this.activeModal){var s=document.activeElement,a=this.isInputElement(s);switch(e){case"Escape":return this.closeCurrentModal(),t.preventDefault(),!0;case"Enter":if(a&&"TEXTAREA"===s.tagName&&t.shiftKey)return!1;if(a){var i=s.closest("form");if(i){i=i.querySelector('button[type="submit"], .btn-primary');if(i)return i.click(),t.preventDefault(),!0}i=this.activeModal.querySelector(".btn-primary");if(i)return i.click(),t.preventDefault(),!0}else{i=this.activeModal.querySelector(".btn-primary");if(i)return i.click(),t.preventDefault(),!0}break;case"s":i=this.activeModal.querySelector('button[type="submit"], .btn-primary');if(i)return i.click(),t.preventDefault(),!0;break;case"y":let e=this.activeModal.querySelector("#confirmAction");if(e=(e=e||this.activeModal.querySelector('button[type="submit"]'))||this.activeModal.querySelector(".btn-primary")){i=window.getComputedStyle(e);if("none"!==i.display&&"hidden"!==i.visibility&&!e.disabled)return e.click(),t.preventDefault(),!0}break;case"c":i=this.activeModal.querySelector(".btn-secondary, button[data-modal]");return i?i.click():this.closeCurrentModal(),t.preventDefault(),!0;case"Tab":return this.handleTabNavigation(t),!0;case" ":if(!s||"radio"!==s.type&&"checkbox"!==s.type)break;return s.click(),t.preventDefault(),!0;case"ArrowDown":case"ArrowUp":if(a&&"SELECT"===s.tagName)return!1}}return!1}handleTabShortcuts(e,t){if("0"<=e&&e<="9"&&"settings"!==this.currentTab){var s="0"===e?9:parseInt(e)-1;if(s<this.tabs.length)return this.switchToTab(this.tabs[s]),t.preventDefault(),!0}if("settings"===this.currentTab&&"Tab"===e)return this.handleSettingsTabNavigation(t);if(this.handleRowNavigation(e,t))return!0;switch(e){case"ArrowRight":return this.nextTab(),t.preventDefault(),!0;case"ArrowLeft":return this.previousTab(),t.preventDefault(),!0;case"r":return this.refreshCurrentTab(),t.preventDefault(),!0;case"n":return this.createNewItem(),t.preventDefault(),!0;case"u":return this.uploadItem(),t.preventDefault(),!0;case"f":return this.openSearch(),t.preventDefault(),!0}return this.handleSpecificTabShortcuts(e,t)}handleSpecificTabShortcuts(e,t){switch(this.currentTab){case"articles":return this.handleArticleShortcuts(e,t);case"filemanager":return this.handleFileManagerShortcuts(e,t);case"users":return this.handleUserShortcuts(e,t);case"comments":return this.handleCommentShortcuts(e,t);case"categories":return this.handleCategoryShortcuts(e,t);case"tags":return this.handleTagShortcuts(e,t);case"attachments":return this.handleAttachmentShortcuts(e,t);case"about":return this.handleAboutShortcuts(e,t);case"settings":return this.handleSettingsShortcuts(e,t)}return!1}handleArticleShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"e":return this.editSelectedArticle(),t.preventDefault(),!0;case"d":return this.deleteSelectedArticle(),t.preventDefault(),!0;case"v":return this.viewSelectedArticle(),t.preventDefault(),!0;case"a":return this.attachToSelectedArticle(),t.preventDefault(),!0;case"p":return this.publishSelectedArticle(),t.preventDefault(),!0}return!1}handleFileManagerShortcuts(e,t){switch(e){case"Enter":return this.openSelectedFile(),t.preventDefault(),!0;case"Backspace":return this.goUpDirectory(),t.preventDefault(),!0;case"r":return this.selectedFile?this.renameSelectedFile():this.refreshCurrentTab(),t.preventDefault(),!0;case"Delete":return this.deleteSelectedFile(),t.preventDefault(),!0}return!1}handleUserShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"e":return this.editSelectedUser(),t.preventDefault(),!0;case"d":return this.deleteSelectedUser(),t.preventDefault(),!0}return!1}handleCommentShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"a":return this.approveSelectedComment(),t.preventDefault(),!0;case"d":return this.deleteSelectedComment(),t.preventDefault(),!0}return!1}handleCategoryShortcuts(e,t){switch(e){case"a":var s=document.getElementById("addCategoryBtn");if(s)return s.click(),t.preventDefault(),!0;break;case"e":return this.selectedRows.size||this.hasSelectedRow()?(this.editSelectedCategory(),t.preventDefault(),!0):!1;case"d":return this.selectedRows.size||this.hasSelectedRow()?(this.deleteSelectedCategory(),t.preventDefault(),!0):!1}return!1}handleTagShortcuts(e,t){switch(e){case"a":var s=document.getElementById("addTagBtn");if(s)return s.click(),t.preventDefault(),!0;break;case"e":return this.selectedRows.size||this.hasSelectedRow()?(this.editSelectedTag(),t.preventDefault(),!0):!1;case"d":return this.selectedRows.size||this.hasSelectedRow()?(this.deleteSelectedTag(),t.preventDefault(),!0):!1}return!1}handleAttachmentShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"v":return this.viewSelectedAttachment(),t.preventDefault(),!0;case"e":return this.editSelectedAttachment(),t.preventDefault(),!0;case"d":return this.deleteSelectedAttachment(),t.preventDefault(),!0}return!1}handleAboutShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow()){var s=this.getSelectedRow();if(s)switch(e){case"e":var a=s.querySelector('button[onclick*="edit"]');if(a)return a.click(),t.preventDefault(),!0;break;case"d":a=s.querySelector('button[onclick*="toggle"]');if(a)return a.click(),t.preventDefault(),!0;break;case"c":a=s.querySelector("button.btn-danger");if(a)return a.click(),t.preventDefault(),!0}}return!1}handleSettingsShortcuts(e,t){var s=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(s)switch(e){case"1":var a=s.querySelector(".settings-section:nth-of-type(1)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到外观设置","success"),t.preventDefault(),!0;break;case"2":a=s.querySelector(".settings-section:nth-of-type(2)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到音乐设置","success"),t.preventDefault(),!0;break;case"3":a=s.querySelector(".settings-section:nth-of-type(3)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到模板设置","success"),t.preventDefault(),!0;break;case"4":a=s.querySelector(".settings-section:nth-of-type(4)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到文章标题设置","success"),t.preventDefault(),!0;break;case"5":a=s.querySelector(".settings-section:nth-of-type(5)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到切换界面提示设置","success"),t.preventDefault(),!0;break;case"6":a=s.querySelector(".settings-section:nth-of-type(6)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到外部链接设置","success"),t.preventDefault(),!0;break;case"7":a=s.querySelector(".settings-section:nth-of-type(7)");if(a)return a.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(a),this.showToast("已跳转到赞助设置","success"),t.preventDefault(),!0;break;case"s":a=this.getVisibleSettingsSection(s);if(a){let e=null;a=a.querySelector("h4")?.textContent||"";if(a.includes("外观")?e=document.getElementById("saveSettingsBtn"):a.includes("音乐")?e=document.getElementById("saveMusicSettingsBtn"):(a.includes("模板")||a.includes("文章标题")||a.includes("切换界面")||a.includes("外部链接")||a.includes("赞助"))&&(e=document.getElementById("saveTemplateSettingsBtn")),e)return e.click(),t.preventDefault(),!0}break;case"r":a=document.getElementById("resetSettingsBtn");if(a)return a.click(),t.preventDefault(),!0;break;case"?":return this.showSettingsShortcutHelp(),t.preventDefault(),!0}return!1}getVisibleSettingsSection(e){var e=e.querySelectorAll(".settings-section"),t=window.innerHeight/2;for(const i of e){var s=i.getBoundingClientRect(),a=s.top+s.height/2;if(Math.abs(a-t)<s.height/2)return i}return null}focusFirstInputInSection(e){if(e){const t=e.querySelectorAll('input[type="text"], input[type="number"], input[type="color"], textarea, select, input[type="checkbox"]');0<t.length&&setTimeout(()=>{var e=t[0];e.focus(),e.scrollIntoView({behavior:"smooth",block:"center"})},300)}}handleSettingsTabNavigation(e){var t,s=document.querySelectorAll('#settings input[type="text"], #settings input[type="number"], #settings input[type="color"], #settings textarea, #settings select, #settings input[type="checkbox"], #settings input[type="range"]');return 0!==s.length&&(t=document.activeElement,t=Array.from(s).indexOf(t),(e.shiftKey?(e.preventDefault(),t<=0?s[s.length-1]:s[t-1]):(e.preventDefault(),-1===t||t>=s.length-1?s[0]:s[t+1])).focus(),(e=document.activeElement)&&e.scrollIntoView({behavior:"smooth",block:"center"}),!0)}showSettingsShortcutHelp(){const t=document.createElement("div");t.className="modal shortcuts-help-modal active",t.innerHTML=`
      <div class="modal-content">
        <div class="modal-header">
          <h3>系统设置快捷键</h3>
          <button class="modal-close">&times;</button>
        </div>
        <div class="modal-body">
          <div class="shortcuts-list">
            <h4>区块导航</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">1</kbd>
              <span class="shortcut-label">外观设置（背景、透明度、毛玻璃颜色等）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">2</kbd>
              <span class="shortcut-label">音乐设置（播放器、上传、播放列表）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">3</kbd>
              <span class="shortcut-label">模板设置（标题、欢迎语、年份、头像）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">4</kbd>
              <span class="shortcut-label">文章标题设置（显示、前缀）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">5</kbd>
              <span class="shortcut-label">切换界面提示设置</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">6</kbd>
              <span class="shortcut-label">外部链接设置（警告、白名单、Live2D）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">7</kbd>
              <span class="shortcut-label">赞助设置（标题、图片、描述）</span>
            </div>

            <h4>表单操作</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">Tab</kbd>
              <span class="shortcut-label">在表单控件间导航（Shift+Tab 反向）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">Space</kbd>
              <span class="shortcut-label">切换复选框选中状态</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">↑</kbd>
              <kbd class="shortcut-key">↓</kbd>
              <span class="shortcut-label">在下拉框中切换选项</span>
            </div>

            <h4>功能快捷键</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">s</kbd>
              <span class="shortcut-label">保存当前区块设置（在输入框中也可用）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">r</kbd>
              <span class="shortcut-label">重置为默认设置</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">退出聚焦模式（在输入框中也可用）</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">?</kbd>
              <span class="shortcut-label">显示此帮助</span>
            </div>

            <h4>提示</h4>
            <div class="shortcut-description">
              • 数字键可在编辑输入框时直接使用，无需先移出焦点<br>
              • 按 s 键保存时，输入框会自动失去焦点并更新值<br>
              • Tab 键只导航到表单控件，会跳过操作按钮
            </div>
          </div>
        </div>
      </div>
    `,document.body.appendChild(t),t.querySelector(".modal-close").addEventListener("click",()=>{t.classList.remove("active"),setTimeout(()=>t.remove(),300)}),t.addEventListener("click",e=>{e.target===t&&(t.classList.remove("active"),setTimeout(()=>t.remove(),300))});const s=e=>{"Escape"===e.key&&(t.classList.remove("active"),setTimeout(()=>t.remove(),300),document.removeEventListener("keydown",s))};document.addEventListener("keydown",s)}handleRowNavigation(e,t){var s=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(s){if("about"===this.currentTab)return this.handleAboutRowNavigation(e,t,s);s=s.querySelector(".data-table");if(s){s=s.querySelector("tbody");if(s){var a=Array.from(s.querySelectorAll("tr"));if(0!==a.length){var i=this.getSelectedRow(),r=i?a.indexOf(i):-1;switch(e){case"ArrowUp":return t.preventDefault(),r<=0?this.selectRow(a[a.length-1]):this.selectRow(a[r-1]),!0;case"ArrowDown":return t.preventDefault(),r<0||r>=a.length-1?this.selectRow(a[0]):this.selectRow(a[r+1]),!0;case"Home":return t.preventDefault(),this.selectRow(a[0]),!0;case"End":return t.preventDefault(),this.selectRow(a[a.length-1]),!0;case"PageUp":t.preventDefault();var l=Math.max(0,r-10);return this.selectRow(a[l]),!0;case"PageDown":t.preventDefault();l=Math.min(a.length-1,r+10);return this.selectRow(a[l]),!0;case"Enter":return t.preventDefault(),i&&this.activateSelectedRow(),!0;case" ":return t.preventDefault(),i&&this.toggleRowSelection(i),!0}}}}}return!1}handleAboutRowNavigation(e,t,s){var a=s.querySelector("#mainCards"),s=s.querySelector("#subCards"),i="main"===this.aboutCurrentTable?a:s,r="main"===this.aboutCurrentTable?s:a;if(i){s=i.querySelector("tbody");if(s){var l=Array.from(s.querySelectorAll("tr"));if(0!==l.length){var c=this.getSelectedRow(),o=c?l.indexOf(c):-1;switch(e){case"ArrowUp":if(t.preventDefault(),o<=0){if(r){var n=r.querySelector("tbody"),n=Array.from(n.querySelectorAll("tr"));if(0<n.length)return this.aboutCurrentTable="main"===this.aboutCurrentTable?"sub":"main",this.selectRow(n[n.length-1]),!0}this.selectRow(l[l.length-1])}else this.selectRow(l[o-1]);return!0;case"ArrowDown":if(t.preventDefault(),o<0||o>=l.length-1){if(r){n=r.querySelector("tbody"),n=Array.from(n.querySelectorAll("tr"));if(0<n.length)return this.aboutCurrentTable="main"===this.aboutCurrentTable?"sub":"main",this.selectRow(n[0]),!0}this.selectRow(l[0])}else this.selectRow(l[o+1]);return!0;case"Tab":if(t.preventDefault(),r){n=r.querySelector("tbody"),n=Array.from(n.querySelectorAll("tr"));if(0<n.length)return this.aboutCurrentTable="main"===this.aboutCurrentTable?"sub":"main",this.selectRow(n[0]),!0}return!1;case"Home":return t.preventDefault(),this.selectRow(l[0]),!0;case"End":return t.preventDefault(),this.selectRow(l[l.length-1]),!0;case"PageUp":t.preventDefault();n=Math.max(0,o-10);return this.selectRow(l[n]),!0;case"PageDown":t.preventDefault();n=Math.min(l.length-1,o+10);return this.selectRow(l[n]),!0;case"Enter":return t.preventDefault(),c&&this.activateSelectedRow(),!0;case" ":return t.preventDefault(),c&&this.toggleRowSelection(c),!0}}}}return!1}selectRow(e){document.querySelectorAll(".data-table tbody tr").forEach(e=>e.classList.remove("selected")),e.classList.add("selected"),e.scrollIntoView({behavior:"smooth",block:"nearest"}),this.selectedRows.clear();e=e.dataset.id||e.querySelector("td:first-child")?.textContent;e&&this.selectedRows.add(e)}toggleRowSelection(e){var t;e.classList.contains("selected")?(e.classList.remove("selected"),(t=e.dataset.id||e.querySelector("td:first-child")?.textContent)&&this.selectedRows.delete(t)):(e.classList.add("selected"),(t=e.dataset.id||e.querySelector("td:first-child")?.textContent)&&this.selectedRows.add(t))}activateSelectedRow(){var e=this.getSelectedRow();if(e)switch(this.currentTab){case"articles":this.viewSelectedArticle();break;case"filemanager":this.openSelectedFile();break;case"attachments":this.viewSelectedAttachment();break;default:var t=e.querySelector(".btn-edit");t?t.click():(t=e.querySelector(".btn-view"))&&t.click()}}clearRowSelection(){document.querySelectorAll(".data-table tbody tr").forEach(e=>e.classList.remove("selected")),this.selectedRows.clear()}handleFocusModeShortcuts(e,t){switch(e){case"Tab":return t.preventDefault(),!0;case"?":return this.showAdminShortcutHelp(),t.preventDefault(),!0}return!1}enterFocusMode(){this.isFocusMode||(this.isFocusMode=!0,document.body.classList.add("admin-focus-mode"),window.keyboardShortcuts&&window.keyboardShortcuts.disable(),setTimeout(()=>{var e=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);e&&(e=e.querySelector(".data-table tbody"))&&(e=e.querySelector("tr"))&&this.selectRow(e)},100),this.showToast("已进入管理员聚焦模式","success"))}exitFocusMode(){this.isFocusMode&&(this.isFocusMode=!1,document.body.classList.remove("admin-focus-mode"),this.clearRowSelection(),window.keyboardShortcuts&&window.keyboardShortcuts.enable(),this.showToast("已退出管理员聚焦模式","info"))}switchToTab(e){var t=document.querySelector(`.tab-btn[data-tab="${e}"]`);t&&(t.click(),this.currentTab=e,this.clearRowSelection(),"about"===e)&&(this.aboutCurrentTable="main")}nextTab(){var e=(this.tabs.indexOf(this.currentTab)+1)%this.tabs.length;this.switchToTab(this.tabs[e])}previousTab(){var e=(this.tabs.indexOf(this.currentTab)-1+this.tabs.length)%this.tabs.length;this.switchToTab(this.tabs[e])}refreshCurrentTab(){let e=null;if(e="articles"===this.currentTab?document.getElementById("refreshArticlesBtn"):"attachments"===this.currentTab?document.getElementById("amRefreshBtn"):"filemanager"===this.currentTab?document.getElementById("fmRefreshBtn"):document.querySelector(`#${this.currentTab}RefreshBtn, .refresh-btn`))e.click(),this.showToast("已刷新","success");else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const a of t.querySelectorAll("button")){var s=a.textContent.trim();if("刷新"===s||s.includes("刷新"))return a.click(),void this.showToast("已刷新","success")}"articles"===this.currentTab&&"function"==typeof loadPassages?(loadPassages(),this.showToast("已刷新","success")):"attachments"===this.currentTab&&"function"==typeof loadAttachments?(loadAttachments(),this.showToast("已刷新","success")):"filemanager"===this.currentTab&&window.FileManager&&(FileManager.loadFiles(),this.showToast("已刷新","success"))}}createNewItem(){let e=null;if(e="articles"===this.currentTab?document.getElementById("newArticleBtn"):"users"===this.currentTab?document.getElementById("newUserBtn"):"categories"===this.currentTab?document.getElementById("newCategoryBtn"):"tags"===this.currentTab?document.getElementById("newTagBtn"):document.querySelector(`#${this.currentTab}NewBtn, .new-btn`))e.click();else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const i of t.querySelectorAll("button")){var s=i.textContent.trim();if("新建文章"===s||"新建用户"===s||"新建分类"===s||"新建标签"===s||s.startsWith("新建")||s.startsWith("创建")||s.startsWith("添加"))return void i.click()}for(const r of document.querySelectorAll("button")){var a=r.textContent.trim();if("新建文章"===a||"新建用户"===a||"新建分类"===a||"新建标签"===a||a.startsWith("新建"))return void r.click()}this.showToast("未找到新建按钮","warning")}}uploadItem(){let e=null;if(e="attachments"===this.currentTab?document.getElementById("amUploadBtn"):"filemanager"===this.currentTab?document.getElementById("fmUploadBtn"):document.querySelector(`#${this.currentTab}UploadBtn, .upload-btn`))e.click();else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const s of t.querySelectorAll("button"))if(s.textContent.trim().includes("上传"))return void s.click();for(const a of document.querySelectorAll("button"))if(a.textContent.trim().includes("上传"))return void a.click();this.showToast("未找到上传按钮","warning")}}openSearch(){let e=null;if(e="attachments"===this.currentTab?document.getElementById("amSearchInput"):"filemanager"===this.currentTab?document.getElementById("fmSearchInput"):"articles"===this.currentTab?document.getElementById("articlesSearchInput"):document.querySelector(`#${this.currentTab}SearchInput, .search-input, input[type="search"]`))e.focus(),this.showToast("已聚焦到搜索框","success");else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const a of t.querySelectorAll('input[type="text"], input[type="search"]')){var s=a.placeholder||"";if(s.includes("搜索")||s.includes("筛选")||s.includes("查找"))return a.focus(),void this.showToast("已聚焦到搜索框","success")}this.showToast("未找到搜索框","warning")}}hasSelectedRow(){return null!==document.querySelector(".data-table tr.selected")}getSelectedRow(){return document.querySelector(".data-table tr.selected")}editSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}viewSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-view"))&&e.click()}attachToSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-upload"))&&e.click()}publishSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector('.btn-publish, .btn-primary:contains("发布")'))&&e.click()}openSelectedFile(){this.selectedFile&&window.FileManager&&window.FileManager.openFile(this.selectedFile.path)}goUpDirectory(){window.FileManager&&window.FileManager.goBack()}renameSelectedFile(){this.selectedFile&&window.FileManager&&window.FileManager.openRenameModal()}deleteSelectedFile(){this.selectedFile&&window.FileManager&&window.FileManager.openDeleteModal()}editSelectedUser(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedUser(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}approveSelectedComment(){var e=this.getSelectedRow();e&&(e=e.querySelector('.btn-approve, .btn-primary:contains("批准")'))&&e.click()}deleteSelectedComment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}editSelectedCategory(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedCategory(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}editSelectedTag(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedTag(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}viewSelectedAttachment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-view"))&&e.click()}editSelectedAttachment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedAttachment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}observeModals(){new MutationObserver(e=>{e.forEach(e=>{var t;"attributes"===e.type&&"class"===e.attributeName&&(t=e.target).classList&&t.classList.contains("modal")&&(t.classList.contains("active")&&!t.classList.contains("closing")?(this.activeModal=t,this.setupFocusTrap(t),this.focusFirstInput(t)):t.classList.contains("active")||this.activeModal===t&&(this.activeModal=null)),e.addedNodes.forEach(e=>{e.classList&&e.classList.contains("modal")&&e.classList.contains("active")&&(this.activeModal=e,this.setupFocusTrap(e),this.focusFirstInput(e))}),e.removedNodes.forEach(e=>{e.classList&&e.classList.contains("modal")&&this.activeModal===e&&(this.activeModal=null)})})}).observe(document.body,{childList:!0,subtree:!0,attributes:!0,attributeFilter:["class"]})}setupFocusTrap(i){if(i){var e=i.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'),e=Array.from(e).filter(e=>{if(e.disabled)return!1;if("-1"===e.getAttribute("tabindex"))return!1;var t=window.getComputedStyle(e);if("none"===t.display)return!1;if("hidden"===t.visibility)return!1;if("0"===t.opacity)return!1;t=e.getBoundingClientRect();if(0===t.width&&0===t.height)return!1;let s=e.parentElement;for(;s&&s!==i;){var a=window.getComputedStyle(s);if("none"===a.display||"hidden"===a.visibility)return!1;s=s.parentElement}return!0});if(0!==e.length){const t=e[0],s=e[e.length-1];e=e=>{"Tab"===e.key&&(e.shiftKey?document.activeElement===t&&(e.preventDefault(),s.focus()):document.activeElement===s&&(e.preventDefault(),t.focus()))};i.addEventListener("keydown",e),i._focusTrapHandler=e}}}handleTabNavigation(e){var t,s;this.activeModal&&(t=this.activeModal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'),0!==(t=Array.from(t).filter(e=>{if(e.disabled)return!1;if("-1"===e.getAttribute("tabindex"))return!1;var t=window.getComputedStyle(e);if("none"===t.display)return!1;if("hidden"===t.visibility)return!1;if("0"===t.opacity)return!1;t=e.getBoundingClientRect();if(0===t.width&&0===t.height)return!1;let s=e.parentElement;for(;s&&s!==this.activeModal;){var a=window.getComputedStyle(s);if("none"===a.display||"hidden"===a.visibility)return!1;s=s.parentElement}return!0})).length)&&(s=document.activeElement,s=t.indexOf(s),(e.shiftKey?(e.preventDefault(),s<=0?t[t.length-1]:t[s-1]):(e.preventDefault(),-1===s||s>=t.length-1?t[0]:t[s+1])).focus())}focusFirstInput(e){if(e){const t=e.querySelectorAll('input[type="text"], input[type="email"], input[type="password"], input[type="number"], input[type="url"], textarea, select');0<t.length&&setTimeout(()=>{var e=t[0];e.focus(),"text"!==e.type&&"TEXTAREA"!==e.tagName||e.select()},300)}}closeCurrentModal(){var e;this.activeModal&&(this.activeModal._focusTrapHandler&&(this.activeModal.removeEventListener("keydown",this.activeModal._focusTrapHandler),delete this.activeModal._focusTrapHandler),(e=this.activeModal.querySelector(".modal-close"))?e.click():this.activeModal.classList.remove("active"))}observeTabs(){const t=new MutationObserver(e=>{e.forEach(e=>{"attributes"===e.type&&"class"===e.attributeName&&(e=e.target)&&e.classList&&e.classList.contains("tab-btn")&&e.classList.contains("active")&&(this.currentTab=e.dataset.tab)})});document.querySelectorAll(".tab-btn").forEach(e=>{e&&t.observe(e,{attributes:!0})})}showAdminShortcutHelp(){var e=document.createElement("div");e.className="modal active",e.innerHTML=`
      <div class="modal-content" style="max-width: 650px;">
        <div class="modal-header">
          <h3>管理员聚焦模式快捷键</h3>
          <button class="modal-close" onclick="this.closest('.modal').remove()">×</button>
        </div>
        <div class="modal-body">
          
      <div style="padding: 20px; max-width: 600px;">
        <h3 style="margin-bottom: 15px; color: rgba(255,255,255,0.9);">管理员聚焦模式快捷键</h3>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">聚焦模式控制</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd style="background: rgba(255,183,122,0.2); padding: 2px 8px; border-radius: 4px; border: 1px solid rgba(255,183,122,0.5);">i</kbd> - 进入聚焦模式</li>
          <li><kbd style="background: rgba(255,183,122,0.2); padding: 2px 8px; border-radius: 4px; border: 1px solid rgba(255,183,122,0.5);">q</kbd> - 退出聚焦模式</li>
          <li><kbd style="background: rgba(255,183,122,0.2); padding: 2px 8px; border-radius: 4px; border: 1px solid rgba(255,183,122,0.5);">Esc</kbd> - 退出聚焦模式/关闭模态框</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">标签页切换</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>1</kbd> - 文章管理</li>
          <li><kbd>2</kbd> - 用户管理</li>
          <li><kbd>3</kbd> - 评论管理</li>
          <li><kbd>4</kbd> - 分类管理</li>
          <li><kbd>5</kbd> - 标签管理</li>
          <li><kbd>6</kbd> - 统计分析</li>
          <li><kbd>7</kbd> - 关于页面</li>
          <li><kbd>8</kbd> - 文件管理</li>
          <li><kbd>9</kbd> - 附件管理</li>
          <li><kbd>0</kbd> - 系统设置</li>
          <li><kbd>← →</kbd> - 切换标签页</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">通用操作</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>r</kbd> - 刷新当前数据</li>
          <li><kbd>n</kbd> - 新建项目</li>
          <li><kbd>u</kbd> - 上传</li>
          <li><kbd>f</kbd> - 搜索/筛选</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">表格行导航</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>↑</kbd> - 选择上一行</li>
          <li><kbd>↓</kbd> - 选择下一行</li>
          <li><kbd>Home</kbd> - 跳转到第一行</li>
          <li><kbd>End</kbd> - 跳转到最后一行</li>
          <li><kbd>PageUp</kbd> - 向上翻页（10行）</li>
          <li><kbd>PageDown</kbd> - 向下翻页（10行）</li>
          <li><kbd>Enter</kbd> - 激活选中行（执行默认操作）</li>
          <li><kbd>Space</kbd> - 切换选中状态</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">文章管理</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>e</kbd> - 编辑选中文章</li>
          <li><kbd>d</kbd> - 删除选中文章</li>
          <li><kbd>v</kbd> - 查看详情</li>
          <li><kbd>a</kbd> - 上传附件</li>
          <li><kbd>p</kbd> - 发布文章</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">分类/标签管理</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>e</kbd> - 编辑选中项</li>
          <li><kbd>d</kbd> - 删除选中项</li>
          <li><kbd>a</kbd> - 添加分类/标签</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">关于页面</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>e</kbd> - 编辑选中卡片</li>
          <li><kbd>d</kbd> - 禁用/启用卡片</li>
          <li><kbd>c</kbd> - 删除卡片</li>
          <li><kbd>Tab</kbd> - 在主卡片和次卡片表格间切换</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">附件管理</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>v</kbd> - 查看详情</li>
          <li><kbd>e</kbd> - 编辑附件</li>
          <li><kbd>d</kbd> - 删除附件</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">系统设置</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>1</kbd> - 外观设置（背景、透明度、毛玻璃颜色等）</li>
          <li><kbd>2</kbd> - 音乐设置（播放器、控件、位置等）</li>
          <li><kbd>3</kbd> - 模板设置（标题、欢迎语、年份等）</li>
          <li><kbd>4</kbd> - 文章标题设置</li>
          <li><kbd>5</kbd> - 切换界面提示设置</li>
          <li><kbd>6</kbd> - 外部链接设置</li>
          <li><kbd>7</kbd> - 赞助设置</li>
          <li><kbd>s</kbd> - 保存当前区块设置（在输入框中也可用）</li>
          <li><kbd>r</kbd> - 重置为默认设置</li>
          <li><kbd>q</kbd> - 退出聚焦模式（在输入框中也可用）</li>
          <li><kbd>?</kbd> - 显示设置快捷键帮助</li>
          <li><kbd>Tab</kbd> - 在表单控件间导航</li>
          <li><kbd>Shift+Tab</kbd> - 反向导航</li>
          <li><kbd>Space</kbd> - 切换复选框</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">文件管理</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>Enter</kbd> - 打开选中项</li>
          <li><kbd>Backspace</kbd> - 返回上级目录</li>
          <li><kbd>r</kbd> - 重命名</li>
          <li><kbd>Delete</kbd> - 删除</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">模态框操作</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>y</kbd> - 确认/保存</li>
          <li><kbd>c</kbd> - 取消/关闭</li>
          <li><kbd>Esc</kbd> - 关闭模态框</li>
          <li><kbd>Enter</kbd> - 确认/主要操作</li>
          <li><kbd>s</kbd> - 保存/提交</li>
          <li><kbd>Tab</kbd> - 在元素间导航</li>
          <li><kbd>Shift+Tab</kbd> - 反向导航</li>
          <li><kbd>Space</kbd> - 切换复选框/单选框</li>
        </ul>
      </div>
    
        </div>
      </div>
    `,document.body.appendChild(e)}showToast(e,t="info"){const s=document.createElement("div");s.className="toast "+t,s.innerHTML=`
      <span class="toast-icon">${this.getToastIcon(t)}</span>
      <span class="toast-message">${e}</span>
      <button class="toast-close">&times;</button>
    `;t=document.getElementById("toastContainer");(t||((e=document.createElement("div")).id="toastContainer",e.className="toast-container",document.body.appendChild(e),e)).appendChild(s),setTimeout(()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)},2e3),s.querySelector(".toast-close").addEventListener("click",()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)})}getToastIcon(e){var t={success:"✓",error:"✕",warning:"⚠",info:"ℹ"};return t[e]||t.info}}"loading"===document.readyState?document.addEventListener("DOMContentLoaded",()=>{window.keyboardShortcuts=new KeyboardShortcuts,window.adminKeyboardManager=new AdminKeyboardManager}):(window.keyboardShortcuts=new KeyboardShortcuts,window.adminKeyboardManager=new AdminKeyboardManager);
