const FileManager={currentPath:"img",currentRoot:"img",selectedFile:null,selectedAttachmentId:null,filesToUpload:[],getAuthHeader(){return"Bearer "+this.getCookie("auth_token")},getCookie(e){e=("; "+document.cookie).split(`; ${e}=`);return 2===e.length?e.pop().split(";").shift():""},init(){this.bindEvents(),this.loadFiles()},bindEvents(){document.getElementById("backBtn").addEventListener("click",()=>this.goBack()),document.getElementById("uploadBtn").addEventListener("click",()=>this.openUploadModal()),document.getElementById("createDirBtn").addEventListener("click",()=>this.openCreateDirModal()),document.querySelectorAll(".fm-root-btn").forEach(e=>{e.addEventListener("click",e=>{e=e.currentTarget.dataset.path;this.switchRoot(e)})}),document.querySelectorAll(".modal-close, .fm-modal-close-btn, .fm-modal-close").forEach(e=>{e.addEventListener("click",e=>{e=e.target.closest(".modal")||e.target.closest(".fm-modal");e&&this.closeModal(e)})});const t=document.getElementById("uploadArea"),i=document.getElementById("fileInput");t&&i?(t.addEventListener("click",e=>{e.stopPropagation(),e.preventDefault(),i.click()}),i.addEventListener("change",e=>{e.stopPropagation(),this.handleFileSelect(e)}),t.addEventListener("dragover",e=>{e.preventDefault(),e.stopPropagation(),t.classList.add("dragover")}),t.addEventListener("dragleave",e=>{e.preventDefault(),e.stopPropagation(),t.classList.remove("dragover")}),t.addEventListener("drop",e=>{e.preventDefault(),e.stopPropagation(),t.classList.remove("dragover"),this.handleFileDrop(e)})):console.error("上传区域或文件输入框未找到"),document.getElementById("confirmUploadBtn").addEventListener("click",()=>this.uploadFiles()),document.getElementById("confirmCreateDirBtn").addEventListener("click",()=>this.createDirectory()),document.getElementById("confirmRenameBtn").addEventListener("click",()=>this.renameFile()),document.getElementById("confirmDeleteBtn").addEventListener("click",()=>{this.selectedAttachmentId?this.deleteAttachment(this.selectedAttachmentId):this.deleteFile()}),document.addEventListener("click",e=>{e.target.closest(".context-menu")||e.target.closest(".file-item")||this.hideContextMenu()}),document.addEventListener("keydown",e=>{"Escape"===e.key&&(this.hideContextMenu(),document.querySelectorAll(".modal.active, .fm-modal.active").forEach(e=>{this.closeModal(e)}))})},async loadFiles(){if("attachments"===this.currentRoot)await this.loadAttachments();else try{var e=await(await fetch("/api/files?path="+encodeURIComponent(this.currentPath),{headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.renderFiles(e.data.files),this.updateBreadcrumb(e.data.current_path),this.updateBackButton(e.data.parent_path),this.updateFileCount(e.data.files.length)):this.showToast(e.message,"error")}catch(e){console.error("加载文件失败:",e),this.showToast("加载文件失败","error")}},async loadAttachments(){try{var e=await(await fetch("/api/admin/attachments",{headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.currentAttachments=e.data,this.renderAttachments(e.data),this.updateBreadcrumb("/attachments"),this.updateBackButton(null),this.updateFileCount(e.total)):this.showToast(e.message,"error")}catch(e){console.error("加载附件失败:",e),this.showToast("加载附件失败","error")}},renderAttachments(e){var t=document.getElementById("fileGrid"),i=document.getElementById("emptyState");0===e.length?(t.innerHTML="",i.style.display="flex"):(i.style.display="none",t.innerHTML=e.map(e=>this.createAttachmentItem(e)).join(""),t.querySelectorAll(".file-item").forEach(i=>{i.addEventListener("click",e=>{e.stopPropagation();var t=i.dataset.id;this.showAttachmentMenu(t,e)})}))},createAttachmentItem(e){var t={public:'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>',private:'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>',protected:'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>'}[e.visibility]||'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>',i={public:"公开",private:"私密",protected:"受保护"}[e.visibility]||"公开",o=e.show_in_passage?'<span class="badge badge-success">显示</span>':'<span class="badge badge-secondary">隐藏</span>';return`
      <div class="file-item" data-id="${e.id}">
        <div class="file-icon">${this.getFileIcon(e.file_type)}</div>
        <div class="file-info">
          <div class="file-name">${e.file_name}</div>
          <div class="file-meta">
            <span>${t} ${i}</span>
            ${o}
            <span>${this.formatFileSize(e.file_size)}</span>
          </div>
        </div>
      </div>
    `},showAttachmentMenu(i,e){var t=this.currentAttachments?.find(e=>e.id===parseInt(i));if(t){const o=document.createElement("div");o.className="context-menu",o.style.position="absolute",o.style.left=e.clientX+"px",o.style.top=e.clientY+"px",o.innerHTML=`
      <div class="context-menu-item" data-action="toggle-visibility">
        <span>切换可见性</span>
        <span class="context-menu-icon">${"public"===t.visibility?'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>':'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>'}</span>
      </div>
      <div class="context-menu-item" data-action="toggle-show">
        <span>${t.show_in_passage?"在文章中隐藏":"在文章中显示"}</span>
        <span class="context-menu-icon">${t.show_in_passage?'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>':'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>'}</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item context-menu-danger" data-action="delete">
        <span>删除附件</span>
        <span class="context-menu-icon"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></span>
      </div>
    `,document.body.appendChild(o),o.querySelectorAll(".context-menu-item").forEach(t=>{t.addEventListener("click",e=>{e.stopPropagation();e=t.dataset.action;this.handleAttachmentAction(i,e),o.remove()})}),setTimeout(()=>{document.addEventListener("click",function e(){o.remove(),document.removeEventListener("click",e)})},0)}},async handleAttachmentAction(e,t){switch(t){case"toggle-visibility":await this.toggleAttachmentVisibility(e);break;case"toggle-show":await this.toggleAttachmentShow(e);break;case"delete":await this.deleteAttachment(e)}},async toggleAttachmentVisibility(t){var e=this.currentAttachments?.find(e=>e.id===parseInt(t));if(e){e="public"===e.visibility?"private":"public";try{var i=await(await fetch("/api/admin/attachments/"+t,{method:"PATCH",headers:{Authorization:this.getAuthHeader(),"Content-Type":"application/json"},body:JSON.stringify({visibility:e})})).json();i.success?(this.showToast("更新成功","success"),this.loadAttachments()):this.showToast(i.message,"error")}catch(e){console.error("更新附件失败:",e),this.showToast("更新附件失败","error")}}},async toggleAttachmentShow(t){var e=this.currentAttachments?.find(e=>e.id===parseInt(t));if(e)try{var i=await(await fetch("/api/admin/attachments?id="+t,{method:"PATCH",headers:{Authorization:this.getAuthHeader(),"Content-Type":"application/json"},body:JSON.stringify({show_in_passage:!e.show_in_passage})})).json();i.success?(this.showToast("更新成功","success"),this.loadAttachments()):this.showToast(i.message,"error")}catch(e){console.error("更新附件失败:",e),this.showToast("更新附件失败","error")}},async deleteAttachment(t){if(this.selectedAttachmentId)try{var e=await(await fetch("/api/admin/attachments/"+t,{method:"DELETE",headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.showToast("删除成功","success"),this.closeModal(document.getElementById("deleteModal")),this.loadAttachments()):this.showToast(e.message,"error")}catch(e){console.error("删除附件失败:",e),this.showToast("删除附件失败","error")}finally{this.selectedAttachmentId=null}else(e=this.currentAttachments?.find(e=>e.id===parseInt(t)))&&(this.selectedAttachmentId=t,document.getElementById("deleteFileName").textContent=e.file_name,document.getElementById("deleteModal").classList.add("active"))},formatFileSize:function(e){var t;return 0===e?"0 B":(t=Math.floor(Math.log(e)/Math.log(1024)),parseFloat((e/Math.pow(1024,t)).toFixed(2))+" "+["B","KB","MB","GB"][t])},getFileIcon(e){return{image:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="3" ry="3"/>
        <circle cx="8.5" cy="8.5" r="1.5"/>
        <path d="M21 15l-5-5L5 21"/>
      </svg>`,video:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="5 3 19 12 5 21 5 3" fill="currentColor"/>
        <rect x="2" y="2" width="20" height="20" rx="3" ry="3"/>
      </svg>`,audio:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 18V5l12-2v13"/>
        <circle cx="6" cy="18" r="3" fill="currentColor"/>
        <circle cx="18" cy="16" r="3" fill="currentColor"/>
      </svg>`,document:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <polyline points="10 9 9 9 8 9"/>
      </svg>`,archive:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
        <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
        <line x1="12" y1="22.08" x2="12" y2="12"/>
      </svg>`,markdown:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <path d="M10 13l-2 2 2 2"/>
        <path d="M14 13l2 2-2 2"/>
      </svg>`,code:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M16 18l6-6-6-6"/>
        <path d="M8 6l-6 6 6 6"/>
      </svg>`,pdf:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <text x="7" y="17" font-size="6" font-weight="bold" fill="currentColor">PDF</text>
      </svg>`,word:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <text x="4" y="17" font-size="5" font-weight="bold" fill="currentColor">DOC</text>
      </svg>`,excel:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <rect x="7" y="10" width="10" height="2"/>
        <rect x="7" y="14" width="10" height="2"/>
        <rect x="7" y="18" width="10" height="2"/>
      </svg>`,ppt:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <rect x="8" y="11" width="8" height="6" rx="1"/>
      </svg>`,text:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <line x1="10" y1="9" x2="8" y2="9"/>
      </svg>`,font:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <text x="6" y="18" font-size="10" font-weight="bold" fill="currentColor">Aa</text>
      </svg>`,database:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <ellipse cx="12" cy="5" rx="9" ry="3"/>
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
      </svg>`,executable:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <circle cx="12" cy="14" r="3"/>
        <line x1="12" y1="11" x2="12" y2="17"/>
        <line x1="9" y1="14" x2="15" y2="14"/>
      </svg>`}[e]||`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
      <polyline points="14 2 14 8 20 8"/>
      <line x1="12" y1="18" x2="12" y2="12"/>
      <line x1="9" y1="15" x2="15" y2="15"/>
    </svg>`},renderFiles(e){var t=document.getElementById("fileGrid"),i=document.getElementById("emptyState");0===e.length?(t.innerHTML="",i.style.display="flex"):(i.style.display="none",i=[...e].sort((e,t)=>e.is_dir&&!t.is_dir?-1:!e.is_dir&&t.is_dir?1:e.name.localeCompare(t.name)),t.innerHTML=i.map(e=>this.createFileItem(e)).join(""),t.querySelectorAll(".file-item").forEach(o=>{o.addEventListener("click",e=>{e.stopPropagation();e=o.dataset.path;"true"===o.dataset.isDir?this.navigateTo(e):e.toLowerCase().endsWith(".md")?this.previewMarkdownFile(e):this.openFile(e)}),o.addEventListener("contextmenu",e=>{e.preventDefault();var t=o.dataset.path,i="true"===o.dataset.isDir;this.showContextMenu(e,t,i)})}))},createFileItem(e){let t,i="";i=e.is_dir?(t=`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>`,"directory"):[".jpg",".jpeg",".png",".gif",".webp",".bmp",".svg",".ico",".tiff",".tif",".avif",".jxl"].includes(e.extension)?(t=`<img src="/${e.path}" alt="${e.name}" onerror="this.parentElement.innerHTML='<svg width=\\'72\\' height=\\'72\\' viewBox=\\'0 0 24 24\\' fill=\\'none\\' stroke=\\'currentColor\\' stroke-width=\\'2\\'><rect x=\\'3\\' y=\\'3\\' width=\\'18\\' height=\\'18\\' rx=\\'3\\' ry=\\'3\\'></rect><circle cx=\\'8.5\\' cy=\\'8.5\\' r=\\'1.5\\'></circle><path d=\\'21 15l-5-5L5 21\\'></path></svg>'">`,"image"):[".mp3",".flac",".wav",".ogg",".m4a",".aac",".wma",".opus",".ape",".wv",".tta"].includes(e.extension)?(t=this.getFileIcon("audio"),"audio"):[".mp4",".webm",".mkv",".avi",".mov",".wmv",".flv",".m4v",".3gp",".ts",".m2ts"].includes(e.extension)?(t=this.getFileIcon("video"),"video"):".md"===e.extension?(t=this.getFileIcon("markdown"),"markdown"):[".pdf"].includes(e.extension)?(t=this.getFileIcon("pdf"),"document pdf"):[".doc",".docx",".odt",".rtf"].includes(e.extension)?(t=this.getFileIcon("word"),"document word"):[".xls",".xlsx",".ods",".csv"].includes(e.extension)?(t=this.getFileIcon("excel"),"document excel"):[".ppt",".pptx",".odp"].includes(e.extension)?(t=this.getFileIcon("ppt"),"document ppt"):[".txt",".log",".md"].includes(e.extension)?(t=this.getFileIcon("text"),"document text"):[".zip",".rar",".7z",".tar",".gz",".bz2",".xz",".tar.gz",".tar.bz2",".tar.xz"].includes(e.extension)?(t=this.getFileIcon("archive"),"archive"):[".html",".htm",".css",".js",".ts",".jsx",".tsx",".vue",".svelte",".json",".xml",".yaml",".yml",".toml",".ini",".cfg",".conf"].includes(e.extension)?(t=this.getFileIcon("code"),"code"):[".ttf",".otf",".woff",".woff2",".eot"].includes(e.extension)?(t=this.getFileIcon("font"),"font"):[".db",".sqlite",".sqlite3",".mdb",".sql"].includes(e.extension)?(t=this.getFileIcon("database"),"database"):[".exe",".app",".dmg",".msi",".deb",".rpm",".sh",".bat",".cmd",".ps1"].includes(e.extension)?(t=this.getFileIcon("executable"),"executable"):(t=this.getFileIcon("default"),"file");var o=this.formatFileSize(e.size);return`
      <div class="file-item ${i}" data-path="${e.path}" data-is-dir="${e.is_dir}">
        <div class="file-icon">${t}</div>
        <div class="file-name">${e.name}</div>
        <div class="file-meta">${e.is_dir?"文件夹":o}</div>
      </div>
    `},updateBreadcrumb(e){document.getElementById("currentPath").textContent=e||"/"},updateBackButton(e){document.getElementById("backBtn").disabled=!e},updateFileCount(e){document.getElementById("fileCount").textContent=e+" 个项目"},switchRoot(t){this.currentRoot=t,this.currentPath=t,document.querySelectorAll(".fm-root-btn").forEach(e=>{e.classList.toggle("fm-root-btn-active",e.dataset.path===t)}),this.loadFiles()},navigateTo(e){this.currentPath=e,this.loadFiles()},goBack(){var e=this.getParentPath(this.currentPath);e&&this.navigateTo(e)},getParentPath(e){return e===this.currentRoot?null:((e=e.split("/")).pop(),e.join("/")||this.currentRoot)},async openFile(e){var t=e.split(".").pop().toLowerCase(),i=e.split("/").pop();if("md"===t)this.previewMarkdownFile(e);else if([".jpg",".jpeg",".png",".gif",".webp",".bmp",".svg",".ico",".tiff",".tif"].includes("."+t))this.openImagePreview(e,i);else if([".mp3",".flac",".wav",".ogg",".m4a",".aac",".wma"].includes("."+t))this.openAudioPreview(e,i);else if([".mp4",".webm",".mkv",".avi",".mov",".wmv",".flv"].includes("."+t))this.openVideoPreview(e,i);else if([".pdf",".doc",".docx",".xls",".xlsx",".ppt",".pptx",".txt"].includes("."+t))this.openDocumentPreview(e,i,t);else try{var o,s,a,n,l=await fetch("/api/files/download?path="+encodeURIComponent(e),{headers:{Authorization:this.getAuthHeader()}});l.ok?(s=await l.blob(),a=window.URL.createObjectURL(s),(n=document.createElement("a")).href=a,n.download=i,document.body.appendChild(n),n.click(),document.body.removeChild(n),window.URL.revokeObjectURL(a)):(o=await l.json(),this.showToast(o.message||"下载失败","error"))}catch(e){console.error("下载失败:",e),this.showToast("下载失败","error")}},openImagePreview(e,t){e="/"+e;const i=document.createElement("div");i.className="fm-modal preview-modal",i.innerHTML=`
      <div class="fm-modal-content preview-content">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <img src="${e}" alt="${t}" class="preview-image">
        </div>
      </div>
    `,document.body.appendChild(i);e=i.querySelector(".fm-modal-close");const o=()=>{document.body.removeChild(i)},s=(e.addEventListener("click",o),i.addEventListener("click",e=>{e.target===i&&o()}),e=>{"Escape"===e.key&&(o(),document.removeEventListener("keydown",s))});document.addEventListener("keydown",s),i.classList.add("active")},openAudioPreview(e,t){var i="/"+e;const o=document.createElement("div");o.className="fm-modal preview-modal",o.innerHTML=`
      <div class="fm-modal-content preview-content audio-preview">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <div class="audio-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg></div>
          <audio controls autoplay class="preview-audio">
            <source src="${i}" type="audio/${e.split(".").pop()}">
            您的浏览器不支持音频播放
          </audio>
        </div>
      </div>
    `,document.body.appendChild(o);t=o.querySelector(".fm-modal-close");const s=()=>{var e=o.querySelector("audio");e&&e.pause(),document.body.removeChild(o)},a=(t.addEventListener("click",s),o.addEventListener("click",e=>{e.target===o&&s()}),e=>{"Escape"===e.key&&(s(),document.removeEventListener("keydown",a))});document.addEventListener("keydown",a),o.classList.add("active")},openVideoPreview(e,t){var i="/"+e;const o=document.createElement("div");o.className="fm-modal preview-modal",o.innerHTML=`
      <div class="fm-modal-content preview-content video-preview">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <video controls autoplay class="preview-video">
            <source src="${i}" type="video/${e.split(".").pop()}">
            您的浏览器不支持视频播放
          </video>
        </div>
      </div>
    `,document.body.appendChild(o);t=o.querySelector(".fm-modal-close");const s=()=>{var e=o.querySelector("video");e&&(e.pause(),e.currentTime=0),document.body.removeChild(o)},a=(t.addEventListener("click",s),o.addEventListener("click",e=>{e.target===o&&s()}),e=>{"Escape"===e.key&&(s(),document.removeEventListener("keydown",a))});document.addEventListener("keydown",a),requestAnimationFrame(()=>{o.classList.add("active")})},async openDocumentPreview(e,t,i){var o="/"+e;const s=document.createElement("div");s.className="fm-modal preview-modal";let a="",n="document-preview";switch(i){case"pdf":a=`
          <embed src="${o}" type="application/pdf" class="preview-embed" />
        `,n="pdf-preview";break;case"txt":a=`
          <iframe src="${o}" class="preview-iframe"></iframe>
        `,n="txt-preview";break;case"doc":case"docx":case"xls":case"xlsx":case"ppt":case"pptx":a=`
          <iframe src="https://docs.google.com/viewer?url=${encodeURIComponent(window.location.origin+"/"+e)}&embedded=true" class="preview-iframe"></iframe>
        `,n="office-preview";break;default:a=`
          <div class="preview-placeholder">
            <div class="placeholder-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg></div>
            <p>此文件类型暂不支持在线预览</p>
            <button class="fm-btn fm-btn-primary" onclick="FileManager.downloadFile('${e}')">下载文件</button>
          </div>
        `}s.innerHTML=`
      <div class="fm-modal-content preview-content ${n}">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          ${a}
        </div>
        <div class="fm-modal-footer">
          <button class="fm-btn fm-btn-secondary fm-modal-close-btn">关闭</button>
          <button class="fm-btn fm-btn-primary" onclick="FileManager.downloadFile('${e}')">下载文件</button>
        </div>
      </div>
    `,document.body.appendChild(s);i=s.querySelector(".fm-modal-close");const l=()=>{document.body.removeChild(s)},r=(i.addEventListener("click",l),s.addEventListener("click",e=>{e.target===s&&l()}),s.querySelectorAll(".fm-modal-close-btn").forEach(e=>{e.addEventListener("click",l)}),e=>{"Escape"===e.key&&(l(),document.removeEventListener("keydown",r))});document.addEventListener("keydown",r),s.classList.add("active")},async previewMarkdownFile(e){var t;window.MarkdownPreviewModal?this.openMarkdownPreview(e):((t=document.createElement("script")).src="/js/markdown-preview-modal.js",t.onload=()=>{this.openMarkdownPreview(e)},t.onerror=()=>{this.showToast("预览组件加载失败","error")},document.head.appendChild(t))},openMarkdownPreview(e){let t=e;if(t.startsWith("/")){var i=t.indexOf("/markdown/");if(-1===i)return console.error("无效的 Markdown 路径:",e),void this.showToast("无效的 Markdown 路径","error");t=t.substring(i+10)}else{if(!t.startsWith("markdown/"))return console.error("无效的 Markdown 路径:",e),void this.showToast("无效的 Markdown 路径","error");t=t.substring(9)}t&&"/"!==t&&""!==t.trim()?(console.log("Markdown 预览路径:",t),window.MarkdownPreviewModal?window.MarkdownPreviewModal.open(t):this.showToast("预览功能不可用","error")):(console.error("提取后的 Markdown 路径无效:",t),this.showToast("无效的 Markdown 路径","error"))},async downloadFile(e){try{var t,i,o,s,a,n=await fetch("/api/files/download?path="+encodeURIComponent(e),{headers:{Authorization:this.getAuthHeader()}});n.ok?(i=e.split("/").pop(),o=await n.blob(),s=window.URL.createObjectURL(o),(a=document.createElement("a")).href=s,a.download=i,document.body.appendChild(a),a.click(),document.body.removeChild(a),window.URL.revokeObjectURL(s)):(t=await n.json(),this.showToast(t.message||"下载失败","error"))}catch(e){console.error("下载失败:",e),this.showToast("下载失败","error")}},showContextMenu(e,t,i){this.selectedFile={path:t,isDir:i};var o=document.getElementById("contextMenu");o.style.left=e.pageX+"px",o.style.top=e.pageY+"px",o.classList.add("active");const s=t.toLowerCase().endsWith(".md");e=o.querySelectorAll(".context-menu-item");e.forEach(e=>{var t=e.dataset.action;e.style.display=("download"!==t||!i)&&("preview"!==t||s&&!i)?"flex":"none"}),e.forEach(e=>{e.onclick=()=>{this.handleContextAction(e.dataset.action),this.hideContextMenu()}})},hideContextMenu(){document.getElementById("contextMenu").classList.remove("active")},handleContextAction(e){if(this.selectedFile)switch(e){case"open":this.selectedFile.isDir?this.navigateTo(this.selectedFile.path):this.openFile(this.selectedFile.path);break;case"preview":this.previewMarkdownFile(this.selectedFile.path);break;case"download":this.downloadFile(this.selectedFile.path);break;case"rename":this.openRenameModal();break;case"delete":this.openDeleteModal()}},openUploadModal(){this.filesToUpload=[],this.updateUploadList(),document.getElementById("uploadModal").classList.add("active")},handleFileSelect(e){var t=Array.from(e.target.files);this.addFilesToUpload(t),e.target.value=""},handleFileDrop(e){e=Array.from(e.dataTransfer.files);this.addFilesToUpload(e)},addFilesToUpload(e){e.forEach(t=>{this.filesToUpload.find(e=>e.name===t.name)||this.filesToUpload.push(t)}),this.updateUploadList()},updateUploadList(){var e=document.getElementById("uploadList"),t=document.getElementById("confirmUploadBtn");0===this.filesToUpload.length?(e.innerHTML="",t.disabled=!0):(t.disabled=!1,e.innerHTML=this.filesToUpload.map((e,t)=>`
      <div class="upload-item">
        <div class="upload-item-name">${e.name}</div>
        <div class="upload-item-size">${this.formatFileSize(e.size)}</div>
        <button class="upload-item-remove" onclick="FileManager.removeFileFromUpload(${t})">✕</button>
      </div>
    `).join(""))},removeFileFromUpload(e){this.filesToUpload.splice(e,1),this.updateUploadList()},async uploadFiles(){if(0!==this.filesToUpload.length){var i=document.getElementById("confirmUploadBtn");i.disabled=!0,i.textContent="上传中...";let e=0,t=0;for(const a of this.filesToUpload)try{var o=new FormData;o.append("file",a);var s=await(await fetch("/api/files?path="+encodeURIComponent(this.currentPath),{method:"POST",headers:{Authorization:this.getAuthHeader()},body:o})).json();s.success?e++:(t++,console.error("上传失败:",s.message))}catch(e){console.error("上传失败:",e),t++}this.closeModal(document.getElementById("uploadModal")),this.loadFiles(),0<e&&0===t?this.showToast(`成功上传 ${e} 个文件`,"success"):0<e?this.showToast(`成功上传 ${e} 个文件，失败 ${t} 个`,"warning"):this.showToast("上传失败","error"),i.disabled=!1,i.textContent="上传"}},openCreateDirModal(){document.getElementById("dirNameInput").value="",document.getElementById("createDirModal").classList.add("active"),setTimeout(()=>{document.getElementById("dirNameInput").focus()},100)},async createDirectory(){var e=document.getElementById("dirNameInput").value.trim();if(e)try{var t=await(await fetch("/api/files/create-dir",{method:"POST",headers:{"Content-Type":"application/json",Authorization:this.getAuthHeader()},body:JSON.stringify({path:this.currentPath,dir_name:e})})).json();t.success?(this.showToast("文件夹创建成功","success"),this.closeModal(document.getElementById("createDirModal")),this.loadFiles()):this.showToast(t.message,"error")}catch(e){console.error("创建目录失败:",e),this.showToast("创建文件夹失败","error")}else this.showToast("请输入文件夹名称","warning")},openRenameModal(){var e;this.selectedFile&&(e=this.selectedFile.path.split("/").pop(),document.getElementById("renameInput").value=e,document.getElementById("renameModal").classList.add("active"),setTimeout(()=>{document.getElementById("renameInput").focus(),document.getElementById("renameInput").select()},100))},async renameFile(){if(this.selectedFile){var e=document.getElementById("renameInput").value.trim();if(e)try{var t=await(await fetch("/api/files",{method:"PUT",headers:{"Content-Type":"application/json",Authorization:this.getAuthHeader()},body:JSON.stringify({old_path:this.selectedFile.path,new_name:e})})).json();t.success?(this.showToast("重命名成功","success"),this.closeModal(document.getElementById("renameModal")),this.loadFiles()):this.showToast(t.message,"error")}catch(e){console.error("重命名失败:",e),this.showToast("重命名失败","error")}else this.showToast("请输入新名称","warning")}},openDeleteModal(){var e;this.selectedFile&&(e=this.selectedFile.path.split("/").pop(),document.getElementById("deleteFileName").textContent=e,document.getElementById("deleteModal").classList.add("active"))},async deleteFile(){if(this.selectedFile)try{var e=await(await fetch("/api/files?path="+encodeURIComponent(this.selectedFile.path),{method:"DELETE",headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.showToast("删除成功","success"),this.closeModal(document.getElementById("deleteModal")),this.loadFiles()):this.showToast(e.message,"error")}catch(e){console.error("删除失败:",e),this.showToast("删除失败","error")}},closeModal(e){e.classList.remove("active"),e.classList.add("closing"),setTimeout(()=>{e.classList.remove("closing")},300)},showToast(e,t="success"){const i=document.getElementById("toast");i.textContent=e,i.className=`toast ${t} active`,setTimeout(()=>{i.classList.remove("active")},3e3)}};document.addEventListener("DOMContentLoaded",()=>{FileManager.init()});
