/* ESBuild compressed */
var b=(g,d,p)=>new Promise((y,c)=>{var n=h=>{try{f(p.next(h))}catch(B){c(B)}},E=h=>{try{f(p.throw(h))}catch(B){c(B)}},f=h=>h.done?y(h.value):Promise.resolve(h.value).then(n,E);f((p=p.apply(g,d)).next())});(function(){let g=[],d=1;const p=20;let y=0,c=window.selectedAttachments||new Set;window.currentAction=window.currentAction||null,window.currentItemId=window.currentItemId||null,window.selectedAttachments=c;const n={uploadBtn:document.getElementById("amUploadBtn"),emptyUploadBtn:document.getElementById("amEmptyUploadBtn"),refreshBtn:document.getElementById("amRefreshBtn"),fileTypeFilter:document.getElementById("amFileTypeFilter"),visibilityFilter:document.getElementById("amVisibilityFilter"),passageFilter:document.getElementById("amPassageFilter"),searchInput:document.getElementById("amSearchInput"),tableBody:document.getElementById("attachmentsTableBody"),selectAll:document.getElementById("amSelectAll"),totalCount:document.getElementById("amTotalCount"),totalSize:document.getElementById("amTotalSize"),imageCount:document.getElementById("amImageCount"),documentCount:document.getElementById("amDocumentCount"),paginationContainer:document.getElementById("amPaginationContainer"),paginationInfo:document.getElementById("amPaginationInfo"),prevPageBtn:document.getElementById("amPrevPageBtn"),nextPageBtn:document.getElementById("amNextPageBtn"),paginationPages:document.getElementById("amPaginationPages"),batchActions:document.getElementById("amBatchActions"),selectedCount:document.getElementById("amSelectedCount"),batchDeleteBtn:document.getElementById("amBatchDeleteBtn"),batchSetPublicBtn:document.getElementById("amBatchSetPublicBtn"),batchSetPrivateBtn:document.getElementById("amBatchSetPrivateBtn"),cancelSelectionBtn:document.getElementById("amCancelSelectionBtn")};function E(t){var e;return t===0?"0 B":(e=Math.floor(Math.log(t)/Math.log(1024)),Math.round(t/Math.pow(1024,e)*100)/100+" "+["B","KB","MB","GB"][e])}function f(t){return{image:'<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>\uFE0F',document:'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>',video:'<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect><line x1="7" y1="2" x2="7" y2="22"></line><line x1="17" y1="2" x2="17" y2="22"></line><line x1="2" y1="12" x2="22" y2="12"></line><line x1="2" y1="7" x2="7" y2="7"></line><line x1="2" y1="17" x2="7" y2="17"></line><line x1="17" y1="17" x2="22" y2="17"></line><line x1="17" y1="7" x2="22" y2="7"></line></svg>',audio:'<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>',archive:'<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>'}[t]||'<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path></svg>'}function h(){if(g.length===0)return n.tableBody.innerHTML=`
        <tr>
          <td colspan="9" style="text-align: center; padding: 40px;">
            <div class="am-empty-state">
              <div class="am-empty-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg></div>
              <p>\u6682\u65E0\u9644\u4EF6</p>
</div>
          </td>
        </tr>
      `;n.tableBody.innerHTML=g.map(e=>{var i=c.has(e.id),a=e.file_path;return`
        <tr class="${i?"selected":""}" data-id="${e.id}">
          <td>
            <input type="checkbox" class="am-checkbox" 
                   data-id="${e.id}" 
                   ${i?"checked":""}>
          </td>
          <td>
            ${e.file_type==="image"?`<img src="${a}" alt="${e.file_name}" 
                     style="width: 50px; height: 50px; object-fit: cover; border-radius: 4px; cursor: pointer;"
                     onclick="window.open('${a}', '_blank')">`:`<div style="width: 50px; height: 50px; display: flex; align-items: center; justify-content: center; background: #f5f5f5; border-radius: 4px;">
                ${f(e.file_type)}
               </div>`}
          </td>
          <td>
            <div style="font-weight: 500;">${e.file_name}</div>
            <div style="font-size: 0.85em; color: #888;">${e.stored_name}</div>
          </td>
          <td>${f(e.file_type)} ${e.file_type}</td>
          <td>${E(e.file_size)}</td>
          <td>${i=e.visibility,{public:'<span class="status published">\u516C\u5F00</span>',private:'<span class="status draft">\u79C1\u5BC6</span>',protected:'<span class="status pending">\u53D7\u4FDD\u62A4</span>'}[i]||i}</td>
          <td>${e.passage_id?`<a href="/passage?id=${e.passage_id}" target="_blank">#${e.passage_id}</a>`:"-"}</td>
          <td>${a=e.uploaded_at,new Date(a).toLocaleString("zh-CN",{year:"numeric",month:"2-digit",day:"2-digit",hour:"2-digit",minute:"2-digit"})}</td>
          <td>
            <div class="action-buttons">
              <button class="btn btn-sm btn-view" onclick="viewAttachment(${e.id})" title="\u67E5\u770B">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>\uFE0F
              </button>
              <button class="btn btn-sm btn-edit" onclick="editAttachment(${e.id})" title="\u7F16\u8F91\u6743\u9650">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
              </button>
              <button class="btn btn-sm btn-delete" onclick="deleteAttachment(${e.id})" title="\u5220\u9664">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>\uFE0F
              </button>
            </div>
          </td>
        </tr>
      `}).join(""),document.querySelectorAll(".am-checkbox").forEach(e=>{e.addEventListener("change",B)});var t=document.getElementById("amEmptyUploadBtn");t&&t.addEventListener("click",k)}function B(t){var e=parseInt(t.target.dataset.id);t.target.checked?c.add(e):(c.delete(e),n.selectAll.checked=!1),updateBatchActions()}function k(){document.body.insertAdjacentHTML("beforeend",`
      <div class="modal active" id="uploadModal">
        <div class="modal-content">
          <div class="modal-header">
            <h3>\u4E0A\u4F20\u9644\u4EF6</h3>
            <button class="modal-close" onclick="closeUploadModal()">\xD7</button>
          </div>
          <div class="modal-body">
            <div class="upload-area" id="uploadArea">
              <div class="upload-icon"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg></div>
              <div class="upload-text">
                <h4>\u62D6\u62FD\u6587\u4EF6\u5230\u8FD9\u91CC\u6216\u70B9\u51FB\u4E0A\u4F20</h4>
                <p>\u652F\u6301\u56FE\u7247\u3001\u6587\u6863\u3001\u89C6\u9891\u3001\u97F3\u9891\u3001\u538B\u7F29\u5305</p>
              </div>
              <input type="file" id="fileInput" multiple style="display: none;">
            </div>
            <div class="upload-preview" id="uploadPreview"></div>
            <div class="form-group" style="margin-top: 20px;">
              <label for="uploadPassageId">\u5173\u8054\u6587\u7AE0\uFF08\u53EF\u9009\uFF09</label>
              <select id="uploadPassageId" class="form-control">
                <option value="">\u4E0D\u5173\u8054</option>
              </select>
            </div>
          </div>
          <div class="btn-group" style="padding: 0 30px 30px;">
            <button class="btn-secondary" onclick="closeUploadModal()">\u53D6\u6D88</button>
            <button class="btn-primary" id="confirmUploadBtn" disabled>\u5F00\u59CB\u4E0A\u4F20</button>
          </div>
        </div>
      </div>
    `);{let x=function(l){s=Array.from(l),v(),a.disabled=s.length===0},v=function(){i.innerHTML=s.map((l,r)=>{return`
        <div class="upload-item">
          ${l.type.startsWith("image/")?`<img src="${URL.createObjectURL(l)}" alt="${l.name}">`:`<div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f5f5f5;">
              ${f((o=l.type,o.startsWith("image/")?"image":o.startsWith("video/")?"video":o.startsWith("audio/")?"audio":o.includes("pdf")||o.includes("document")||o.includes("word")||o.includes("excel")||o.includes("powerpoint")||!(o.includes("zip")||o.includes("tar")||o.includes("rar")||o.includes("7z"))?"document":"archive"))}
             </div>`}
          <button class="upload-remove" onclick="removeUploadFile(${r})">\xD7</button>
          <div style="position: absolute; bottom: 0; left: 0; right: 0; background: rgba(0,0,0,0.7); color: white; font-size: 0.75em; padding: 4px; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            ${l.name}
          </div>
        </div>
      `;var o}).join("")};document.getElementById("uploadModal");const t=document.getElementById("uploadArea"),e=document.getElementById("fileInput"),i=document.getElementById("uploadPreview"),a=document.getElementById("confirmUploadBtn"),m=document.getElementById("uploadPassageId");let s=[];(function(l){return b(this,null,function*(){try{var r=yield(yield fetch("/api/admin/passages?limit=100")).json();r.success&&r.data&&r.data.forEach(o=>{var w=document.createElement("option");w.value=o.id,w.textContent=o.title,l.appendChild(w)})}catch(o){console.error("\u52A0\u8F7D\u6587\u7AE0\u5217\u8868\u5931\u8D25:",o)}})})(m),t.addEventListener("click",()=>e.click()),e.addEventListener("change",l=>{x(l.target.files)}),t.addEventListener("dragover",l=>{l.preventDefault(),t.classList.add("dragover")}),t.addEventListener("dragleave",()=>{t.classList.remove("dragover")}),t.addEventListener("drop",l=>{l.preventDefault(),t.classList.remove("dragover"),x(l.dataTransfer.files)}),a.addEventListener("click",()=>b(this,null,function*(){if(s.length!==0){a.disabled=!0,a.textContent="\u4E0A\u4F20\u4E2D...";var l=m.value;for(const o of s){var r=new FormData;r.append("file",o),l&&r.append("passage_id",l);try{const w=yield fetch("/api/admin/attachments",{method:"POST",body:r}),_=yield w.json();_.success||u(`\u4E0A\u4F20 ${o.name} \u5931\u8D25: `+_.message,"error")}catch(w){console.error("\u4E0A\u4F20\u5931\u8D25:",w),u(`\u4E0A\u4F20 ${o.name} \u5931\u8D25`,"error")}}u("\u4E0A\u4F20\u5B8C\u6210","success"),closeUploadModal(),loadAttachments()}})),window.removeUploadFile=function(l){s.splice(l,1),v(),a.disabled=s.length===0}}}function M(){return b(this,null,function*(){var t;c.size!==0&&(window.currentAction="batch-delete-attachment",window.currentItemId=Array.from(c).join(","),t=`\u786E\u5B9A\u8981\u5220\u9664\u9009\u4E2D\u7684 ${c.size} \u4E2A\u9644\u4EF6\u5417\uFF1F\u6B64\u64CD\u4F5C\u4E0D\u53EF\u6062\u590D\u3002`,document.getElementById("confirmMessage").textContent=t,openModal("confirmModal"))})}function I(t){return b(this,null,function*(){if(c.size!==0){let e=0,i=0;for(const a of c)try{(yield(yield fetch("/api/admin/attachments/"+a,{method:"PATCH",headers:{"Content-Type":"application/json"},body:JSON.stringify({visibility:t})})).json()).success?e++:i++}catch(m){console.error("\u8BBE\u7F6E\u5931\u8D25:",m),i++}0<e&&u(`\u6210\u529F\u8BBE\u7F6E ${e} \u4E2A\u9644\u4EF6`,"success"),0<i&&u(i+" \u4E2A\u9644\u4EF6\u8BBE\u7F6E\u5931\u8D25","error"),loadAttachments()}})}function C(){c.clear(),n.selectAll.checked=!1,updateBatchActions(),h()}function u(t,e="info"){const i=document.getElementById("toastContainer"),a=document.createElement("div");a.className="toast "+e,a.innerHTML=`
      <span class="toast-icon">${e==="success"?"\u2713":e==="error"?"\u2717":"\u2139"}</span>
      <span class="toast-message">${t}</span>
      <button class="toast-close" onclick="this.parentElement.remove()">\xD7</button>
    `,i.appendChild(a),setTimeout(()=>{a.remove()},3e3)}function A(){var t;(t=document.getElementById("adminShortcutsHelpBtn"))&&window.adminKeyboardManager&&t.addEventListener("click",()=>{window.adminKeyboardManager.showAdminShortcutHelp()}),n.uploadBtn&&n.uploadBtn.addEventListener("click",k),n.refreshBtn&&n.refreshBtn.addEventListener("click",loadAttachments),n.fileTypeFilter&&n.fileTypeFilter.addEventListener("change",()=>{d=1,loadAttachments()}),n.visibilityFilter&&n.visibilityFilter.addEventListener("change",()=>{d=1,loadAttachments()}),n.passageFilter&&n.passageFilter.addEventListener("change",()=>{d=1,loadAttachments()}),n.searchInput&&n.searchInput.addEventListener("input",function(){let e;return function(...i){clearTimeout(e),e=setTimeout(()=>{clearTimeout(e),[...i],d=1,loadAttachments()},500)}}()),n.selectAll&&n.selectAll.addEventListener("change",e=>{const i=e.target.checked;document.querySelectorAll(".am-checkbox").forEach(a=>{a.checked=i,a=parseInt(a.dataset.id),i?c.add(a):c.delete(a)}),updateBatchActions(),h()}),n.prevPageBtn&&n.prevPageBtn.addEventListener("click",()=>{1<d&&(d--,loadAttachments())}),n.nextPageBtn&&n.nextPageBtn.addEventListener("click",()=>{var e=Math.ceil(y/p);d<e&&(d++,loadAttachments())}),n.batchDeleteBtn&&n.batchDeleteBtn.addEventListener("click",M),n.batchSetPublicBtn&&n.batchSetPublicBtn.addEventListener("click",()=>I("public")),n.batchSetPrivateBtn&&n.batchSetPrivateBtn.addEventListener("click",()=>I("private")),n.cancelSelectionBtn&&n.cancelSelectionBtn.addEventListener("click",C),loadAttachments(),fetch("/api/settings/appearance").then(e=>e.json()).then(e=>{function i(){var a=window.innerWidth<=768&&e.mobile_background_image?e.mobile_background_image:e.background_image;a&&(document.body.style.backgroundImage=`url('${a}')`,document.body.style.backgroundSize=e.background_size||"cover",document.body.style.backgroundPosition=e.background_position||"center",document.body.style.backgroundRepeat=e.background_repeat||"no-repeat",document.body.style.backgroundAttachment=e.background_attachment||"fixed")}localStorage.setItem("appearanceSettings",JSON.stringify(e)),e.dark_mode_enabled&&document.documentElement.classList.add("dark-mode"),(e.navbar_glass_color||e.card_glass_color||e.footer_glass_color||e.navbar_text_color)&&(document.documentElement.style.setProperty("--navbar-glass-color",e.navbar_glass_color||"rgba(255, 255, 255, 0.85)"),document.documentElement.style.setProperty("--navbar-text-color",e.navbar_text_color||"rgba(255, 255, 255, 0.9)"),document.documentElement.style.setProperty("--card-glass-color",e.card_glass_color||"rgba(255, 255, 255, 0.75)"),document.documentElement.style.setProperty("--footer-glass-color",e.footer_glass_color||"rgba(255, 255, 255, 0.9)")),i(),window.addEventListener("resize",i)}).catch(e=>{console.error("\u52A0\u8F7D\u5916\u89C2\u8BBE\u7F6E\u5931\u8D25:",e)})}window.loadAttachments=function(){return b(this,null,function*(){try{const i=new URLSearchParams({limit:p,offset:(d-1)*p}),a=n.fileTypeFilter.value,m=n.visibilityFilter.value,s=n.passageFilter.value,x=n.searchInput.value.trim();a&&i.append("file_type",a),m&&i.append("visibility",m),s&&i.append("passage_id",s);var t=yield(yield fetch("/api/admin/attachments?"+i.toString())).json();if(t.success){g=t.data||[],y=t.total||0,x&&(g=g.filter(v=>v.file_name.toLowerCase().includes(x.toLowerCase())),y=g.length),h();{n.totalCount.textContent=y;let v=0,l=0,r=0;g.forEach(o=>{v+=o.file_size,o.file_type==="image"&&l++,o.file_type==="document"&&r++}),n.totalSize.textContent=E(v),n.imageCount.textContent=l,n.documentCount.textContent=r}var e=Math.ceil(y/p);if(e<=1)n.paginationContainer.style.display="none";else{n.paginationContainer.style.display="flex",n.paginationInfo.textContent=`\u663E\u793A ${(d-1)*p+1}-${Math.min(d*p,y)} \u6761\uFF0C\u5171 ${y} \u6761`,n.prevPageBtn.disabled=d===1,n.nextPageBtn.disabled=d===e;let v="",l=Math.max(1,d-Math.floor(2.5)),r=Math.min(e,l+5-1);for(let o=r-l<4?Math.max(1,r-5+1):l;o<=r;o++)v+=`<button class="pagination-page ${o===d?"active":""}" 
                       data-page="${o}">${o}</button>`;n.paginationPages.innerHTML=v,document.querySelectorAll("#amPaginationPages .pagination-page").forEach(o=>{o.addEventListener("click",()=>{d=parseInt(o.dataset.page),loadAttachments()})})}}else u("\u52A0\u8F7D\u9644\u4EF6\u5217\u8868\u5931\u8D25: "+t.message,"error")}catch(i){console.error("\u52A0\u8F7D\u9644\u4EF6\u5217\u8868\u5931\u8D25:",i),u("\u52A0\u8F7D\u9644\u4EF6\u5217\u8868\u5931\u8D25\uFF0C\u8BF7\u68C0\u67E5\u7F51\u7EDC\u8FDE\u63A5","error")}})},window.updateBatchActions=function(){var t=c.size;n.selectedCount.textContent=t,n.batchActions.style.display=0<t?"flex":"none"},window.closeUploadModal=function(){var t=document.getElementById("uploadModal");t&&t.remove()},window.viewAttachment=function(t){var e=g.find(i=>i.id===t);e&&window.open(e.file_path,"_blank")},window.editAttachment=function(t){return b(this,null,function*(){const e=document.getElementById("editModal");e&&e.remove();try{const a=yield fetch("/api/admin/attachments/"+t),m=yield a.json();if(m.success&&m.data){const s=Array.isArray(m.data)?m.data[0]:m.data;var i=`
          <div class="modal active" id="editModal">
            <div class="modal-content">
              <div class="modal-header">
                <h3>\u7F16\u8F91\u9644\u4EF6\u6743\u9650</h3>
                <button class="modal-close" onclick="closeEditModal()">\xD7</button>
              </div>
              <div class="modal-body">
                <div class="form-group">
                  <label>\u6587\u4EF6\u540D</label>
                  <input type="text" class="form-control" value="${s.file_name}" disabled>
                </div>
                <div class="form-group">
                  <label for="editVisibility">\u53EF\u89C1\u6027</label>
                  <select id="editVisibility" class="form-control">
                    <option value="public" ${(s.visibility||"public")==="public"?"selected":""}>\u516C\u5F00</option>
                    <option value="private" ${(s.visibility||"public")==="private"?"selected":""}>\u79C1\u5BC6</option>
                    <option value="protected" ${(s.visibility||"public")==="protected"?"selected":""}>\u53D7\u4FDD\u62A4</option>
                  </select>
                </div>
                <div class="form-group">
                  <label for="editShowInPassage">
                    <input type="checkbox" id="editShowInPassage" ${s.show_in_passage?"checked":""}>
                    \u5728\u6587\u7AE0\u4E2D\u663E\u793A
                  </label>
                </div>
              </div>
              <div class="btn-group" style="padding: 0 30px 30px;">
                <button class="btn-secondary" onclick="closeEditModal()">\u53D6\u6D88</button>
                <button class="btn-primary" onclick="saveAttachmentSettings(${t})">\u4FDD\u5B58</button>
              </div>
            </div>
          </div>
        `;document.body.insertAdjacentHTML("beforeend",i)}else u("\u83B7\u53D6\u9644\u4EF6\u4FE1\u606F\u5931\u8D25: "+(m.message||"\u672A\u77E5\u9519\u8BEF"),"error")}catch(a){console.error("\u83B7\u53D6\u9644\u4EF6\u4FE1\u606F\u5931\u8D25:",a),u("\u83B7\u53D6\u9644\u4EF6\u4FE1\u606F\u5931\u8D25","error")}})},window.saveAttachmentSettings=function(t){return b(this,null,function*(){var e=document.getElementById("editVisibility").value,i=document.getElementById("editShowInPassage").checked;try{var a=yield(yield fetch("/api/admin/attachments/"+t,{method:"PATCH",headers:{"Content-Type":"application/json"},body:JSON.stringify({visibility:e,show_in_passage:i})})).json();a.success?(u("\u4FDD\u5B58\u6210\u529F","success"),closeEditModal(),yield loadAttachments()):u("\u4FDD\u5B58\u5931\u8D25: "+a.message,"error")}catch(m){u("\u4FDD\u5B58\u5931\u8D25","error")}})},window.closeEditModal=function(){const t=document.getElementById("editModal");t&&(t.classList.add("closing"),setTimeout(()=>{t.remove()},300))},window.deleteAttachment=function(t){return b(this,null,function*(){window.currentAction="delete-attachment",t=`\u786E\u5B9A\u8981\u5220\u9664\u9644\u4EF6 #${window.currentItemId=t} \u5417\uFF1F\u6B64\u64CD\u4F5C\u4E0D\u53EF\u64A4\u9500\u3002`,document.getElementById("confirmMessage").textContent=t,openModal("confirmModal")})},document.readyState==="loading"?document.addEventListener("DOMContentLoaded",A):A()})();
