! function() {
    let d = [],
        c = 1;
    const s = 20;
    let r = 0;
    let selectedAttachments = window.selectedAttachments || new Set();
    window.currentAction = window.currentAction || null;
    window.currentItemId = window.currentItemId || null;
    window.selectedAttachments = selectedAttachments;
    const m = {
        uploadBtn: document.getElementById("amUploadBtn"),
        emptyUploadBtn: document.getElementById("amEmptyUploadBtn"),
        refreshBtn: document.getElementById("amRefreshBtn"),
        fileTypeFilter: document.getElementById("amFileTypeFilter"),
        visibilityFilter: document.getElementById("amVisibilityFilter"),
        passageFilter: document.getElementById("amPassageFilter"),
        searchInput: document.getElementById("amSearchInput"),
        tableBody: document.getElementById("attachmentsTableBody"),
        selectAll: document.getElementById("amSelectAll"),
        totalCount: document.getElementById("amTotalCount"),
        totalSize: document.getElementById("amTotalSize"),
        imageCount: document.getElementById("amImageCount"),
        documentCount: document.getElementById("amDocumentCount"),
        paginationContainer: document.getElementById("amPaginationContainer"),
        paginationInfo: document.getElementById("amPaginationInfo"),
        prevPageBtn: document.getElementById("amPrevPageBtn"),
        nextPageBtn: document.getElementById("amNextPageBtn"),
        paginationPages: document.getElementById("amPaginationPages"),
        batchActions: document.getElementById("amBatchActions"),
        selectedCount: document.getElementById("amSelectedCount"),
        batchDeleteBtn: document.getElementById("amBatchDeleteBtn"),
        batchSetPublicBtn: document.getElementById("amBatchSetPublicBtn"),
        batchSetPrivateBtn: document.getElementById("amBatchSetPrivateBtn"),
        cancelSelectionBtn: document.getElementById("amCancelSelectionBtn")
    };

    function u(e) {
        var t;
        return 0 === e ? "0 B" : (t = Math.floor(Math.log(e) / Math.log(1024)), Math.round(e / Math.pow(1024, t) * 100) / 100 + " " + ["B", "KB", "MB", "GB"][t])
    }

    function p(e) {
        return {
            image: '<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>️',
            document: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>',
            video: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect><line x1="7" y1="2" x2="7" y2="22"></line><line x1="17" y1="2" x2="17" y2="22"></line><line x1="2" y1="12" x2="22" y2="12"></line><line x1="2" y1="7" x2="7" y2="7"></line><line x1="2" y1="17" x2="7" y2="17"></line><line x1="17" y1="17" x2="22" y2="17"></line><line x1="17" y1="7" x2="22" y2="7"></line></svg>',
            audio: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>',
            archive: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>'
        } [e] || '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path></svg>'
    }

    function h() {
        if (0 === d.length) return m.tableBody.innerHTML = '\n        <tr>\n          <td colspan="9" style="text-align: center; padding: 40px;">\n            <div class="am-empty-state">\n              <div class="am-empty-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg></div>\n              <p>暂无附件</p>\n</div>\n          </td>\n        </tr>\n      ';
        m.tableBody.innerHTML = d.map(e => {
            var t = selectedAttachments.has(e.id),
                n = e.file_path;
            return `
        <tr class="${t?"selected":""}" data-id="${e.id}">
          <td>
            <input type="checkbox" class="am-checkbox" 
                   data-id="${e.id}" 
                   ${t?"checked":""}>
          </td>
          <td>
            ${"image"===e.file_type?`<img src="${n}" alt="${e.file_name}" 
                     style="width: 50px; height: 50px; object-fit: cover; border-radius: 4px; cursor: pointer;"
                     onclick="window.open('${n}', '_blank')">`:`<div style="width: 50px; height: 50px; display: flex; align-items: center; justify-content: center; background: #f5f5f5; border-radius: 4px;">
                ${p(e.file_type)}
               </div>`}
          </td>
          <td>
            <div style="font-weight: 500;">${e.file_name}</div>
            <div style="font-size: 0.85em; color: #888;">${e.stored_name}</div>
          </td>
          <td>${p(e.file_type)} ${e.file_type}</td>
          <td>${u(e.file_size)}</td>
          <td>${t=e.visibility,{public:'<span class="status published">公开</span>',private:'<span class="status draft">私密</span>',protected:'<span class="status pending">受保护</span>'}[t]||t}</td>
          <td>${e.passage_id?`<a href="/passage?id=${e.passage_id}" target="_blank">#${e.passage_id}</a>`:"-"}</td>
          <td>${n=e.uploaded_at,new Date(n).toLocaleString("zh-CN",{year:"numeric",month:"2-digit",day:"2-digit",hour:"2-digit",minute:"2-digit"})}</td>
          <td>
            <div class="action-buttons">
              <button class="btn btn-sm btn-view" onclick="viewAttachment(${e.id})" title="查看">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>️
              </button>
              <button class="btn btn-sm btn-edit" onclick="editAttachment(${e.id})" title="编辑权限">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
              </button>
              <button class="btn btn-sm btn-delete" onclick="deleteAttachment(${e.id})" title="删除">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>️
              </button>
            </div>
          </td>
        </tr>
      `
        }).join(""), document.querySelectorAll(".am-checkbox").forEach(e => {
            e.addEventListener("change", t)
        });
        var e = document.getElementById("amEmptyUploadBtn");
        e && e.addEventListener("click", n)
    }

    function t(e) {
        var t = parseInt(e.target.dataset.id);
        e.target.checked ? selectedAttachments.add(t) : (selectedAttachments.delete(t), m.selectAll.checked = !1), updateBatchActions()
    }

    function n() {
        document.body.insertAdjacentHTML("beforeend", '\n      <div class="modal active" id="uploadModal">\n        <div class="modal-content">\n          <div class="modal-header">\n            <h3>上传附件</h3>\n            <button class="modal-close" onclick="closeUploadModal()">×</button>\n          </div>\n          <div class="modal-body">\n            <div class="upload-area" id="uploadArea">\n              <div class="upload-icon"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg></div>\n              <div class="upload-text">\n                <h4>拖拽文件到这里或点击上传</h4>\n                <p>支持图片、文档、视频、音频、压缩包</p>\n              </div>\n              <input type="file" id="fileInput" multiple style="display: none;">\n            </div>\n            <div class="upload-preview" id="uploadPreview"></div>\n            <div class="form-group" style="margin-top: 20px;">\n              <label for="uploadPassageId">关联文章（可选）</label>\n              <select id="uploadPassageId" class="form-control">\n                <option value="">不关联</option>\n              </select>\n            </div>\n          </div>\n          <div class="btn-group" style="padding: 0 30px 30px;">\n            <button class="btn-secondary" onclick="closeUploadModal()">取消</button>\n            <button class="btn-primary" id="confirmUploadBtn" disabled>开始上传</button>\n          </div>\n        </div>\n      </div>\n    ');
        {
            document.getElementById("uploadModal");
            const o = document.getElementById("uploadArea"),
                e = document.getElementById("fileInput"),
                i = document.getElementById("uploadPreview"),
                l = document.getElementById("confirmUploadBtn"),
                d = document.getElementById("uploadPassageId");
            let a = [];

            function t(e) {
                a = Array.from(e), n(), l.disabled = 0 === a.length
            }

            function n() {
                i.innerHTML = a.map((e, t) => {
                    return `
        <div class="upload-item">
          ${e.type.startsWith("image/")?`<img src="${URL.createObjectURL(e)}" alt="${e.name}">`:`<div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f5f5f5;">
              ${p((n=e.type,n.startsWith("image/")?"image":n.startsWith("video/")?"video":n.startsWith("audio/")?"audio":!(n.includes("pdf")||n.includes("document")||n.includes("word")||n.includes("excel")||n.includes("powerpoint"))&&(n.includes("zip")||n.includes("tar")||n.includes("rar")||n.includes("7z"))?"archive":"document"))}
             </div>`}
          <button class="upload-remove" onclick="removeUploadFile(${t})">×</button>
          <div style="position: absolute; bottom: 0; left: 0; right: 0; background: rgba(0,0,0,0.7); color: white; font-size: 0.75em; padding: 4px; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            ${e.name}
          </div>
        </div>
      `;
                    var n
                }).join("")
            }(async function(n) {
                try {
                    var e = await (await fetch("/api/admin/passages?limit=100")).json();
                    e.success && e.data && e.data.forEach(e => {
                        var t = document.createElement("option");
                        t.value = e.id, t.textContent = e.title, n.appendChild(t)
                    })
                } catch (n) {
                    console.error("加载文章列表失败:", n)
                }
            })(d), o.addEventListener("click", () => e.click()), e.addEventListener("change", e => {
                t(e.target.files)
            }), o.addEventListener("dragover", e => {
                e.preventDefault(), o.classList.add("dragover")
            }), o.addEventListener("dragleave", () => {
                o.classList.remove("dragover")
            }), o.addEventListener("drop", e => {
                e.preventDefault(), o.classList.remove("dragover"), t(e.dataTransfer.files)
            }), l.addEventListener("click", async () => {
                if (0 !== a.length) {
                    l.disabled = !0, l.textContent = "上传中...";
                    const t = d.value;
                    for (const n of a) {
                        var e = new FormData;
                        e.append("file", n), t && e.append("passage_id", t);
                        try {
                            const t = await fetch("/api/admin/attachments", {
                                    method: "POST",
                                    body: e
                                }),
                                l = await t.json();
                            l.success || g(`上传 ${n.name} 失败: ` + l.message, "error")
                        } catch (t) {
                            console.error("上传失败:", t), g(`上传 ${n.name} 失败`, "error")
                        }
                    }
                    g("上传完成", "success"), closeUploadModal(), loadAttachments()
                }
            }), window.removeUploadFile = function(e) {
                a.splice(e, 1), n(), l.disabled = 0 === a.length
            }
        }
    }
    async function a() {
        var e;
        0 !== selectedAttachments.size && (window.currentAction = "batch-delete-attachment", window.currentItemId = Array.from(selectedAttachments).join(","), e = `确定要删除选中的 ${selectedAttachments.size} 个附件吗？此操作不可恢复。`, document.getElementById("confirmMessage").textContent = e, openModal("confirmModal"))
    }
    async function o(n) {
        if (0 !== selectedAttachments.size) {
            let e = 0,
                t = 0;
            for (const a of selectedAttachments) try {
                (await (await fetch("/api/admin/attachments/" + a, {
                    method: "PATCH",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        visibility: n
                    })
                })).json()).success ? e++ : t++
            } catch (n) {
                console.error("设置失败:", n), t++
            }
            0 < e && g(`成功设置 ${e} 个附件`, "success"), 0 < t && g(t + " 个附件设置失败", "error"), loadAttachments()
        }
    }

    function i() {
        selectedAttachments.clear(), m.selectAll.checked = !1, updateBatchActions(), h()
    }

    function g(e, t = "info") {
        const n = document.getElementById("toastContainer"),
            a = document.createElement("div");
        a.className = "toast " + t, a.innerHTML = `
      <span class="toast-icon">${"success"===t?"✓":"error"===t?"✗":"ℹ"}</span>
      <span class="toast-message">${e}</span>
      <button class="toast-close" onclick="this.parentElement.remove()">×</button>
    `, n.appendChild(a), setTimeout(() => {
            a.remove()
        }, 3e3)
    }

    function e() {
        var e = document.getElementById("adminShortcutsHelpBtn");
        e && window.adminKeyboardManager && e.addEventListener("click", () => {
            window.adminKeyboardManager.showAdminShortcutHelp()
        }), m.uploadBtn && m.uploadBtn.addEventListener("click", n), m.refreshBtn && m.refreshBtn.addEventListener("click", loadAttachments), m.fileTypeFilter && m.fileTypeFilter.addEventListener("change", () => {
            c = 1, loadAttachments()
        }), m.visibilityFilter && m.visibilityFilter.addEventListener("change", () => {
            c = 1, loadAttachments()
        }), m.passageFilter && m.passageFilter.addEventListener("change", () => {
            c = 1, loadAttachments()
        }), m.searchInput && m.searchInput.addEventListener("input", function() {
            let t;
            return function(...e) {
                clearTimeout(t), t = setTimeout(() => {
                    clearTimeout(t), [...e], c = 1, loadAttachments()
                }, 500)
            }
        }()), m.selectAll && m.selectAll.addEventListener("change", e => {
            const t = e.target.checked;
            document.querySelectorAll(".am-checkbox").forEach(e => {
                e.checked = t;
                e = parseInt(e.dataset.id);
                t ? selectedAttachments.add(e) : selectedAttachments.delete(e)
            }), updateBatchActions(), h()
        }), m.prevPageBtn && m.prevPageBtn.addEventListener("click", () => {
            1 < c && (c--, loadAttachments())
        }), m.nextPageBtn && m.nextPageBtn.addEventListener("click", () => {
            var e = Math.ceil(r / s);
            c < e && (c++, loadAttachments())
        }), m.batchDeleteBtn && m.batchDeleteBtn.addEventListener("click", a), m.batchSetPublicBtn && m.batchSetPublicBtn.addEventListener("click", () => o("public")), m.batchSetPrivateBtn && m.batchSetPrivateBtn.addEventListener("click", () => o("private")), m.cancelSelectionBtn && m.cancelSelectionBtn.addEventListener("click", i)
    }

    function l() {
        e(), loadAttachments(), fetch("/api/settings/appearance").then(e => e.json()).then(t => {
            function e() {
                var e = window.innerWidth <= 768 && t.mobile_background_image ? t.mobile_background_image : t.background_image;
                e && (document.body.style.backgroundImage = `url('${e}')`, document.body.style.backgroundSize = t.background_size || "cover", document.body.style.backgroundPosition = t.background_position || "center", document.body.style.backgroundRepeat = t.background_repeat || "no-repeat", document.body.style.backgroundAttachment = t.background_attachment || "fixed")
            }
            localStorage.setItem("appearanceSettings", JSON.stringify(t)), t.dark_mode_enabled && document.documentElement.classList.add("dark-mode"), (t.navbar_glass_color || t.card_glass_color || t.footer_glass_color || t.navbar_text_color) && (document.documentElement.style.setProperty("--navbar-glass-color", t.navbar_glass_color || "rgba(255, 255, 255, 0.85)"), document.documentElement.style.setProperty("--navbar-text-color", t.navbar_text_color || "rgba(255, 255, 255, 0.9)"), document.documentElement.style.setProperty("--card-glass-color", t.card_glass_color || "rgba(255, 255, 255, 0.75)"), document.documentElement.style.setProperty("--footer-glass-color", t.footer_glass_color || "rgba(255, 255, 255, 0.9)")), e(), window.addEventListener("resize", e)
        }).catch(e => {
            console.error("加载外观设置失败:", e)
        })
    }
    window.loadAttachments = async function() {
        try {
            const t = new URLSearchParams({
                    limit: s,
                    offset: (c - 1) * s
                }),
                n = m.fileTypeFilter.value,
                a = m.visibilityFilter.value,
                i = m.passageFilter.value,
                l = m.searchInput.value.trim();
            n && t.append("file_type", n), a && t.append("visibility", a), i && t.append("passage_id", i);
            var e = await (await fetch("/api/admin/attachments?" + t.toString())).json();
            if (e.success) {
                d = e.data || [], r = e.total || 0, l && (d = d.filter(e => e.file_name.toLowerCase().includes(l.toLowerCase())), r = d.length), h();
                {
                    m.totalCount.textContent = r;
                    let t = 0,
                        n = 0,
                        a = 0;
                    d.forEach(e => {
                        t += e.file_size, "image" === e.file_type && n++, "document" === e.file_type && a++
                    }), m.totalSize.textContent = u(t), m.imageCount.textContent = n, m.documentCount.textContent = a
                }
                var o = Math.ceil(r / s);
                if (o <= 1) m.paginationContainer.style.display = "none";
                else {
                    m.paginationContainer.style.display = "flex", m.paginationInfo.textContent = `显示 ${(c-1)*s+1}-${Math.min(c*s,r)} 条，共 ${r} 条`, m.prevPageBtn.disabled = 1 === c, m.nextPageBtn.disabled = c === o;
                    let t = "",
                        n = Math.max(1, c - Math.floor(2.5)),
                        a = Math.min(o, n + 5 - 1);
                    for (let e = n = a - n < 4 ? Math.max(1, a - 5 + 1) : n; e <= a; e++) t += `<button class="pagination-page ${e===c?"active":""}" 
                       data-page="${e}">${e}</button>`;
                    m.paginationPages.innerHTML = t, document.querySelectorAll("#amPaginationPages .pagination-page").forEach(e => {
                        e.addEventListener("click", () => {
                            c = parseInt(e.dataset.page), loadAttachments()
                        })
                    })
                }
            } else g("加载附件列表失败: " + e.message, "error")
        } catch (e) {
            console.error("加载附件列表失败:", e), g("加载附件列表失败，请检查网络连接", "error")
        }
    }, window.updateBatchActions = function() {
        var e = selectedAttachments.size;
        m.selectedCount.textContent = e, m.batchActions.style.display = 0 < e ? "flex" : "none"
    }, window.closeUploadModal = function() {
        var e = document.getElementById("uploadModal");
        e && e.remove()
    }, window.viewAttachment = function(t) {
        var e = d.find(e => e.id === t);
        e && window.open(e.file_path, "_blank")
    }, window.editAttachment = async function(e) {
        const t = document.getElementById("editModal");
        t && t.remove();
        try {
            const t = await fetch("/api/admin/attachments/" + e),
                a = await t.json();
            if (a.success && a.data) {
                const t = Array.isArray(a.data) ? a.data[0] : a.data;
                var n = `
          <div class="modal active" id="editModal">
            <div class="modal-content">
              <div class="modal-header">
                <h3>编辑附件权限</h3>
                <button class="modal-close" onclick="closeEditModal()">×</button>
              </div>
              <div class="modal-body">
                <div class="form-group">
                  <label>文件名</label>
                  <input type="text" class="form-control" value="${t.file_name}" disabled>
                </div>
                <div class="form-group">
                  <label for="editVisibility">可见性</label>
                  <select id="editVisibility" class="form-control">
                    <option value="public" ${"public"===(t.visibility||"public")?"selected":""}>公开</option>
                    <option value="private" ${"private"===(t.visibility||"public")?"selected":""}>私密</option>
                    <option value="protected" ${"protected"===(t.visibility||"public")?"selected":""}>受保护</option>
                  </select>
                </div>
                <div class="form-group">
                  <label for="editShowInPassage">
                    <input type="checkbox" id="editShowInPassage" ${t.show_in_passage?"checked":""}>
                    在文章中显示
                  </label>
                </div>
              </div>
              <div class="btn-group" style="padding: 0 30px 30px;">
                <button class="btn-secondary" onclick="closeEditModal()">取消</button>
                <button class="btn-primary" onclick="saveAttachmentSettings(${e})">保存</button>
              </div>
            </div>
          </div>
        `;
                document.body.insertAdjacentHTML("beforeend", n)
            } else g("获取附件信息失败: " + (a.message || "未知错误"), "error")
        } catch (e) {
            console.error("获取附件信息失败:", e), g("获取附件信息失败", "error")
        }
    }, window.saveAttachmentSettings = async function(e) {
        var t = document.getElementById("editVisibility").value,
            n = document.getElementById("editShowInPassage").checked;
        try {
            var a = await (await fetch("/api/admin/attachments/" + e, {
                method: "PATCH",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    visibility: t,
                    show_in_passage: n
                })
            })).json();
            a.success ? (g("保存成功", "success"), closeEditModal(), await loadAttachments()) : g("保存失败: " + a.message, "error")
        } catch (e) {
            g("保存失败", "error")
        }
    }, window.closeEditModal = function() {
        const e = document.getElementById("editModal");
        e && (e.classList.add("closing"), setTimeout(() => {
            e.remove()
        }, 300))
    }, window.deleteAttachment = async function(e) {
        window.currentAction = "delete-attachment";
        e = `确定要删除附件 #${window.currentItemId=e} 吗？此操作不可撤销。`;
        document.getElementById("confirmMessage").textContent = e, openModal("confirmModal")
    }, "loading" === document.readyState ? document.addEventListener("DOMContentLoaded", l) : l()
}();
