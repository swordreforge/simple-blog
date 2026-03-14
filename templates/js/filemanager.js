/* ESBuild compressed */
var l=(e,t,i)=>new Promise((o,s)=>{var a=d=>{try{r(i.next(d))}catch(c){s(c)}},n=d=>{try{r(i.throw(d))}catch(c){s(c)}},r=d=>d.done?o(d.value):Promise.resolve(d.value).then(a,n);r((i=i.apply(e,t)).next())});const h={currentPath:"img",currentRoot:"img",selectedFile:null,selectedAttachmentId:null,filesToUpload:[],getAuthHeader(){return"Bearer "+this.getCookie("auth_token")},getCookie(e){return e=("; "+document.cookie).split(`; ${e}=`),e.length===2?e.pop().split(";").shift():""},init(){this.bindEvents(),this.loadFiles()},bindEvents(){document.getElementById("backBtn").addEventListener("click",()=>this.goBack()),document.getElementById("uploadBtn").addEventListener("click",()=>this.openUploadModal()),document.getElementById("createDirBtn").addEventListener("click",()=>this.openCreateDirModal()),document.querySelectorAll(".fm-root-btn").forEach(i=>{i.addEventListener("click",o=>{o=o.currentTarget.dataset.path,this.switchRoot(o)})}),document.querySelectorAll(".modal-close, .fm-modal-close-btn, .fm-modal-close").forEach(i=>{i.addEventListener("click",o=>{o=o.target.closest(".modal")||o.target.closest(".fm-modal"),o&&this.closeModal(o)})});const e=document.getElementById("uploadArea"),t=document.getElementById("fileInput");e&&t?(e.addEventListener("click",i=>{i.stopPropagation(),i.preventDefault(),t.click()}),t.addEventListener("change",i=>{i.stopPropagation(),this.handleFileSelect(i)}),e.addEventListener("dragover",i=>{i.preventDefault(),i.stopPropagation(),e.classList.add("dragover")}),e.addEventListener("dragleave",i=>{i.preventDefault(),i.stopPropagation(),e.classList.remove("dragover")}),e.addEventListener("drop",i=>{i.preventDefault(),i.stopPropagation(),e.classList.remove("dragover"),this.handleFileDrop(i)})):console.error("\u4E0A\u4F20\u533A\u57DF\u6216\u6587\u4EF6\u8F93\u5165\u6846\u672A\u627E\u5230"),document.getElementById("confirmUploadBtn").addEventListener("click",()=>this.uploadFiles()),document.getElementById("confirmCreateDirBtn").addEventListener("click",()=>this.createDirectory()),document.getElementById("confirmRenameBtn").addEventListener("click",()=>this.renameFile()),document.getElementById("confirmDeleteBtn").addEventListener("click",()=>{this.selectedAttachmentId?this.deleteAttachment(this.selectedAttachmentId):this.deleteFile()}),document.addEventListener("click",i=>{i.target.closest(".context-menu")||i.target.closest(".file-item")||this.hideContextMenu()}),document.addEventListener("keydown",i=>{i.key==="Escape"&&(this.hideContextMenu(),document.querySelectorAll(".modal.active, .fm-modal.active").forEach(o=>{this.closeModal(o)}))})},loadFiles(){return l(this,null,function*(){if(this.currentRoot==="attachments")yield this.loadAttachments();else try{var e=yield(yield fetch("/api/files?path="+encodeURIComponent(this.currentPath),{headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.renderFiles(e.data.files),this.updateBreadcrumb(e.data.current_path),this.updateBackButton(e.data.parent_path),this.updateFileCount(e.data.files.length)):this.showToast(e.message,"error")}catch(t){console.error("\u52A0\u8F7D\u6587\u4EF6\u5931\u8D25:",t),this.showToast("\u52A0\u8F7D\u6587\u4EF6\u5931\u8D25","error")}})},loadAttachments(){return l(this,null,function*(){try{var e=yield(yield fetch("/api/admin/attachments",{headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.currentAttachments=e.data,this.renderAttachments(e.data),this.updateBreadcrumb("/attachments"),this.updateBackButton(null),this.updateFileCount(e.total)):this.showToast(e.message,"error")}catch(t){console.error("\u52A0\u8F7D\u9644\u4EF6\u5931\u8D25:",t),this.showToast("\u52A0\u8F7D\u9644\u4EF6\u5931\u8D25","error")}})},renderAttachments(e){var t=document.getElementById("fileGrid"),i=document.getElementById("emptyState");e.length===0?(t.innerHTML="",i.style.display="flex"):(i.style.display="none",t.innerHTML=e.map(o=>this.createAttachmentItem(o)).join(""),t.querySelectorAll(".file-item").forEach(o=>{o.addEventListener("click",s=>{s.stopPropagation();var a=o.dataset.id;this.showAttachmentMenu(a,s)})}))},createAttachmentItem(e){var t={public:'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>',private:'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>',protected:'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>'}[e.visibility]||'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>',i={public:"\u516C\u5F00",private:"\u79C1\u5BC6",protected:"\u53D7\u4FDD\u62A4"}[e.visibility]||"\u516C\u5F00",o=e.show_in_passage?'<span class="badge badge-success">\u663E\u793A</span>':'<span class="badge badge-secondary">\u9690\u85CF</span>';return`
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
    `},showAttachmentMenu(e,t){var o;var i=(o=this.currentAttachments)==null?void 0:o.find(s=>s.id===parseInt(e));if(i){const s=document.createElement("div");s.className="context-menu",s.style.position="absolute",s.style.left=t.clientX+"px",s.style.top=t.clientY+"px",s.innerHTML=`
      <div class="context-menu-item" data-action="toggle-visibility">
        <span>\u5207\u6362\u53EF\u89C1\u6027</span>
        <span class="context-menu-icon">${i.visibility==="public"?'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>':'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>'}</span>
      </div>
      <div class="context-menu-item" data-action="toggle-show">
        <span>${i.show_in_passage?"\u5728\u6587\u7AE0\u4E2D\u9690\u85CF":"\u5728\u6587\u7AE0\u4E2D\u663E\u793A"}</span>
        <span class="context-menu-icon">${i.show_in_passage?'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>':'<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>'}</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item context-menu-danger" data-action="delete">
        <span>\u5220\u9664\u9644\u4EF6</span>
        <span class="context-menu-icon"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></span>
      </div>
    `,document.body.appendChild(s),s.querySelectorAll(".context-menu-item").forEach(a=>{a.addEventListener("click",n=>{n.stopPropagation(),n=a.dataset.action,this.handleAttachmentAction(e,n),s.remove()})}),setTimeout(()=>{document.addEventListener("click",function a(){s.remove(),document.removeEventListener("click",a)})},0)}},handleAttachmentAction(e,t){return l(this,null,function*(){switch(t){case"toggle-visibility":yield this.toggleAttachmentVisibility(e);break;case"toggle-show":yield this.toggleAttachmentShow(e);break;case"delete":yield this.deleteAttachment(e)}})},toggleAttachmentVisibility(e){return l(this,null,function*(){var o;var t=(o=this.currentAttachments)==null?void 0:o.find(s=>s.id===parseInt(e));if(t){t=t.visibility==="public"?"private":"public";try{var i=yield(yield fetch("/api/admin/attachments/"+e,{method:"PATCH",headers:{Authorization:this.getAuthHeader(),"Content-Type":"application/json"},body:JSON.stringify({visibility:t})})).json();i.success?(this.showToast("\u66F4\u65B0\u6210\u529F","success"),this.loadAttachments()):this.showToast(i.message,"error")}catch(s){console.error("\u66F4\u65B0\u9644\u4EF6\u5931\u8D25:",s),this.showToast("\u66F4\u65B0\u9644\u4EF6\u5931\u8D25","error")}}})},toggleAttachmentShow(e){return l(this,null,function*(){var o;var t=(o=this.currentAttachments)==null?void 0:o.find(s=>s.id===parseInt(e));if(t)try{var i=yield(yield fetch("/api/admin/attachments?id="+e,{method:"PATCH",headers:{Authorization:this.getAuthHeader(),"Content-Type":"application/json"},body:JSON.stringify({show_in_passage:!t.show_in_passage})})).json();i.success?(this.showToast("\u66F4\u65B0\u6210\u529F","success"),this.loadAttachments()):this.showToast(i.message,"error")}catch(s){console.error("\u66F4\u65B0\u9644\u4EF6\u5931\u8D25:",s),this.showToast("\u66F4\u65B0\u9644\u4EF6\u5931\u8D25","error")}})},deleteAttachment(e){return l(this,null,function*(){var i;if(this.selectedAttachmentId)try{var t=yield(yield fetch("/api/admin/attachments/"+e,{method:"DELETE",headers:{Authorization:this.getAuthHeader()}})).json();t.success?(this.showToast("\u5220\u9664\u6210\u529F","success"),this.closeModal(document.getElementById("deleteModal")),this.loadAttachments()):this.showToast(t.message,"error")}catch(o){console.error("\u5220\u9664\u9644\u4EF6\u5931\u8D25:",o),this.showToast("\u5220\u9664\u9644\u4EF6\u5931\u8D25","error")}finally{this.selectedAttachmentId=null}else(t=(i=this.currentAttachments)==null?void 0:i.find(o=>o.id===parseInt(e)))&&(this.selectedAttachmentId=e,document.getElementById("deleteFileName").textContent=t.file_name,document.getElementById("deleteModal").classList.add("active"))})},formatFileSize:function(e){var t;return e===0?"0 B":(t=Math.floor(Math.log(e)/Math.log(1024)),parseFloat((e/Math.pow(1024,t)).toFixed(2))+" "+["B","KB","MB","GB"][t])},getFileIcon(e){return{image:`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
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
    </svg>`},renderFiles(e){var t=document.getElementById("fileGrid"),i=document.getElementById("emptyState");e.length===0?(t.innerHTML="",i.style.display="flex"):(i.style.display="none",i=[...e].sort((o,s)=>o.is_dir&&!s.is_dir?-1:!o.is_dir&&s.is_dir?1:o.name.localeCompare(s.name)),t.innerHTML=i.map(o=>this.createFileItem(o)).join(""),t.querySelectorAll(".file-item").forEach(o=>{o.addEventListener("click",s=>{s.stopPropagation(),s=o.dataset.path,o.dataset.isDir==="true"?this.navigateTo(s):s.toLowerCase().endsWith(".md")?this.previewMarkdownFile(s):this.openFile(s)}),o.addEventListener("contextmenu",s=>{s.preventDefault();var a=o.dataset.path,n=o.dataset.isDir==="true";this.showContextMenu(s,a,n)})}))},createFileItem(e){let t,i="";i=e.is_dir?(t=`<svg width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>`,"directory"):[".jpg",".jpeg",".png",".gif",".webp",".bmp",".svg",".ico",".tiff",".tif",".avif",".jxl"].includes(e.extension)?(t=`<img src="/${e.path}" alt="${e.name}" onerror="this.parentElement.innerHTML='<svg width=\\'72\\' height=\\'72\\' viewBox=\\'0 0 24 24\\' fill=\\'none\\' stroke=\\'currentColor\\' stroke-width=\\'2\\'><rect x=\\'3\\' y=\\'3\\' width=\\'18\\' height=\\'18\\' rx=\\'3\\' ry=\\'3\\'></rect><circle cx=\\'8.5\\' cy=\\'8.5\\' r=\\'1.5\\'></circle><path d=\\'21 15l-5-5L5 21\\'></path></svg>'">`,"image"):[".mp3",".flac",".wav",".ogg",".m4a",".aac",".wma",".opus",".ape",".wv",".tta"].includes(e.extension)?(t=this.getFileIcon("audio"),"audio"):[".mp4",".webm",".mkv",".avi",".mov",".wmv",".flv",".m4v",".3gp",".ts",".m2ts"].includes(e.extension)?(t=this.getFileIcon("video"),"video"):e.extension===".md"?(t=this.getFileIcon("markdown"),"markdown"):[".pdf"].includes(e.extension)?(t=this.getFileIcon("pdf"),"document pdf"):[".doc",".docx",".odt",".rtf"].includes(e.extension)?(t=this.getFileIcon("word"),"document word"):[".xls",".xlsx",".ods",".csv"].includes(e.extension)?(t=this.getFileIcon("excel"),"document excel"):[".ppt",".pptx",".odp"].includes(e.extension)?(t=this.getFileIcon("ppt"),"document ppt"):[".txt",".log",".md"].includes(e.extension)?(t=this.getFileIcon("text"),"document text"):[".zip",".rar",".7z",".tar",".gz",".bz2",".xz",".tar.gz",".tar.bz2",".tar.xz"].includes(e.extension)?(t=this.getFileIcon("archive"),"archive"):[".html",".htm",".css",".js",".ts",".jsx",".tsx",".vue",".svelte",".json",".xml",".yaml",".yml",".toml",".ini",".cfg",".conf"].includes(e.extension)?(t=this.getFileIcon("code"),"code"):[".ttf",".otf",".woff",".woff2",".eot"].includes(e.extension)?(t=this.getFileIcon("font"),"font"):[".db",".sqlite",".sqlite3",".mdb",".sql"].includes(e.extension)?(t=this.getFileIcon("database"),"database"):[".exe",".app",".dmg",".msi",".deb",".rpm",".sh",".bat",".cmd",".ps1"].includes(e.extension)?(t=this.getFileIcon("executable"),"executable"):(t=this.getFileIcon("default"),"file");var o=this.formatFileSize(e.size);return`
      <div class="file-item ${i}" data-path="${e.path}" data-is-dir="${e.is_dir}">
        <div class="file-icon">${t}</div>
        <div class="file-name">${e.name}</div>
        <div class="file-meta">${e.is_dir?"\u6587\u4EF6\u5939":o}</div>
      </div>
    `},updateBreadcrumb(e){document.getElementById("currentPath").textContent=e||"/"},updateBackButton(e){document.getElementById("backBtn").disabled=!e},updateFileCount(e){document.getElementById("fileCount").textContent=e+" \u4E2A\u9879\u76EE"},switchRoot(e){this.currentRoot=e,this.currentPath=e,document.querySelectorAll(".fm-root-btn").forEach(t=>{t.classList.toggle("fm-root-btn-active",t.dataset.path===e)}),this.loadFiles()},navigateTo(e){this.currentPath=e,this.loadFiles()},goBack(){var e=this.getParentPath(this.currentPath);e&&this.navigateTo(e)},getParentPath(e){return e===this.currentRoot?null:((e=e.split("/")).pop(),e.join("/")||this.currentRoot)},openFile(e){return l(this,null,function*(){var t=e.split(".").pop().toLowerCase(),i=e.split("/").pop();if(t==="md")this.previewMarkdownFile(e);else if([".jpg",".jpeg",".png",".gif",".webp",".bmp",".svg",".ico",".tiff",".tif"].includes("."+t))this.openImagePreview(e,i);else if([".mp3",".flac",".wav",".ogg",".m4a",".aac",".wma"].includes("."+t))this.openAudioPreview(e,i);else if([".mp4",".webm",".mkv",".avi",".mov",".wmv",".flv"].includes("."+t))this.openVideoPreview(e,i);else if([".pdf",".doc",".docx",".xls",".xlsx",".ppt",".pptx",".txt"].includes("."+t))this.openDocumentPreview(e,i,t);else try{var o,s,a,n,r=yield fetch("/api/files/download?path="+encodeURIComponent(e),{headers:{Authorization:this.getAuthHeader()}});r.ok?(s=yield r.blob(),a=window.URL.createObjectURL(s),(n=document.createElement("a")).href=a,n.download=i,document.body.appendChild(n),n.click(),document.body.removeChild(n),window.URL.revokeObjectURL(a)):(o=yield r.json(),this.showToast(o.message||"\u4E0B\u8F7D\u5931\u8D25","error"))}catch(d){console.error("\u4E0B\u8F7D\u5931\u8D25:",d),this.showToast("\u4E0B\u8F7D\u5931\u8D25","error")}})},openImagePreview(e,t){e="/"+e;const i=document.createElement("div");i.className="fm-modal preview-modal",i.innerHTML=`
      <div class="fm-modal-content preview-content">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <img src="${e}" alt="${t}" class="preview-image">
        </div>
      </div>
    `,document.body.appendChild(i),e=i.querySelector(".fm-modal-close");const o=()=>{document.body.removeChild(i)},s=(e.addEventListener("click",o),i.addEventListener("click",a=>{a.target===i&&o()}),a=>{a.key==="Escape"&&(o(),document.removeEventListener("keydown",s))});document.addEventListener("keydown",s),i.classList.add("active")},openAudioPreview(e,t){var i="/"+e;const o=document.createElement("div");o.className="fm-modal preview-modal",o.innerHTML=`
      <div class="fm-modal-content preview-content audio-preview">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <div class="audio-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg></div>
          <audio controls autoplay class="preview-audio">
            <source src="${i}" type="audio/${e.split(".").pop()}">
            \u60A8\u7684\u6D4F\u89C8\u5668\u4E0D\u652F\u6301\u97F3\u9891\u64AD\u653E
          </audio>
        </div>
      </div>
    `,document.body.appendChild(o),t=o.querySelector(".fm-modal-close");const s=()=>{var n=o.querySelector("audio");n&&n.pause(),document.body.removeChild(o)},a=(t.addEventListener("click",s),o.addEventListener("click",n=>{n.target===o&&s()}),n=>{n.key==="Escape"&&(s(),document.removeEventListener("keydown",a))});document.addEventListener("keydown",a),o.classList.add("active")},openVideoPreview(e,t){var i="/"+e;const o=document.createElement("div");o.className="fm-modal preview-modal",o.innerHTML=`
      <div class="fm-modal-content preview-content video-preview">
        <div class="fm-modal-header">
          <h3>${t}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <video controls autoplay class="preview-video">
            <source src="${i}" type="video/${e.split(".").pop()}">
            \u60A8\u7684\u6D4F\u89C8\u5668\u4E0D\u652F\u6301\u89C6\u9891\u64AD\u653E
          </video>
        </div>
      </div>
    `,document.body.appendChild(o),t=o.querySelector(".fm-modal-close");const s=()=>{var n=o.querySelector("video");n&&(n.pause(),n.currentTime=0),document.body.removeChild(o)},a=(t.addEventListener("click",s),o.addEventListener("click",n=>{n.target===o&&s()}),n=>{n.key==="Escape"&&(s(),document.removeEventListener("keydown",a))});document.addEventListener("keydown",a),requestAnimationFrame(()=>{o.classList.add("active")})},openDocumentPreview(e,t,i){return l(this,null,function*(){var o="/"+e;const s=document.createElement("div");s.className="fm-modal preview-modal";let a="",n="document-preview";switch(i){case"pdf":a=`
          <embed src="${o}" type="application/pdf" class="preview-embed" />
        `,n="pdf-preview";break;case"txt":a=`
          <iframe src="${o}" class="preview-iframe"></iframe>
        `,n="txt-preview";break;case"doc":case"docx":case"xls":case"xlsx":case"ppt":case"pptx":a=`
          <iframe src="https://docs.google.com/viewer?url=${encodeURIComponent(window.location.origin+"/"+e)}&embedded=true" class="preview-iframe"></iframe>
        `,n="office-preview";break;default:a=`
          <div class="preview-placeholder">
            <div class="placeholder-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg></div>
            <p>\u6B64\u6587\u4EF6\u7C7B\u578B\u6682\u4E0D\u652F\u6301\u5728\u7EBF\u9884\u89C8</p>
            <button class="fm-btn fm-btn-primary" onclick="FileManager.downloadFile('${e}')">\u4E0B\u8F7D\u6587\u4EF6</button>
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
          <button class="fm-btn fm-btn-secondary fm-modal-close-btn">\u5173\u95ED</button>
          <button class="fm-btn fm-btn-primary" onclick="FileManager.downloadFile('${e}')">\u4E0B\u8F7D\u6587\u4EF6</button>
        </div>
      </div>
    `,document.body.appendChild(s),i=s.querySelector(".fm-modal-close");const r=()=>{document.body.removeChild(s)},d=(i.addEventListener("click",r),s.addEventListener("click",c=>{c.target===s&&r()}),s.querySelectorAll(".fm-modal-close-btn").forEach(c=>{c.addEventListener("click",r)}),c=>{c.key==="Escape"&&(r(),document.removeEventListener("keydown",d))});document.addEventListener("keydown",d),s.classList.add("active")})},previewMarkdownFile(e){return l(this,null,function*(){var t;window.MarkdownPreviewModal?this.openMarkdownPreview(e):((t=document.createElement("script")).src="/js/markdown-preview-modal.js",t.onload=()=>{this.openMarkdownPreview(e)},t.onerror=()=>{this.showToast("\u9884\u89C8\u7EC4\u4EF6\u52A0\u8F7D\u5931\u8D25","error")},document.head.appendChild(t))})},openMarkdownPreview(e){let t=e;if(t.startsWith("/")){var i=t.indexOf("/markdown/");if(i===-1)return console.error("\u65E0\u6548\u7684 Markdown \u8DEF\u5F84:",e),void this.showToast("\u65E0\u6548\u7684 Markdown \u8DEF\u5F84","error");t=t.substring(i+10)}else{if(!t.startsWith("markdown/"))return console.error("\u65E0\u6548\u7684 Markdown \u8DEF\u5F84:",e),void this.showToast("\u65E0\u6548\u7684 Markdown \u8DEF\u5F84","error");t=t.substring(9)}t&&t!=="/"&&t.trim()!==""?(console.log("Markdown \u9884\u89C8\u8DEF\u5F84:",t),window.MarkdownPreviewModal?window.MarkdownPreviewModal.open(t):this.showToast("\u9884\u89C8\u529F\u80FD\u4E0D\u53EF\u7528","error")):(console.error("\u63D0\u53D6\u540E\u7684 Markdown \u8DEF\u5F84\u65E0\u6548:",t),this.showToast("\u65E0\u6548\u7684 Markdown \u8DEF\u5F84","error"))},downloadFile(e){return l(this,null,function*(){try{var t,i,o,s,a,n=yield fetch("/api/files/download?path="+encodeURIComponent(e),{headers:{Authorization:this.getAuthHeader()}});n.ok?(i=e.split("/").pop(),o=yield n.blob(),s=window.URL.createObjectURL(o),(a=document.createElement("a")).href=s,a.download=i,document.body.appendChild(a),a.click(),document.body.removeChild(a),window.URL.revokeObjectURL(s)):(t=yield n.json(),this.showToast(t.message||"\u4E0B\u8F7D\u5931\u8D25","error"))}catch(r){console.error("\u4E0B\u8F7D\u5931\u8D25:",r),this.showToast("\u4E0B\u8F7D\u5931\u8D25","error")}})},showContextMenu(e,t,i){this.selectedFile={path:t,isDir:i};var o=document.getElementById("contextMenu");o.style.left=e.pageX+"px",o.style.top=e.pageY+"px",o.classList.add("active");const s=t.toLowerCase().endsWith(".md");e=o.querySelectorAll(".context-menu-item"),e.forEach(a=>{var n=a.dataset.action;a.style.display=(n!=="download"||!i)&&(n!=="preview"||s&&!i)?"flex":"none"}),e.forEach(a=>{a.onclick=()=>{this.handleContextAction(a.dataset.action),this.hideContextMenu()}})},hideContextMenu(){document.getElementById("contextMenu").classList.remove("active")},handleContextAction(e){if(this.selectedFile)switch(e){case"open":this.selectedFile.isDir?this.navigateTo(this.selectedFile.path):this.openFile(this.selectedFile.path);break;case"preview":this.previewMarkdownFile(this.selectedFile.path);break;case"download":this.downloadFile(this.selectedFile.path);break;case"rename":this.openRenameModal();break;case"delete":this.openDeleteModal()}},openUploadModal(){this.filesToUpload=[],this.updateUploadList(),document.getElementById("uploadModal").classList.add("active")},handleFileSelect(e){var t=Array.from(e.target.files);this.addFilesToUpload(t),e.target.value=""},handleFileDrop(e){e=Array.from(e.dataTransfer.files),this.addFilesToUpload(e)},addFilesToUpload(e){e.forEach(t=>{this.filesToUpload.find(i=>i.name===t.name)||this.filesToUpload.push(t)}),this.updateUploadList()},updateUploadList(){var e=document.getElementById("uploadList"),t=document.getElementById("confirmUploadBtn");this.filesToUpload.length===0?(e.innerHTML="",t.disabled=!0):(t.disabled=!1,e.innerHTML=this.filesToUpload.map((i,o)=>`
      <div class="upload-item">
        <div class="upload-item-name">${i.name}</div>
        <div class="upload-item-size">${this.formatFileSize(i.size)}</div>
        <button class="upload-item-remove" onclick="FileManager.removeFileFromUpload(${o})">\u2715</button>
      </div>
    `).join(""))},removeFileFromUpload(e){this.filesToUpload.splice(e,1),this.updateUploadList()},uploadFiles(){return l(this,null,function*(){if(this.filesToUpload.length!==0){var e=document.getElementById("confirmUploadBtn");e.disabled=!0,e.textContent="\u4E0A\u4F20\u4E2D...";let o=0,s=0;for(const a of this.filesToUpload)try{var t=new FormData;t.append("file",a);var i=yield(yield fetch("/api/files?path="+encodeURIComponent(this.currentPath),{method:"POST",headers:{Authorization:this.getAuthHeader()},body:t})).json();i.success?o++:(s++,console.error("\u4E0A\u4F20\u5931\u8D25:",i.message))}catch(n){console.error("\u4E0A\u4F20\u5931\u8D25:",n),s++}this.closeModal(document.getElementById("uploadModal")),this.loadFiles(),0<o&&s===0?this.showToast(`\u6210\u529F\u4E0A\u4F20 ${o} \u4E2A\u6587\u4EF6`,"success"):0<o?this.showToast(`\u6210\u529F\u4E0A\u4F20 ${o} \u4E2A\u6587\u4EF6\uFF0C\u5931\u8D25 ${s} \u4E2A`,"warning"):this.showToast("\u4E0A\u4F20\u5931\u8D25","error"),e.disabled=!1,e.textContent="\u4E0A\u4F20"}})},openCreateDirModal(){document.getElementById("dirNameInput").value="",document.getElementById("createDirModal").classList.add("active"),setTimeout(()=>{document.getElementById("dirNameInput").focus()},100)},createDirectory(){return l(this,null,function*(){var e=document.getElementById("dirNameInput").value.trim();if(e)try{var t=yield(yield fetch("/api/files/create-dir",{method:"POST",headers:{"Content-Type":"application/json",Authorization:this.getAuthHeader()},body:JSON.stringify({path:this.currentPath,dir_name:e})})).json();t.success?(this.showToast("\u6587\u4EF6\u5939\u521B\u5EFA\u6210\u529F","success"),this.closeModal(document.getElementById("createDirModal")),this.loadFiles()):this.showToast(t.message,"error")}catch(i){console.error("\u521B\u5EFA\u76EE\u5F55\u5931\u8D25:",i),this.showToast("\u521B\u5EFA\u6587\u4EF6\u5939\u5931\u8D25","error")}else this.showToast("\u8BF7\u8F93\u5165\u6587\u4EF6\u5939\u540D\u79F0","warning")})},openRenameModal(){var e;this.selectedFile&&(e=this.selectedFile.path.split("/").pop(),document.getElementById("renameInput").value=e,document.getElementById("renameModal").classList.add("active"),setTimeout(()=>{document.getElementById("renameInput").focus(),document.getElementById("renameInput").select()},100))},renameFile(){return l(this,null,function*(){if(this.selectedFile){var e=document.getElementById("renameInput").value.trim();if(e)try{var t=yield(yield fetch("/api/files",{method:"PUT",headers:{"Content-Type":"application/json",Authorization:this.getAuthHeader()},body:JSON.stringify({old_path:this.selectedFile.path,new_name:e})})).json();t.success?(this.showToast("\u91CD\u547D\u540D\u6210\u529F","success"),this.closeModal(document.getElementById("renameModal")),this.loadFiles()):this.showToast(t.message,"error")}catch(i){console.error("\u91CD\u547D\u540D\u5931\u8D25:",i),this.showToast("\u91CD\u547D\u540D\u5931\u8D25","error")}else this.showToast("\u8BF7\u8F93\u5165\u65B0\u540D\u79F0","warning")}})},openDeleteModal(){var e;this.selectedFile&&(e=this.selectedFile.path.split("/").pop(),document.getElementById("deleteFileName").textContent=e,document.getElementById("deleteModal").classList.add("active"))},deleteFile(){return l(this,null,function*(){if(this.selectedFile)try{var e=yield(yield fetch("/api/files?path="+encodeURIComponent(this.selectedFile.path),{method:"DELETE",headers:{Authorization:this.getAuthHeader()}})).json();e.success?(this.showToast("\u5220\u9664\u6210\u529F","success"),this.closeModal(document.getElementById("deleteModal")),this.loadFiles()):this.showToast(e.message,"error")}catch(t){console.error("\u5220\u9664\u5931\u8D25:",t),this.showToast("\u5220\u9664\u5931\u8D25","error")}})},closeModal(e){e.classList.remove("active"),e.classList.add("closing"),setTimeout(()=>{e.classList.remove("closing")},300)},showToast(e,t="success"){const i=document.getElementById("toast");i.textContent=e,i.className=`toast ${t} active`,setTimeout(()=>{i.classList.remove("active")},3e3)}};document.addEventListener("DOMContentLoaded",()=>{h.init()});
