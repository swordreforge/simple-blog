/* ESBuild compressed */
class u{constructor(){this.shortcuts={1:{action:"navigate",url:"/",label:"\u4E3B\u9875"},2:{action:"navigate",url:"/passage",label:"\u6587\u7AE0"},3:{action:"navigate",url:"/collect",label:"\u5F52\u6863"},4:{action:"navigate",url:"/about",label:"\u5173\u4E8E"},5:{action:"openModal",modalId:"userCenterModal",label:"\u4E2A\u4EBA\u4E2D\u5FC3"},6:{action:"navigate",url:"/markdown-editor",label:"\u7F16\u8F91\u5668"},f:{action:"navigate",url:"/friends",label:"\u53CB\u94FE"},l:{action:"openModal",modalId:"loginModal",label:"\u767B\u5F55"},"/":{action:"showHelp",label:"\u5FEB\u6377\u952E\u5E2E\u52A9"},Escape:{action:"closeAllModals",label:"\u5173\u95ED\u6A21\u6001\u6846"}," ":{action:"music",musicAction:"togglePlay",label:"\u64AD\u653E/\u6682\u505C"},ArrowLeft:{action:"music",musicAction:"previous",label:"\u4E0A\u4E00\u9996"},ArrowRight:{action:"music",musicAction:"next",label:"\u4E0B\u4E00\u9996"},ArrowUp:{action:"music",musicAction:"volumeUp",label:"\u97F3\u91CF+"},ArrowDown:{action:"music",musicAction:"volumeDown",label:"\u97F3\u91CF-"},m:{action:"music",musicAction:"mute",label:"\u9759\u97F3"},p:{action:"music",musicAction:"playlist",label:"\u64AD\u653E\u5217\u8868"},a:{action:"navigate",url:"/admin",label:"\u7BA1\u7406\u5458\u8BBE\u7F6E",adminOnly:!0}},this.enabled=!0,this.init()}init(){document.addEventListener("keydown",e=>this.handleKeyPress(e)),this.showShortcutHints(),this.addHelpButton()}handleKeyPress(e){if(this.enabled){var t=document.activeElement;if((!t||t.tagName!=="INPUT"&&t.tagName!=="TEXTAREA"&&!t.isContentEditable)&&(t=document.getElementById("musicPlaylist"),t=t&&t.classList.contains("show"),(!t||e.key!=="ArrowUp"&&e.key!=="ArrowDown")&&(!document.body.classList.contains("focus-mode")||e.key!=="ArrowUp"&&e.key!=="ArrowDown"&&e.key!=="ArrowLeft"&&e.key!=="ArrowRight")&&(t=document.body.classList.contains("collect-focus-mode"),(!t||e.key!=="ArrowUp"&&e.key!=="ArrowDown"&&e.key!=="ArrowLeft"&&e.key!=="ArrowRight")&&(!document.body.classList.contains("about-focus-mode")||e.key!=="ArrowUp"&&e.key!=="ArrowDown")))){var t=e.key;let i=null;this.shortcuts[t]?i=this.shortcuts[t]:(t={49:"1",50:"2",51:"3",52:"4",53:"5",54:"6",76:"l"}[e.keyCode])&&this.shortcuts[t]&&(i=this.shortcuts[t]),i&&(e.preventDefault(),e.stopPropagation(),i.adminOnly&&!this.isAdmin()?this.showToast("\u6B64\u5FEB\u6377\u952E\u4EC5\u7BA1\u7406\u5458\u53EF\u7528","warning"):this.executeAction(i))}}}executeAction(e){switch(e.action){case"navigate":window.location.pathname!==e.url&&(window.location.href=e.url);break;case"openModal":var t=document.getElementById(e.modalId);t&&(t.classList.add("active"),this.showToast("\u5DF2\u6253\u5F00: "+e.label,"success"));break;case"closeAllModals":document.querySelectorAll(".modal.active").forEach(s=>{s.classList.remove("active")}),this.showToast("\u5DF2\u5173\u95ED\u6240\u6709\u6A21\u6001\u6846","success");break;case"showHelp":this.showHelpModal(),this.showToast("\u5FEB\u6377\u952E\u5E2E\u52A9","success");break;case"music":this.executeMusicAction(e.musicAction,e.label)}}executeMusicAction(e,t){if(window.musicPlayer&&window.musicPlayer.settings&&window.musicPlayer.settings.enabled){var s,i,a,r,l=window.musicPlayer;try{switch(e){case"togglePlay":l.togglePlay(),this.showToast(l.isPlaying?"\u6B63\u5728\u64AD\u653E":"\u5DF2\u6682\u505C","success");break;case"previous":l.playPrevious(),this.showToast("\u4E0A\u4E00\u9996","success");break;case"next":l.playNext(),this.showToast("\u4E0B\u4E00\u9996","success");break;case"volumeUp":l.audio&&(s=Math.min(100,100*l.audio.volume+10),l.audio.volume=s/100,(i=document.querySelector("#volumeBar"))&&(i.value=s),l.saveState(),this.showToast(`\u97F3\u91CF: ${Math.round(s)}%`,"success"));break;case"volumeDown":l.audio&&(a=Math.max(0,100*l.audio.volume-10),l.audio.volume=a/100,(r=document.querySelector("#volumeBar"))&&(r.value=a),l.saveState(),this.showToast(`\u97F3\u91CF: ${Math.round(a)}%`,"success"));break;case"mute":l.toggleMute(),this.showToast(l.audio.muted?"\u5DF2\u9759\u97F3":"\u5DF2\u53D6\u6D88\u9759\u97F3","success");break;case"playlist":l.togglePlaylist(),this.showToast("\u64AD\u653E\u5217\u8868","success")}}catch(o){console.error("[\u97F3\u4E50\u64AD\u653E\u5668\u5FEB\u6377\u952E\u9519\u8BEF]",o),this.showToast("\u64CD\u4F5C\u5931\u8D25: "+o.message,"error")}}else this.showToast("\u97F3\u4E50\u64AD\u653E\u5668\u672A\u542F\u7528","warning")}isAdmin(){var e=document.querySelectorAll(".admin-only");return Array.from(e).some(t=>window.getComputedStyle(t).display!=="none")}isPassagePage(){return window.location.pathname==="/passage"||window.location.pathname.startsWith("/passage/")}isCollectPage(){return window.location.pathname==="/collect"||window.location.pathname.startsWith("/collect/")}isAboutPage(){return window.location.pathname==="/about"}showShortcutHints(){/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)||window.innerWidth<=768||document.querySelectorAll("nav a, nav button").forEach(e=>{const t=e.getAttribute("href"),s=e.getAttribute("id");let i=null,a=null;var r;Object.entries(this.shortcuts).forEach(([l,o])=>{(o.action==="navigate"&&o.url===t||o.action==="openModal"&&o.modalId===s)&&(i=l,a=o.label)}),i&&a&&((r=e.querySelector(".shortcut-hint"))||((r=document.createElement("span")).className="shortcut-hint",r.textContent=i,e.appendChild(r)))})}addHelpButton(){var e=document.createElement("button"),t=(e.className="shortcuts-help-btn",e.innerHTML=`
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"></circle>
        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
        <line x1="12" y1="17" x2="12.01" y2="17"></line>
      </svg>
      \u5FEB\u6377\u952E
      <span class="shortcut-hint">/</span>
    `,e.addEventListener("click",()=>this.showHelpModal()),document.querySelector("nav"));t&&t.appendChild(e)}showHelpModal(){const e=document.createElement("div");e.className="modal shortcuts-help-modal active",e.innerHTML=`
      <div class="modal-content">
        <div class="modal-header">
          <h3>\u952E\u76D8\u5FEB\u6377\u952E</h3>
          <button class="modal-close">&times;</button>
        </div>
        <div class="modal-body">
          <div class="shortcuts-list">
            <h4>\u5BFC\u822A\u5FEB\u6377\u952E</h4>
            ${this.renderShortcutList(["1","2","3","4","6"])}

            <h4>\u529F\u80FD\u5FEB\u6377\u952E</h4>
            ${this.renderShortcutList(["5","f","l","/","Escape"])}

            ${this.isPassagePage()?`
            <h4>\u6587\u7AE0\u9875\u9762 - \u6587\u672C\u805A\u7126\u6A21\u5F0F</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">i</kbd>
              <span class="shortcut-label">\u8FDB\u5165\u6587\u672C\u805A\u7126\u6A21\u5F0F</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">\u9000\u51FA\u6587\u672C\u805A\u7126\u6A21\u5F0F</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">ESC</kbd>
              <span class="shortcut-label">\u6682\u65F6\u9000\u51FA\u805A\u7126\u6A21\u5F0F\uFF08\u53EF\u5173\u95ED\u6A21\u6001\u6846\uFF09</span>
            </div>
            <div class="shortcut-description">
              \u805A\u7126\u6A21\u5F0F\u4E0B\uFF1A\u2190 \u2192 \u5207\u6362\u9762\u677F\uFF0C\u2191 \u2193 \u5BFC\u822A\uFF0CEnter \u6FC0\u6D3B\uFF0Cu \u5C55\u5F00/\u6298\u53E0
            </div>
            `:""}

            ${this.isCollectPage()?`
            <h4>\u5F52\u6863\u9875\u9762 - \u805A\u7126\u6A21\u5F0F</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">i</kbd>
              <span class="shortcut-label">\u8FDB\u5165\u805A\u7126\u6A21\u5F0F</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">\u9000\u51FA\u805A\u7126\u6A21\u5F0F</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">ESC</kbd>
              <span class="shortcut-label">\u8FD4\u56DE\u4E0A\u4E00\u7EA7\u6216\u6682\u65F6\u9000\u51FA</span>
            </div>
            <div class="shortcut-description">
              \u805A\u7126\u6A21\u5F0F\u4E0B\uFF1A\u2191 \u2193 \u2190 \u2192 \u5BFC\u822A\uFF0CEnter \u8FDB\u5165\u5B50\u83DC\u5355/\u6FC0\u6D3B\uFF0CESC \u8FD4\u56DE
            </div>
            `:""}

            ${this.isAboutPage()?`
            <h4>\u5173\u4E8E\u9875\u9762 - \u805A\u7126\u6A21\u5F0F</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">i</kbd>
              <span class="shortcut-label">\u8FDB\u5165\u805A\u7126\u6A21\u5F0F</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">\u9000\u51FA\u805A\u7126\u6A21\u5F0F</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">ESC</kbd>
              <span class="shortcut-label">\u6682\u65F6\u9000\u51FA\u805A\u7126\u6A21\u5F0F\uFF08\u53EF\u5173\u95ED\u6A21\u6001\u6846\uFF09</span>
            </div>
            <div class="shortcut-description">
              \u805A\u7126\u6A21\u5F0F\u4E0B\uFF1A\u2191 \u2193 \u5BFC\u822A\u5361\u7247\uFF0CEnter \u67E5\u770B\u5361\u7247\u5185\u5BB9
            </div>
            `:""}

            <h4>\u97F3\u4E50\u64AD\u653E\u5668\u5FEB\u6377\u952E</h4>
            ${this.renderShortcutList([" ","ArrowLeft","ArrowRight","ArrowUp","ArrowDown","m","p"])}

            ${this.isAdmin()?"<h4>\u7BA1\u7406\u5458\u5FEB\u6377\u952E</h4>":""}
            ${this.isAdmin()?this.renderShortcutList(["a"]):""}
          </div>
        </div>
      </div>
    `,document.body.appendChild(e),e.querySelector(".modal-close").addEventListener("click",()=>{e.classList.remove("active"),setTimeout(()=>e.remove(),300)}),e.addEventListener("click",s=>{s.target===e&&(e.classList.remove("active"),setTimeout(()=>e.remove(),300))});const t=s=>{s.key==="Escape"&&(e.classList.remove("active"),setTimeout(()=>e.remove(),300),document.removeEventListener("keydown",t))};document.addEventListener("keydown",t)}renderShortcutList(e){return e.map(t=>{var s=this.shortcuts[t];if(!s)return"";let i=t;return t===" "?i="Space":t==="ArrowLeft"?i="\u2190":t==="ArrowRight"?i="\u2192":t==="ArrowUp"?i="\u2191":t==="ArrowDown"&&(i="\u2193"),`
        <div class="shortcut-item">
          <kbd class="shortcut-key">${i}</kbd>
          <span class="shortcut-label">${s.label}</span>
        </div>
      `}).join("")}showToast(e,t="info"){const s=document.createElement("div");s.className="toast "+t,s.innerHTML=`
      <span class="toast-icon">${this.getToastIcon(t)}</span>
      <span class="toast-message">${e}</span>
      <button class="toast-close">&times;</button>
    `,t=document.getElementById("toastContainer"),(t||((e=document.createElement("div")).id="toastContainer",e.className="toast-container",document.body.appendChild(e),e)).appendChild(s),setTimeout(()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)},2e3),s.querySelector(".toast-close").addEventListener("click",()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)})}getToastIcon(e){var t={success:"\u2713",error:"\u2715",warning:"\u26A0",info:"\u2139"};return t[e]||t.info}enable(){this.enabled=!0}disable(){this.enabled=!1}}class h{constructor(){this.isFocusMode=!1,this.currentTab="articles",this.activeModal=null,this.selectedRows=new Set,this.selectedFile=null,this.currentPath="/",this.aboutCurrentTable="main",this.tabs=["articles","users","comments","categories","tags","analytics","about","filemanager","attachments","settings"],this.init()}init(){this.isAdminPage()&&(document.addEventListener("keydown",this.handleKeyDown.bind(this)),document.addEventListener("keyup",this.handleKeyUp.bind(this)),this.observeModals(),this.observeTabs(),console.log("[\u7BA1\u7406\u5458\u5FEB\u6377\u952E] \u5DF2\u521D\u59CB\u5316"))}isAdminPage(){return window.location.pathname==="/admin"||window.location.pathname.startsWith("/admin")}handleKeyDown(e){if(this.isInputElement(e.target))return this.handleInputShortcuts(e);var t=this.getKeyString(e);this.handleGlobalShortcuts(t,e)||!this.isFocusMode||this.activeModal&&this.handleModalShortcuts(t,e)||this.handleTabShortcuts(t,e)||this.handleFocusModeShortcuts(t,e)}handleKeyUp(e){}isInputElement(e){return e&&(e.tagName==="INPUT"||e.tagName==="TEXTAREA"||e.tagName==="SELECT"||e.isContentEditable)}handleInputShortcuts(e){const t=this.getKeyString(e);if(t==="Escape"){if(this.activeModal)return this.closeCurrentModal(),e.preventDefault(),!0;if(this.isFocusMode)return this.exitFocusMode(),e.preventDefault(),!0}if(this.currentTab==="settings"&&this.isFocusMode){if(t==="q")return e.target.blur(),this.exitFocusMode(),e.preventDefault(),!0;if(t==="s")return e.target.blur(),setTimeout(()=>{var i;var s=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(s&&(s=this.getVisibleSettingsSection(s),s)){let a=null;s=((i=s.querySelector("h4"))==null?void 0:i.textContent)||"",s.includes("\u5916\u89C2")?a=document.getElementById("saveSettingsBtn"):s.includes("\u97F3\u4E50")?a=document.getElementById("saveMusicSettingsBtn"):(s.includes("\u6A21\u677F")||s.includes("\u6587\u7AE0\u6807\u9898")||s.includes("\u5207\u6362\u754C\u9762")||s.includes("\u5916\u90E8\u94FE\u63A5")||s.includes("\u8D5E\u52A9"))&&(a=document.getElementById("saveTemplateSettingsBtn")),a&&a.click()}},100),e.preventDefault(),!0;if("1"<=t&&t<="7")return e.target.blur(),setTimeout(()=>{var s,i=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);i&&(s=parseInt(t),i=i.querySelector(`.settings-section:nth-of-type(${s})`))&&(i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u8BBE\u7F6E\u533A\u5757 "+t,"success"))},100),e.preventDefault(),!0}return!1}getKeyString(e){let t=e.key;return t=t===" "?"Space":t}handleGlobalShortcuts(e,t){switch(e){case"i":if(this.activeModal)break;return this.enterFocusMode(),t.preventDefault(),!0;case"q":return this.exitFocusMode(),t.preventDefault(),!0;case"Escape":return this.activeModal?this.closeCurrentModal():this.isFocusMode&&this.exitFocusMode(),t.preventDefault(),!0}return!1}handleModalShortcuts(e,t){if(this.activeModal){var s=document.activeElement,i=this.isInputElement(s);switch(e){case"Escape":return this.closeCurrentModal(),t.preventDefault(),!0;case"Enter":if(i&&s.tagName==="TEXTAREA"&&t.shiftKey)return!1;if(i){var a=s.closest("form");if(a&&(a=a.querySelector('button[type="submit"], .btn-primary'),a)||(a=this.activeModal.querySelector(".btn-primary"),a))return a.click(),t.preventDefault(),!0}else if(a=this.activeModal.querySelector(".btn-primary"),a)return a.click(),t.preventDefault(),!0;break;case"s":if(a=this.activeModal.querySelector('button[type="submit"], .btn-primary'),a)return a.click(),t.preventDefault(),!0;break;case"y":let r=this.activeModal.querySelector("#confirmAction");if((r=(r=r||this.activeModal.querySelector('button[type="submit"]'))||this.activeModal.querySelector(".btn-primary"))&&(a=window.getComputedStyle(r),a.display!=="none"&&a.visibility!=="hidden"&&!r.disabled))return r.click(),t.preventDefault(),!0;break;case"c":return a=this.activeModal.querySelector(".btn-secondary, button[data-modal]"),a?a.click():this.closeCurrentModal(),t.preventDefault(),!0;case"Tab":return this.handleTabNavigation(t),!0;case" ":if(!s||s.type!=="radio"&&s.type!=="checkbox")break;return s.click(),t.preventDefault(),!0;case"ArrowDown":case"ArrowUp":if(i&&s.tagName==="SELECT")return!1}}return!1}handleTabShortcuts(e,t){if("0"<=e&&e<="9"&&this.currentTab!=="settings"){var s=e==="0"?9:parseInt(e)-1;if(s<this.tabs.length)return this.switchToTab(this.tabs[s]),t.preventDefault(),!0}if(this.currentTab==="settings"&&e==="Tab")return this.handleSettingsTabNavigation(t);if(this.handleRowNavigation(e,t))return!0;switch(e){case"ArrowRight":return this.nextTab(),t.preventDefault(),!0;case"ArrowLeft":return this.previousTab(),t.preventDefault(),!0;case"r":return this.refreshCurrentTab(),t.preventDefault(),!0;case"n":return this.createNewItem(),t.preventDefault(),!0;case"u":return this.uploadItem(),t.preventDefault(),!0;case"f":return this.openSearch(),t.preventDefault(),!0}return this.handleSpecificTabShortcuts(e,t)}handleSpecificTabShortcuts(e,t){switch(this.currentTab){case"articles":return this.handleArticleShortcuts(e,t);case"filemanager":return this.handleFileManagerShortcuts(e,t);case"users":return this.handleUserShortcuts(e,t);case"comments":return this.handleCommentShortcuts(e,t);case"categories":return this.handleCategoryShortcuts(e,t);case"tags":return this.handleTagShortcuts(e,t);case"attachments":return this.handleAttachmentShortcuts(e,t);case"about":return this.handleAboutShortcuts(e,t);case"settings":return this.handleSettingsShortcuts(e,t)}return!1}handleArticleShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"e":return this.editSelectedArticle(),t.preventDefault(),!0;case"d":return this.deleteSelectedArticle(),t.preventDefault(),!0;case"v":return this.viewSelectedArticle(),t.preventDefault(),!0;case"a":return this.attachToSelectedArticle(),t.preventDefault(),!0;case"p":return this.publishSelectedArticle(),t.preventDefault(),!0}return!1}handleFileManagerShortcuts(e,t){switch(e){case"Enter":return this.openSelectedFile(),t.preventDefault(),!0;case"Backspace":return this.goUpDirectory(),t.preventDefault(),!0;case"r":return this.selectedFile?this.renameSelectedFile():this.refreshCurrentTab(),t.preventDefault(),!0;case"Delete":return this.deleteSelectedFile(),t.preventDefault(),!0}return!1}handleUserShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"e":return this.editSelectedUser(),t.preventDefault(),!0;case"d":return this.deleteSelectedUser(),t.preventDefault(),!0}return!1}handleCommentShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"a":return this.approveSelectedComment(),t.preventDefault(),!0;case"d":return this.deleteSelectedComment(),t.preventDefault(),!0}return!1}handleCategoryShortcuts(e,t){switch(e){case"a":var s=document.getElementById("addCategoryBtn");if(s)return s.click(),t.preventDefault(),!0;break;case"e":return this.selectedRows.size||this.hasSelectedRow()?(this.editSelectedCategory(),t.preventDefault(),!0):!1;case"d":return this.selectedRows.size||this.hasSelectedRow()?(this.deleteSelectedCategory(),t.preventDefault(),!0):!1}return!1}handleTagShortcuts(e,t){switch(e){case"a":var s=document.getElementById("addTagBtn");if(s)return s.click(),t.preventDefault(),!0;break;case"e":return this.selectedRows.size||this.hasSelectedRow()?(this.editSelectedTag(),t.preventDefault(),!0):!1;case"d":return this.selectedRows.size||this.hasSelectedRow()?(this.deleteSelectedTag(),t.preventDefault(),!0):!1}return!1}handleAttachmentShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow())switch(e){case"v":return this.viewSelectedAttachment(),t.preventDefault(),!0;case"e":return this.editSelectedAttachment(),t.preventDefault(),!0;case"d":return this.deleteSelectedAttachment(),t.preventDefault(),!0}return!1}handleAboutShortcuts(e,t){if(this.selectedRows.size||this.hasSelectedRow()){var s=this.getSelectedRow();if(s)switch(e){case"e":var i=s.querySelector('button[onclick*="edit"]');if(i)return i.click(),t.preventDefault(),!0;break;case"d":if(i=s.querySelector('button[onclick*="toggle"]'),i)return i.click(),t.preventDefault(),!0;break;case"c":if(i=s.querySelector("button.btn-danger"),i)return i.click(),t.preventDefault(),!0}}return!1}handleSettingsShortcuts(e,t){var a;var s=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(s)switch(e){case"1":var i=s.querySelector(".settings-section:nth-of-type(1)");if(i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u5916\u89C2\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"2":if(i=s.querySelector(".settings-section:nth-of-type(2)"),i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u97F3\u4E50\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"3":if(i=s.querySelector(".settings-section:nth-of-type(3)"),i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u6A21\u677F\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"4":if(i=s.querySelector(".settings-section:nth-of-type(4)"),i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u6587\u7AE0\u6807\u9898\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"5":if(i=s.querySelector(".settings-section:nth-of-type(5)"),i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u5207\u6362\u754C\u9762\u63D0\u793A\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"6":if(i=s.querySelector(".settings-section:nth-of-type(6)"),i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u5916\u90E8\u94FE\u63A5\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"7":if(i=s.querySelector(".settings-section:nth-of-type(7)"),i)return i.scrollIntoView({behavior:"smooth",block:"start"}),this.focusFirstInputInSection(i),this.showToast("\u5DF2\u8DF3\u8F6C\u5230\u8D5E\u52A9\u8BBE\u7F6E","success"),t.preventDefault(),!0;break;case"s":if(i=this.getVisibleSettingsSection(s),i){let r=null;if(i=((a=i.querySelector("h4"))==null?void 0:a.textContent)||"",i.includes("\u5916\u89C2")?r=document.getElementById("saveSettingsBtn"):i.includes("\u97F3\u4E50")?r=document.getElementById("saveMusicSettingsBtn"):(i.includes("\u6A21\u677F")||i.includes("\u6587\u7AE0\u6807\u9898")||i.includes("\u5207\u6362\u754C\u9762")||i.includes("\u5916\u90E8\u94FE\u63A5")||i.includes("\u8D5E\u52A9"))&&(r=document.getElementById("saveTemplateSettingsBtn")),r)return r.click(),t.preventDefault(),!0}break;case"r":if(i=document.getElementById("resetSettingsBtn"),i)return i.click(),t.preventDefault(),!0;break;case"?":return this.showSettingsShortcutHelp(),t.preventDefault(),!0}return!1}getVisibleSettingsSection(t){var t=t.querySelectorAll(".settings-section"),s=window.innerHeight/2;for(const r of t){var i=r.getBoundingClientRect(),a=i.top+i.height/2;if(Math.abs(a-s)<i.height/2)return r}return null}focusFirstInputInSection(e){if(e){const t=e.querySelectorAll('input[type="text"], input[type="number"], input[type="color"], textarea, select, input[type="checkbox"]');0<t.length&&setTimeout(()=>{var s=t[0];s.focus(),s.scrollIntoView({behavior:"smooth",block:"center"})},300)}}handleSettingsTabNavigation(e){var t,s=document.querySelectorAll('#settings input[type="text"], #settings input[type="number"], #settings input[type="color"], #settings textarea, #settings select, #settings input[type="checkbox"], #settings input[type="range"]');return s.length!==0&&(t=document.activeElement,t=Array.from(s).indexOf(t),(e.shiftKey?(e.preventDefault(),t<=0?s[s.length-1]:s[t-1]):(e.preventDefault(),t===-1||t>=s.length-1?s[0]:s[t+1])).focus(),(e=document.activeElement)&&e.scrollIntoView({behavior:"smooth",block:"center"}),!0)}showSettingsShortcutHelp(){const e=document.createElement("div");e.className="modal shortcuts-help-modal active",e.innerHTML=`
      <div class="modal-content">
        <div class="modal-header">
          <h3>\u7CFB\u7EDF\u8BBE\u7F6E\u5FEB\u6377\u952E</h3>
          <button class="modal-close">&times;</button>
        </div>
        <div class="modal-body">
          <div class="shortcuts-list">
            <h4>\u533A\u5757\u5BFC\u822A</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">1</kbd>
              <span class="shortcut-label">\u5916\u89C2\u8BBE\u7F6E\uFF08\u80CC\u666F\u3001\u900F\u660E\u5EA6\u3001\u6BDB\u73BB\u7483\u989C\u8272\u7B49\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">2</kbd>
              <span class="shortcut-label">\u97F3\u4E50\u8BBE\u7F6E\uFF08\u64AD\u653E\u5668\u3001\u4E0A\u4F20\u3001\u64AD\u653E\u5217\u8868\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">3</kbd>
              <span class="shortcut-label">\u6A21\u677F\u8BBE\u7F6E\uFF08\u6807\u9898\u3001\u6B22\u8FCE\u8BED\u3001\u5E74\u4EFD\u3001\u5934\u50CF\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">4</kbd>
              <span class="shortcut-label">\u6587\u7AE0\u6807\u9898\u8BBE\u7F6E\uFF08\u663E\u793A\u3001\u524D\u7F00\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">5</kbd>
              <span class="shortcut-label">\u5207\u6362\u754C\u9762\u63D0\u793A\u8BBE\u7F6E</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">6</kbd>
              <span class="shortcut-label">\u5916\u90E8\u94FE\u63A5\u8BBE\u7F6E\uFF08\u8B66\u544A\u3001\u767D\u540D\u5355\u3001Live2D\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">7</kbd>
              <span class="shortcut-label">\u8D5E\u52A9\u8BBE\u7F6E\uFF08\u6807\u9898\u3001\u56FE\u7247\u3001\u63CF\u8FF0\uFF09</span>
            </div>

            <h4>\u8868\u5355\u64CD\u4F5C</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">Tab</kbd>
              <span class="shortcut-label">\u5728\u8868\u5355\u63A7\u4EF6\u95F4\u5BFC\u822A\uFF08Shift+Tab \u53CD\u5411\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">Space</kbd>
              <span class="shortcut-label">\u5207\u6362\u590D\u9009\u6846\u9009\u4E2D\u72B6\u6001</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">\u2191</kbd>
              <kbd class="shortcut-key">\u2193</kbd>
              <span class="shortcut-label">\u5728\u4E0B\u62C9\u6846\u4E2D\u5207\u6362\u9009\u9879</span>
            </div>

            <h4>\u529F\u80FD\u5FEB\u6377\u952E</h4>
            <div class="shortcut-item">
              <kbd class="shortcut-key">s</kbd>
              <span class="shortcut-label">\u4FDD\u5B58\u5F53\u524D\u533A\u5757\u8BBE\u7F6E\uFF08\u5728\u8F93\u5165\u6846\u4E2D\u4E5F\u53EF\u7528\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">r</kbd>
              <span class="shortcut-label">\u91CD\u7F6E\u4E3A\u9ED8\u8BA4\u8BBE\u7F6E</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">q</kbd>
              <span class="shortcut-label">\u9000\u51FA\u805A\u7126\u6A21\u5F0F\uFF08\u5728\u8F93\u5165\u6846\u4E2D\u4E5F\u53EF\u7528\uFF09</span>
            </div>
            <div class="shortcut-item">
              <kbd class="shortcut-key">?</kbd>
              <span class="shortcut-label">\u663E\u793A\u6B64\u5E2E\u52A9</span>
            </div>

            <h4>\u63D0\u793A</h4>
            <div class="shortcut-description">
              \u2022 \u6570\u5B57\u952E\u53EF\u5728\u7F16\u8F91\u8F93\u5165\u6846\u65F6\u76F4\u63A5\u4F7F\u7528\uFF0C\u65E0\u9700\u5148\u79FB\u51FA\u7126\u70B9<br>
              \u2022 \u6309 s \u952E\u4FDD\u5B58\u65F6\uFF0C\u8F93\u5165\u6846\u4F1A\u81EA\u52A8\u5931\u53BB\u7126\u70B9\u5E76\u66F4\u65B0\u503C<br>
              \u2022 Tab \u952E\u53EA\u5BFC\u822A\u5230\u8868\u5355\u63A7\u4EF6\uFF0C\u4F1A\u8DF3\u8FC7\u64CD\u4F5C\u6309\u94AE
            </div>
          </div>
        </div>
      </div>
    `,document.body.appendChild(e),e.querySelector(".modal-close").addEventListener("click",()=>{e.classList.remove("active"),setTimeout(()=>e.remove(),300)}),e.addEventListener("click",s=>{s.target===e&&(e.classList.remove("active"),setTimeout(()=>e.remove(),300))});const t=s=>{s.key==="Escape"&&(e.classList.remove("active"),setTimeout(()=>e.remove(),300),document.removeEventListener("keydown",t))};document.addEventListener("keydown",t)}handleRowNavigation(e,t){var s=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(s){if(this.currentTab==="about")return this.handleAboutRowNavigation(e,t,s);if(s=s.querySelector(".data-table"),s&&(s=s.querySelector("tbody"),s)){var i=Array.from(s.querySelectorAll("tr"));if(i.length!==0){var a=this.getSelectedRow(),r=a?i.indexOf(a):-1;switch(e){case"ArrowUp":return t.preventDefault(),r<=0?this.selectRow(i[i.length-1]):this.selectRow(i[r-1]),!0;case"ArrowDown":return t.preventDefault(),r<0||r>=i.length-1?this.selectRow(i[0]):this.selectRow(i[r+1]),!0;case"Home":return t.preventDefault(),this.selectRow(i[0]),!0;case"End":return t.preventDefault(),this.selectRow(i[i.length-1]),!0;case"PageUp":t.preventDefault();var l=Math.max(0,r-10);return this.selectRow(i[l]),!0;case"PageDown":return t.preventDefault(),l=Math.min(i.length-1,r+10),this.selectRow(i[l]),!0;case"Enter":return t.preventDefault(),a&&this.activateSelectedRow(),!0;case" ":return t.preventDefault(),a&&this.toggleRowSelection(a),!0}}}}return!1}handleAboutRowNavigation(e,t,a){var i=a.querySelector("#mainCards"),a=a.querySelector("#subCards"),r=this.aboutCurrentTable==="main"?i:a,l=this.aboutCurrentTable==="main"?a:i;if(r&&(a=r.querySelector("tbody"),a)){var o=Array.from(a.querySelectorAll("tr"));if(o.length!==0){var d=this.getSelectedRow(),n=d?o.indexOf(d):-1;switch(e){case"ArrowUp":if(t.preventDefault(),n<=0){if(l){var c=l.querySelector("tbody"),c=Array.from(c.querySelectorAll("tr"));if(0<c.length)return this.aboutCurrentTable=this.aboutCurrentTable==="main"?"sub":"main",this.selectRow(c[c.length-1]),!0}this.selectRow(o[o.length-1])}else this.selectRow(o[n-1]);return!0;case"ArrowDown":if(t.preventDefault(),n<0||n>=o.length-1){if(l&&(c=l.querySelector("tbody"),c=Array.from(c.querySelectorAll("tr")),0<c.length))return this.aboutCurrentTable=this.aboutCurrentTable==="main"?"sub":"main",this.selectRow(c[0]),!0;this.selectRow(o[0])}else this.selectRow(o[n+1]);return!0;case"Tab":return t.preventDefault(),l&&(c=l.querySelector("tbody"),c=Array.from(c.querySelectorAll("tr")),0<c.length)?(this.aboutCurrentTable=this.aboutCurrentTable==="main"?"sub":"main",this.selectRow(c[0]),!0):!1;case"Home":return t.preventDefault(),this.selectRow(o[0]),!0;case"End":return t.preventDefault(),this.selectRow(o[o.length-1]),!0;case"PageUp":return t.preventDefault(),c=Math.max(0,n-10),this.selectRow(o[c]),!0;case"PageDown":return t.preventDefault(),c=Math.min(o.length-1,n+10),this.selectRow(o[c]),!0;case"Enter":return t.preventDefault(),d&&this.activateSelectedRow(),!0;case" ":return t.preventDefault(),d&&this.toggleRowSelection(d),!0}}}return!1}selectRow(e){var t;document.querySelectorAll(".data-table tbody tr").forEach(s=>s.classList.remove("selected")),e.classList.add("selected"),e.scrollIntoView({behavior:"smooth",block:"nearest"}),this.selectedRows.clear(),e=e.dataset.id||((t=e.querySelector("td:first-child"))==null?void 0:t.textContent),e&&this.selectedRows.add(e)}toggleRowSelection(e){var s,i;var t;e.classList.contains("selected")?(e.classList.remove("selected"),(t=e.dataset.id||((s=e.querySelector("td:first-child"))==null?void 0:s.textContent))&&this.selectedRows.delete(t)):(e.classList.add("selected"),(t=e.dataset.id||((i=e.querySelector("td:first-child"))==null?void 0:i.textContent))&&this.selectedRows.add(t))}activateSelectedRow(){var e=this.getSelectedRow();if(e)switch(this.currentTab){case"articles":this.viewSelectedArticle();break;case"filemanager":this.openSelectedFile();break;case"attachments":this.viewSelectedAttachment();break;default:var t=e.querySelector(".btn-edit");(t||(t=e.querySelector(".btn-view")))&&t.click()}}clearRowSelection(){document.querySelectorAll(".data-table tbody tr").forEach(e=>e.classList.remove("selected")),this.selectedRows.clear()}handleFocusModeShortcuts(e,t){switch(e){case"Tab":return t.preventDefault(),!0;case"?":return this.showAdminShortcutHelp(),t.preventDefault(),!0}return!1}enterFocusMode(){this.isFocusMode||(this.isFocusMode=!0,document.body.classList.add("admin-focus-mode"),window.keyboardShortcuts&&window.keyboardShortcuts.disable(),setTimeout(()=>{var e=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);e&&(e=e.querySelector(".data-table tbody"))&&(e=e.querySelector("tr"))&&this.selectRow(e)},100),this.showToast("\u5DF2\u8FDB\u5165\u7BA1\u7406\u5458\u805A\u7126\u6A21\u5F0F","success"),console.log("[\u7BA1\u7406\u5458\u5FEB\u6377\u952E] \u8FDB\u5165\u805A\u7126\u6A21\u5F0F"))}exitFocusMode(){this.isFocusMode&&(this.isFocusMode=!1,document.body.classList.remove("admin-focus-mode"),this.clearRowSelection(),window.keyboardShortcuts&&window.keyboardShortcuts.enable(),this.showToast("\u5DF2\u9000\u51FA\u7BA1\u7406\u5458\u805A\u7126\u6A21\u5F0F","info"),console.log("[\u7BA1\u7406\u5458\u5FEB\u6377\u952E] \u9000\u51FA\u805A\u7126\u6A21\u5F0F"))}switchToTab(e){var t=document.querySelector(`.tab-btn[data-tab="${e}"]`);t&&(t.click(),this.currentTab=e,this.clearRowSelection(),e==="about"&&(this.aboutCurrentTable="main"),console.log("[\u7BA1\u7406\u5458\u5FEB\u6377\u952E] \u5207\u6362\u5230\u6807\u7B7E\u9875: "+e))}nextTab(){var e=(this.tabs.indexOf(this.currentTab)+1)%this.tabs.length;this.switchToTab(this.tabs[e])}previousTab(){var e=(this.tabs.indexOf(this.currentTab)-1+this.tabs.length)%this.tabs.length;this.switchToTab(this.tabs[e])}refreshCurrentTab(){let e=null;if(e=this.currentTab==="articles"?document.getElementById("refreshArticlesBtn"):this.currentTab==="attachments"?document.getElementById("amRefreshBtn"):this.currentTab==="filemanager"?document.getElementById("fmRefreshBtn"):document.querySelector(`#${this.currentTab}RefreshBtn, .refresh-btn`))e.click(),this.showToast("\u5DF2\u5237\u65B0","success");else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const i of t.querySelectorAll("button")){var s=i.textContent.trim();if(s==="\u5237\u65B0"||s.includes("\u5237\u65B0"))return i.click(),void this.showToast("\u5DF2\u5237\u65B0","success")}this.currentTab==="articles"&&typeof loadPassages=="function"?(loadPassages(),this.showToast("\u5DF2\u5237\u65B0","success")):this.currentTab==="attachments"&&typeof loadAttachments=="function"?(loadAttachments(),this.showToast("\u5DF2\u5237\u65B0","success")):this.currentTab==="filemanager"&&window.FileManager&&(FileManager.loadFiles(),this.showToast("\u5DF2\u5237\u65B0","success"))}}createNewItem(){let e=null;if(e=this.currentTab==="articles"?document.getElementById("newArticleBtn"):this.currentTab==="users"?document.getElementById("newUserBtn"):this.currentTab==="categories"?document.getElementById("newCategoryBtn"):this.currentTab==="tags"?document.getElementById("newTagBtn"):document.querySelector(`#${this.currentTab}NewBtn, .new-btn`))e.click();else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const a of t.querySelectorAll("button")){var s=a.textContent.trim();if(s==="\u65B0\u5EFA\u6587\u7AE0"||s==="\u65B0\u5EFA\u7528\u6237"||s==="\u65B0\u5EFA\u5206\u7C7B"||s==="\u65B0\u5EFA\u6807\u7B7E"||s.startsWith("\u65B0\u5EFA")||s.startsWith("\u521B\u5EFA")||s.startsWith("\u6DFB\u52A0"))return void a.click()}for(const a of document.querySelectorAll("button")){var i=a.textContent.trim();if(i==="\u65B0\u5EFA\u6587\u7AE0"||i==="\u65B0\u5EFA\u7528\u6237"||i==="\u65B0\u5EFA\u5206\u7C7B"||i==="\u65B0\u5EFA\u6807\u7B7E"||i.startsWith("\u65B0\u5EFA"))return void a.click()}this.showToast("\u672A\u627E\u5230\u65B0\u5EFA\u6309\u94AE","warning")}}uploadItem(){let e=null;if(e=this.currentTab==="attachments"?document.getElementById("amUploadBtn"):this.currentTab==="filemanager"?document.getElementById("fmUploadBtn"):document.querySelector(`#${this.currentTab}UploadBtn, .upload-btn`))e.click();else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t){for(const s of t.querySelectorAll("button"))if(s.textContent.trim().includes("\u4E0A\u4F20"))return void s.click()}for(const s of document.querySelectorAll("button"))if(s.textContent.trim().includes("\u4E0A\u4F20"))return void s.click();this.showToast("\u672A\u627E\u5230\u4E0A\u4F20\u6309\u94AE","warning")}}openSearch(){let e=null;if(e=this.currentTab==="attachments"?document.getElementById("amSearchInput"):this.currentTab==="filemanager"?document.getElementById("fmSearchInput"):this.currentTab==="articles"?document.getElementById("articlesSearchInput"):document.querySelector(`#${this.currentTab}SearchInput, .search-input, input[type="search"]`))e.focus(),this.showToast("\u5DF2\u805A\u7126\u5230\u641C\u7D22\u6846","success");else{var t=document.querySelector(`.tab-pane[data-tab="${this.currentTab}"], .tab-pane.active`);if(t)for(const i of t.querySelectorAll('input[type="text"], input[type="search"]')){var s=i.placeholder||"";if(s.includes("\u641C\u7D22")||s.includes("\u7B5B\u9009")||s.includes("\u67E5\u627E"))return i.focus(),void this.showToast("\u5DF2\u805A\u7126\u5230\u641C\u7D22\u6846","success")}this.showToast("\u672A\u627E\u5230\u641C\u7D22\u6846","warning")}}hasSelectedRow(){return document.querySelector(".data-table tr.selected")!==null}getSelectedRow(){return document.querySelector(".data-table tr.selected")}editSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}viewSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-view"))&&e.click()}attachToSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-upload"))&&e.click()}publishSelectedArticle(){var e=this.getSelectedRow();e&&(e=e.querySelector('.btn-publish, .btn-primary:contains("\u53D1\u5E03")'))&&e.click()}openSelectedFile(){this.selectedFile&&window.FileManager&&window.FileManager.openFile(this.selectedFile.path)}goUpDirectory(){window.FileManager&&window.FileManager.goBack()}renameSelectedFile(){this.selectedFile&&window.FileManager&&window.FileManager.openRenameModal()}deleteSelectedFile(){this.selectedFile&&window.FileManager&&window.FileManager.openDeleteModal()}editSelectedUser(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedUser(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}approveSelectedComment(){var e=this.getSelectedRow();e&&(e=e.querySelector('.btn-approve, .btn-primary:contains("\u6279\u51C6")'))&&e.click()}deleteSelectedComment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}editSelectedCategory(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedCategory(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}editSelectedTag(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedTag(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}viewSelectedAttachment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-view"))&&e.click()}editSelectedAttachment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-edit"))&&e.click()}deleteSelectedAttachment(){var e=this.getSelectedRow();e&&(e=e.querySelector(".btn-delete"))&&e.click()}observeModals(){new MutationObserver(e=>{e.forEach(t=>{var s;t.type==="attributes"&&t.attributeName==="class"&&(s=t.target).classList&&s.classList.contains("modal")&&(s.classList.contains("active")&&!s.classList.contains("closing")?(this.activeModal=s,this.setupFocusTrap(s),this.focusFirstInput(s)):s.classList.contains("active")||this.activeModal===s&&(this.activeModal=null)),t.addedNodes.forEach(i=>{i.classList&&i.classList.contains("modal")&&i.classList.contains("active")&&(this.activeModal=i,this.setupFocusTrap(i),this.focusFirstInput(i))}),t.removedNodes.forEach(i=>{i.classList&&i.classList.contains("modal")&&this.activeModal===i&&(this.activeModal=null)})})}).observe(document.body,{childList:!0,subtree:!0,attributes:!0,attributeFilter:["class"]})}setupFocusTrap(e){if(e){var t=e.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'),t=Array.from(t).filter(s=>{if(s.disabled||s.getAttribute("tabindex")==="-1")return!1;var i=window.getComputedStyle(s);if(i.display==="none"||i.visibility==="hidden"||i.opacity==="0"||(i=s.getBoundingClientRect(),i.width===0&&i.height===0))return!1;let a=s.parentElement;for(;a&&a!==e;){var r=window.getComputedStyle(a);if(r.display==="none"||r.visibility==="hidden")return!1;a=a.parentElement}return!0});if(t.length!==0){const s=t[0],i=t[t.length-1];t=a=>{a.key==="Tab"&&(a.shiftKey?document.activeElement===s&&(a.preventDefault(),i.focus()):document.activeElement===i&&(a.preventDefault(),s.focus()))},e.addEventListener("keydown",t),e._focusTrapHandler=t}}}handleTabNavigation(e){var t,s;this.activeModal&&(t=this.activeModal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'),(t=Array.from(t).filter(i=>{if(i.disabled||i.getAttribute("tabindex")==="-1")return!1;var a=window.getComputedStyle(i);if(a.display==="none"||a.visibility==="hidden"||a.opacity==="0"||(a=i.getBoundingClientRect(),a.width===0&&a.height===0))return!1;let r=i.parentElement;for(;r&&r!==this.activeModal;){var l=window.getComputedStyle(r);if(l.display==="none"||l.visibility==="hidden")return!1;r=r.parentElement}return!0})).length!==0)&&(s=document.activeElement,s=t.indexOf(s),(e.shiftKey?(e.preventDefault(),s<=0?t[t.length-1]:t[s-1]):(e.preventDefault(),s===-1||s>=t.length-1?t[0]:t[s+1])).focus())}focusFirstInput(e){if(e){const t=e.querySelectorAll('input[type="text"], input[type="email"], input[type="password"], input[type="number"], input[type="url"], textarea, select');0<t.length&&setTimeout(()=>{var s=t[0];s.focus(),s.type!=="text"&&s.tagName!=="TEXTAREA"||s.select()},300)}}closeCurrentModal(){var e;this.activeModal&&(this.activeModal._focusTrapHandler&&(this.activeModal.removeEventListener("keydown",this.activeModal._focusTrapHandler),delete this.activeModal._focusTrapHandler),(e=this.activeModal.querySelector(".modal-close"))?e.click():this.activeModal.classList.remove("active"))}observeTabs(){const e=new MutationObserver(t=>{t.forEach(s=>{s.type==="attributes"&&s.attributeName==="class"&&(s=s.target).classList&&s.classList.contains("tab-btn")&&s.classList.contains("active")&&(this.currentTab=s.dataset.tab,console.log("[\u7BA1\u7406\u5458\u5FEB\u6377\u952E] \u5F53\u524D\u6807\u7B7E\u9875: "+this.currentTab))})});document.querySelectorAll(".tab-btn").forEach(t=>{e.observe(t,{attributes:!0})})}showAdminShortcutHelp(){var e=document.createElement("div");e.className="modal active",e.innerHTML=`
      <div class="modal-content" style="max-width: 650px;">
        <div class="modal-header">
          <h3>\u7BA1\u7406\u5458\u805A\u7126\u6A21\u5F0F\u5FEB\u6377\u952E</h3>
          <button class="modal-close" onclick="this.closest('.modal').remove()">\xD7</button>
        </div>
        <div class="modal-body">
          
      <div style="padding: 20px; max-width: 600px;">
        <h3 style="margin-bottom: 15px; color: rgba(255,255,255,0.9);">\u7BA1\u7406\u5458\u805A\u7126\u6A21\u5F0F\u5FEB\u6377\u952E</h3>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u805A\u7126\u6A21\u5F0F\u63A7\u5236</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd style="background: rgba(255,183,122,0.2); padding: 2px 8px; border-radius: 4px; border: 1px solid rgba(255,183,122,0.5);">i</kbd> - \u8FDB\u5165\u805A\u7126\u6A21\u5F0F</li>
          <li><kbd style="background: rgba(255,183,122,0.2); padding: 2px 8px; border-radius: 4px; border: 1px solid rgba(255,183,122,0.5);">q</kbd> - \u9000\u51FA\u805A\u7126\u6A21\u5F0F</li>
          <li><kbd style="background: rgba(255,183,122,0.2); padding: 2px 8px; border-radius: 4px; border: 1px solid rgba(255,183,122,0.5);">Esc</kbd> - \u9000\u51FA\u805A\u7126\u6A21\u5F0F/\u5173\u95ED\u6A21\u6001\u6846</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u6807\u7B7E\u9875\u5207\u6362</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>1</kbd> - \u6587\u7AE0\u7BA1\u7406</li>
          <li><kbd>2</kbd> - \u7528\u6237\u7BA1\u7406</li>
          <li><kbd>3</kbd> - \u8BC4\u8BBA\u7BA1\u7406</li>
          <li><kbd>4</kbd> - \u5206\u7C7B\u7BA1\u7406</li>
          <li><kbd>5</kbd> - \u6807\u7B7E\u7BA1\u7406</li>
          <li><kbd>6</kbd> - \u7EDF\u8BA1\u5206\u6790</li>
          <li><kbd>7</kbd> - \u5173\u4E8E\u9875\u9762</li>
          <li><kbd>8</kbd> - \u6587\u4EF6\u7BA1\u7406</li>
          <li><kbd>9</kbd> - \u9644\u4EF6\u7BA1\u7406</li>
          <li><kbd>0</kbd> - \u7CFB\u7EDF\u8BBE\u7F6E</li>
          <li><kbd>\u2190 \u2192</kbd> - \u5207\u6362\u6807\u7B7E\u9875</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u901A\u7528\u64CD\u4F5C</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>r</kbd> - \u5237\u65B0\u5F53\u524D\u6570\u636E</li>
          <li><kbd>n</kbd> - \u65B0\u5EFA\u9879\u76EE</li>
          <li><kbd>u</kbd> - \u4E0A\u4F20</li>
          <li><kbd>f</kbd> - \u641C\u7D22/\u7B5B\u9009</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u8868\u683C\u884C\u5BFC\u822A</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>\u2191</kbd> - \u9009\u62E9\u4E0A\u4E00\u884C</li>
          <li><kbd>\u2193</kbd> - \u9009\u62E9\u4E0B\u4E00\u884C</li>
          <li><kbd>Home</kbd> - \u8DF3\u8F6C\u5230\u7B2C\u4E00\u884C</li>
          <li><kbd>End</kbd> - \u8DF3\u8F6C\u5230\u6700\u540E\u4E00\u884C</li>
          <li><kbd>PageUp</kbd> - \u5411\u4E0A\u7FFB\u9875\uFF0810\u884C\uFF09</li>
          <li><kbd>PageDown</kbd> - \u5411\u4E0B\u7FFB\u9875\uFF0810\u884C\uFF09</li>
          <li><kbd>Enter</kbd> - \u6FC0\u6D3B\u9009\u4E2D\u884C\uFF08\u6267\u884C\u9ED8\u8BA4\u64CD\u4F5C\uFF09</li>
          <li><kbd>Space</kbd> - \u5207\u6362\u9009\u4E2D\u72B6\u6001</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u6587\u7AE0\u7BA1\u7406</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>e</kbd> - \u7F16\u8F91\u9009\u4E2D\u6587\u7AE0</li>
          <li><kbd>d</kbd> - \u5220\u9664\u9009\u4E2D\u6587\u7AE0</li>
          <li><kbd>v</kbd> - \u67E5\u770B\u8BE6\u60C5</li>
          <li><kbd>a</kbd> - \u4E0A\u4F20\u9644\u4EF6</li>
          <li><kbd>p</kbd> - \u53D1\u5E03\u6587\u7AE0</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u5206\u7C7B/\u6807\u7B7E\u7BA1\u7406</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>e</kbd> - \u7F16\u8F91\u9009\u4E2D\u9879</li>
          <li><kbd>d</kbd> - \u5220\u9664\u9009\u4E2D\u9879</li>
          <li><kbd>a</kbd> - \u6DFB\u52A0\u5206\u7C7B/\u6807\u7B7E</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u5173\u4E8E\u9875\u9762</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>e</kbd> - \u7F16\u8F91\u9009\u4E2D\u5361\u7247</li>
          <li><kbd>d</kbd> - \u7981\u7528/\u542F\u7528\u5361\u7247</li>
          <li><kbd>c</kbd> - \u5220\u9664\u5361\u7247</li>
          <li><kbd>Tab</kbd> - \u5728\u4E3B\u5361\u7247\u548C\u6B21\u5361\u7247\u8868\u683C\u95F4\u5207\u6362</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u9644\u4EF6\u7BA1\u7406</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>v</kbd> - \u67E5\u770B\u8BE6\u60C5</li>
          <li><kbd>e</kbd> - \u7F16\u8F91\u9644\u4EF6</li>
          <li><kbd>d</kbd> - \u5220\u9664\u9644\u4EF6</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u7CFB\u7EDF\u8BBE\u7F6E</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>1</kbd> - \u5916\u89C2\u8BBE\u7F6E\uFF08\u80CC\u666F\u3001\u900F\u660E\u5EA6\u3001\u6BDB\u73BB\u7483\u989C\u8272\u7B49\uFF09</li>
          <li><kbd>2</kbd> - \u97F3\u4E50\u8BBE\u7F6E\uFF08\u64AD\u653E\u5668\u3001\u63A7\u4EF6\u3001\u4F4D\u7F6E\u7B49\uFF09</li>
          <li><kbd>3</kbd> - \u6A21\u677F\u8BBE\u7F6E\uFF08\u6807\u9898\u3001\u6B22\u8FCE\u8BED\u3001\u5E74\u4EFD\u7B49\uFF09</li>
          <li><kbd>4</kbd> - \u6587\u7AE0\u6807\u9898\u8BBE\u7F6E</li>
          <li><kbd>5</kbd> - \u5207\u6362\u754C\u9762\u63D0\u793A\u8BBE\u7F6E</li>
          <li><kbd>6</kbd> - \u5916\u90E8\u94FE\u63A5\u8BBE\u7F6E</li>
          <li><kbd>7</kbd> - \u8D5E\u52A9\u8BBE\u7F6E</li>
          <li><kbd>s</kbd> - \u4FDD\u5B58\u5F53\u524D\u533A\u5757\u8BBE\u7F6E\uFF08\u5728\u8F93\u5165\u6846\u4E2D\u4E5F\u53EF\u7528\uFF09</li>
          <li><kbd>r</kbd> - \u91CD\u7F6E\u4E3A\u9ED8\u8BA4\u8BBE\u7F6E</li>
          <li><kbd>q</kbd> - \u9000\u51FA\u805A\u7126\u6A21\u5F0F\uFF08\u5728\u8F93\u5165\u6846\u4E2D\u4E5F\u53EF\u7528\uFF09</li>
          <li><kbd>?</kbd> - \u663E\u793A\u8BBE\u7F6E\u5FEB\u6377\u952E\u5E2E\u52A9</li>
          <li><kbd>Tab</kbd> - \u5728\u8868\u5355\u63A7\u4EF6\u95F4\u5BFC\u822A</li>
          <li><kbd>Shift+Tab</kbd> - \u53CD\u5411\u5BFC\u822A</li>
          <li><kbd>Space</kbd> - \u5207\u6362\u590D\u9009\u6846</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u6587\u4EF6\u7BA1\u7406</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>Enter</kbd> - \u6253\u5F00\u9009\u4E2D\u9879</li>
          <li><kbd>Backspace</kbd> - \u8FD4\u56DE\u4E0A\u7EA7\u76EE\u5F55</li>
          <li><kbd>r</kbd> - \u91CD\u547D\u540D</li>
          <li><kbd>Delete</kbd> - \u5220\u9664</li>
        </ul>
        
        <h4 style="color: rgba(255,183,122,0.9); margin-top: 20px;">\u6A21\u6001\u6846\u64CD\u4F5C</h4>
        <ul style="color: rgba(255,255,255,0.7); line-height: 1.8;">
          <li><kbd>y</kbd> - \u786E\u8BA4/\u4FDD\u5B58</li>
          <li><kbd>c</kbd> - \u53D6\u6D88/\u5173\u95ED</li>
          <li><kbd>Esc</kbd> - \u5173\u95ED\u6A21\u6001\u6846</li>
          <li><kbd>Enter</kbd> - \u786E\u8BA4/\u4E3B\u8981\u64CD\u4F5C</li>
          <li><kbd>s</kbd> - \u4FDD\u5B58/\u63D0\u4EA4</li>
          <li><kbd>Tab</kbd> - \u5728\u5143\u7D20\u95F4\u5BFC\u822A</li>
          <li><kbd>Shift+Tab</kbd> - \u53CD\u5411\u5BFC\u822A</li>
          <li><kbd>Space</kbd> - \u5207\u6362\u590D\u9009\u6846/\u5355\u9009\u6846</li>
        </ul>
      </div>
    
        </div>
      </div>
    `,document.body.appendChild(e)}showToast(e,t="info"){const s=document.createElement("div");s.className="toast "+t,s.innerHTML=`
      <span class="toast-icon">${this.getToastIcon(t)}</span>
      <span class="toast-message">${e}</span>
      <button class="toast-close">&times;</button>
    `,t=document.getElementById("toastContainer"),(t||((e=document.createElement("div")).id="toastContainer",e.className="toast-container",document.body.appendChild(e),e)).appendChild(s),setTimeout(()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)},2e3),s.querySelector(".toast-close").addEventListener("click",()=>{s.classList.add("closing"),setTimeout(()=>s.remove(),300)})}getToastIcon(e){var t={success:"\u2713",error:"\u2715",warning:"\u26A0",info:"\u2139"};return t[e]||t.info}}document.readyState==="loading"?document.addEventListener("DOMContentLoaded",()=>{window.keyboardShortcuts=new u,window.adminKeyboardManager=new h}):(window.keyboardShortcuts=new u,window.adminKeyboardManager=new h);
