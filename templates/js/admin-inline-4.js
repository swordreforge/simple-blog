!(function () {
  let t = [],
    e = 1;
  const n = 20;
  let a = 0;
  const o = {
    uploadBtn: document.getElementById('amUploadBtn'),
    emptyUploadBtn: document.getElementById('amEmptyUploadBtn'),
    refreshBtn: document.getElementById('amRefreshBtn'),
    fileTypeFilter: document.getElementById('amFileTypeFilter'),
    visibilityFilter: document.getElementById('amVisibilityFilter'),
    passageFilter: document.getElementById('amPassageFilter'),
    searchInput: document.getElementById('amSearchInput'),
    tableBody: document.getElementById('attachmentsTableBody'),
    selectAll: document.getElementById('amSelectAll'),
    totalCount: document.getElementById('amTotalCount'),
    totalSize: document.getElementById('amTotalSize'),
    imageCount: document.getElementById('amImageCount'),
    documentCount: document.getElementById('amDocumentCount'),
    paginationContainer: document.getElementById('amPaginationContainer'),
    paginationInfo: document.getElementById('amPaginationInfo'),
    prevPageBtn: document.getElementById('amPrevPageBtn'),
    nextPageBtn: document.getElementById('amNextPageBtn'),
    paginationPages: document.getElementById('amPaginationPages'),
    batchActions: document.getElementById('amBatchActions'),
    selectedCount: document.getElementById('amSelectedCount'),
    batchDeleteBtn: document.getElementById('amBatchDeleteBtn'),
    batchSetPublicBtn: document.getElementById('amBatchSetPublicBtn'),
    batchSetPrivateBtn: document.getElementById('amBatchSetPrivateBtn'),
    cancelSelectionBtn: document.getElementById('amCancelSelectionBtn'),
  };
  function i(t) {
    if (0 === t) return '0 B';
    const e = Math.floor(Math.log(t) / Math.log(1024));
    return Math.round((t / Math.pow(1024, e)) * 100) / 100 + ' ' + ['B', 'KB', 'MB', 'GB'][e];
  }
  function d(t) {
    return (
      {
        image:
          '<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>️',
        document: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>`,
        video: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect><line x1="7" y1="2" x2="7" y2="22"></line><line x1="17" y1="2" x2="17" y2="22"></line><line x1="2" y1="12" x2="22" y2="12"></line><line x1="2" y1="7" x2="7" y2="7"></line><line x1="2" y1="17" x2="7" y2="17"></line><line x1="17" y1="17" x2="22" y2="17"></line><line x1="17" y1="7" x2="22" y2="7"></line></svg>`,
        audio: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>`,
        archive: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>`,
      }[t] ||
      `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path></svg>`
    );
  }
  function c() {
    if (0 === t.length)
      return void (o.tableBody.innerHTML =
        '\n        <tr>\n          <td colspan="9" style="text-align: center; padding: 40px;">\n            <div class="am-empty-state">\n              <div class="am-empty-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg></div>\n              <p>暂无附件</p>\n</div>\n          </td>\n        </tr>\n      ');
    ((o.tableBody.innerHTML = t
      .map(t => {
        const e = selectedAttachments.has(t.id),
          n = t.file_path,
          a = 'image' === t.file_type;
        return `\n        <tr class="${e ? 'selected' : ''}" data-id="${t.id}">\n          <td>\n            <input type="checkbox" class="am-checkbox" \n                   data-id="${t.id}" \n                   ${e ? 'checked' : ''}>\n          </td>\n          <td>\n            ${a ? `<img src="${n}" alt="${t.file_name}" \n                     style="width: 50px; height: 50px; object-fit: cover; border-radius: 4px; cursor: pointer;"\n                     onclick="window.open('${n}', '_blank')">` : `<div style="width: 50px; height: 50px; display: flex; align-items: center; justify-content: center; background: #f5f5f5; border-radius: 4px;">\n                ${d(t.file_type)}\n               </div>`}\n          </td>\n          <td>\n            <div style="font-weight: 500;">${t.file_name}</div>\n            <div style="font-size: 0.85em; color: #888;">${t.stored_name}</div>\n          </td>\n          <td>${d(t.file_type)} ${t.file_type}</td>\n          <td>${i(t.file_size)}</td>\n          <td>${((c = t.visibility), { public: '<span class="status published">公开</span>', private: '<span class="status draft">私密</span>', protected: '<span class="status pending">受保护</span>' }[c] || c)}</td>\n          <td>${t.passage_id ? `<a href="/passage?id=${t.passage_id}" target="_blank">#${t.passage_id}</a>` : '-'}</td>\n          <td>${((o = t.uploaded_at), new Date(o).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }))}</td>\n          <td>\n            <div class="action-buttons">\n              <button class="btn btn-sm btn-view" onclick="viewAttachment(${t.id})" title="查看">\n                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>️\n              </button>\n              <button class="btn btn-sm btn-edit" onclick="editAttachment(${t.id})" title="编辑权限">\n                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>\n              </button>\n              <button class="btn btn-sm btn-delete" onclick="deleteAttachment(${t.id})" title="删除">\n                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>️\n              </button>\n            </div>\n          </td>\n        </tr>\n      `;
        var o, c;
      })
      .join('')),
      document.querySelectorAll('.am-checkbox').forEach(t => {
        t.addEventListener('change', l);
      }));
    const e = document.getElementById('amEmptyUploadBtn');
    e && e.addEventListener('click', r);
  }
  function s() {
    const t = Math.ceil(a / n);
    if (t <= 1) return void (o.paginationContainer.style.display = 'none');
    ((o.paginationContainer.style.display = 'flex'),
      (o.paginationInfo.textContent = `显示 ${(e - 1) * n + 1}-${Math.min(e * n, a)} 条，共 ${a} 条`),
      (o.prevPageBtn.disabled = 1 === e),
      (o.nextPageBtn.disabled = e === t));
    let i = '';
    let d = Math.max(1, e - Math.floor(2.5)),
      c = Math.min(t, d + 5 - 1);
    c - d < 4 && (d = Math.max(1, c - 5 + 1));
    for (let t = d; t <= c; t++)
      i += `<button class="pagination-page ${t === e ? 'active' : ''}" \n                       data-page="${t}">${t}</button>`;
    ((o.paginationPages.innerHTML = i),
      document.querySelectorAll('#amPaginationPages .pagination-page').forEach(t => {
        t.addEventListener('click', () => {
          ((e = parseInt(t.dataset.page)), loadAttachments());
        });
      }));
  }
  function l(t) {
    const e = parseInt(t.target.dataset.id);
    (t.target.checked
      ? selectedAttachments.add(e)
      : (selectedAttachments.delete(e), (o.selectAll.checked = !1)),
      updateBatchActions());
  }
  function r() {
    (document.body.insertAdjacentHTML(
      'beforeend',
      '\n      <div class="modal active" id="uploadModal">\n        <div class="modal-content">\n          <div class="modal-header">\n            <h3>上传附件</h3>\n            <button class="modal-close" onclick="closeUploadModal()">×</button>\n          </div>\n          <div class="modal-body">\n            <div class="upload-area" id="uploadArea">\n              <div class="upload-icon"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg></div>\n              <div class="upload-text">\n                <h4>拖拽文件到这里或点击上传</h4>\n                <p>支持图片、文档、视频、音频、压缩包</p>\n              </div>\n              <input type="file" id="fileInput" multiple style="display: none;">\n            </div>\n            <div class="upload-preview" id="uploadPreview"></div>\n            <div class="form-group" style="margin-top: 20px;">\n              <label for="uploadPassageId">关联文章（可选）</label>\n              <select id="uploadPassageId" class="form-control">\n                <option value="">不关联</option>\n              </select>\n            </div>\n          </div>\n          <div class="btn-group" style="padding: 0 30px 30px;">\n            <button class="btn-secondary" onclick="closeUploadModal()">取消</button>\n            <button class="btn-primary" id="confirmUploadBtn" disabled>开始上传</button>\n          </div>\n        </div>\n      </div>\n    '
    ),
      (function () {
        document.getElementById('uploadModal');
        const t = document.getElementById('uploadArea'),
          e = document.getElementById('fileInput'),
          n = document.getElementById('uploadPreview'),
          a = document.getElementById('confirmUploadBtn'),
          o = document.getElementById('uploadPassageId');
        let i = [];
        function c(t) {
          ((i = Array.from(t)), s(), (a.disabled = 0 === i.length));
        }
        function s() {
          n.innerHTML = i
            .map((t, e) => {
              return `\n        <div class="upload-item">\n          ${t.type.startsWith('image/') ? `<img src="${URL.createObjectURL(t)}" alt="${t.name}">` : `<div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f5f5f5;">\n              ${d(((n = t.type), n.startsWith('image/') ? 'image' : n.startsWith('video/') ? 'video' : n.startsWith('audio/') ? 'audio' : n.includes('pdf') || n.includes('document') || n.includes('word') || n.includes('excel') || n.includes('powerpoint') ? 'document' : n.includes('zip') || n.includes('tar') || n.includes('rar') || n.includes('7z') ? 'archive' : 'document'))}\n             </div>`}\n          <button class="upload-remove" onclick="removeUploadFile(${e})">×</button>\n          <div style="position: absolute; bottom: 0; left: 0; right: 0; background: rgba(0,0,0,0.7); color: white; font-size: 0.75em; padding: 4px; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">\n            ${t.name}\n          </div>\n        </div>\n      `;
              var n;
            })
            .join('');
        }
        ((async function (t) {
          try {
            const e = await fetch('/api/admin/passages?limit=100'),
              n = await e.json();
            n.success &&
              n.data &&
              n.data.forEach(e => {
                const n = document.createElement('option');
                ((n.value = e.id), (n.textContent = e.title), t.appendChild(n));
              });
          } catch (t) {
            console.error('加载文章列表失败:', t);
          }
        })(o),
          t.addEventListener('click', () => e.click()),
          e.addEventListener('change', t => {
            c(t.target.files);
          }),
          t.addEventListener('dragover', e => {
            (e.preventDefault(), t.classList.add('dragover'));
          }),
          t.addEventListener('dragleave', () => {
            t.classList.remove('dragover');
          }),
          t.addEventListener('drop', e => {
            (e.preventDefault(), t.classList.remove('dragover'), c(e.dataTransfer.files));
          }),
          a.addEventListener('click', async () => {
            if (0 === i.length) return;
            ((a.disabled = !0), (a.textContent = '上传中...'));
            const t = o.value;
            for (const e of i) {
              const n = new FormData();
              (n.append('file', e), t && n.append('passage_id', t));
              try {
                const t = await fetch('/api/admin/attachments', { method: 'POST', body: n }),
                  a = await t.json();
                a.success || g(`上传 ${e.name} 失败: ${a.message}`, 'error');
              } catch (t) {
                (console.error('上传失败:', t), g(`上传 ${e.name} 失败`, 'error'));
              }
            }
            (g('上传完成', 'success'), closeUploadModal(), loadAttachments());
          }),
          (window.removeUploadFile = function (t) {
            (i.splice(t, 1), s(), (a.disabled = 0 === i.length));
          }));
      })());
  }
  async function m() {
    if (0 === selectedAttachments.size) return;
    ((currentAction = 'batch-delete-attachment'),
      (currentItemId = Array.from(selectedAttachments).join(',')));
    const t = `确定要删除选中的 ${selectedAttachments.size} 个附件吗？此操作不可恢复。`;
    ((document.getElementById('confirmMessage').textContent = t), openModal('confirmModal'));
  }
  async function u(t) {
    if (0 === selectedAttachments.size) return;
    let e = 0,
      n = 0;
    for (const a of selectedAttachments)
      try {
        const o = await fetch(`/api/admin/attachments/${a}`, {
          method: 'PATCH',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ visibility: t }),
        });
        (await o.json()).success ? e++ : n++;
      } catch (t) {
        (console.error('设置失败:', t), n++);
      }
    (e > 0 && g(`成功设置 ${e} 个附件`, 'success'),
      n > 0 && g(`${n} 个附件设置失败`, 'error'),
      loadAttachments());
  }
  function p() {
    (selectedAttachments.clear(), (o.selectAll.checked = !1), updateBatchActions(), c());
  }
  function g(t, e = 'info') {
    const n = document.getElementById('toastContainer'),
      a = document.createElement('div');
    ((a.className = `toast ${e}`),
      (a.innerHTML = `\n      <span class="toast-icon">${'success' === e ? '✓' : 'error' === e ? '✗' : 'ℹ'}</span>\n      <span class="toast-message">${t}</span>\n      <button class="toast-close" onclick="this.parentElement.remove()">×</button>\n    `),
      n.appendChild(a),
      setTimeout(() => {
        a.remove();
      }, 3e3));
  }
  function h() {
    const t = document.getElementById('adminShortcutsHelpBtn');
    (t &&
      window.adminKeyboardManager &&
      t.addEventListener('click', () => {
        window.adminKeyboardManager.showAdminShortcutHelp();
      }),
      o.uploadBtn && o.uploadBtn.addEventListener('click', r),
      o.refreshBtn && o.refreshBtn.addEventListener('click', loadAttachments),
      o.fileTypeFilter &&
        o.fileTypeFilter.addEventListener('change', () => {
          ((e = 1), loadAttachments());
        }),
      o.visibilityFilter &&
        o.visibilityFilter.addEventListener('change', () => {
          ((e = 1), loadAttachments());
        }),
      o.passageFilter &&
        o.passageFilter.addEventListener('change', () => {
          ((e = 1), loadAttachments());
        }),
      o.searchInput &&
        o.searchInput.addEventListener(
          'input',
          (function (t, e) {
            let n;
            return function (...a) {
              const o = () => {
                (clearTimeout(n), t(...a));
              };
              (clearTimeout(n), (n = setTimeout(o, e)));
            };
          })(() => {
            ((e = 1), loadAttachments());
          }, 500)
        ),
      o.selectAll &&
        o.selectAll.addEventListener('change', t => {
          const e = t.target.checked;
          (document.querySelectorAll('.am-checkbox').forEach(t => {
            t.checked = e;
            const n = parseInt(t.dataset.id);
            e ? selectedAttachments.add(n) : selectedAttachments.delete(n);
          }),
            updateBatchActions(),
            c());
        }),
      o.prevPageBtn &&
        o.prevPageBtn.addEventListener('click', () => {
          e > 1 && (e--, loadAttachments());
        }),
      o.nextPageBtn &&
        o.nextPageBtn.addEventListener('click', () => {
          const t = Math.ceil(a / n);
          e < t && (e++, loadAttachments());
        }),
      o.batchDeleteBtn && o.batchDeleteBtn.addEventListener('click', m),
      o.batchSetPublicBtn && o.batchSetPublicBtn.addEventListener('click', () => u('public')),
      o.batchSetPrivateBtn && o.batchSetPrivateBtn.addEventListener('click', () => u('private')),
      o.cancelSelectionBtn && o.cancelSelectionBtn.addEventListener('click', p));
  }
  function y() {
    (h(),
      loadAttachments(),
      fetch('/api/settings/appearance')
        .then(t => t.json())
        .then(t => {
          function e() {
            const e =
              window.innerWidth <= 768 && t.mobile_background_image
                ? t.mobile_background_image
                : t.background_image;
            e &&
              ((document.body.style.backgroundImage = `url('${e}')`),
              (document.body.style.backgroundSize = t.background_size || 'cover'),
              (document.body.style.backgroundPosition = t.background_position || 'center'),
              (document.body.style.backgroundRepeat = t.background_repeat || 'no-repeat'),
              (document.body.style.backgroundAttachment = t.background_attachment || 'fixed'));
          }
          (localStorage.setItem('appearanceSettings', JSON.stringify(t)),
            t.dark_mode_enabled && document.documentElement.classList.add('dark-mode'),
            (t.navbar_glass_color ||
              t.card_glass_color ||
              t.footer_glass_color ||
              t.navbar_text_color) &&
              (document.documentElement.style.setProperty(
                '--navbar-glass-color',
                t.navbar_glass_color || 'rgba(255, 255, 255, 0.85)'
              ),
              document.documentElement.style.setProperty(
                '--navbar-text-color',
                t.navbar_text_color || 'rgba(255, 255, 255, 0.9)'
              ),
              document.documentElement.style.setProperty(
                '--card-glass-color',
                t.card_glass_color || 'rgba(255, 255, 255, 0.75)'
              ),
              document.documentElement.style.setProperty(
                '--footer-glass-color',
                t.footer_glass_color || 'rgba(255, 255, 255, 0.9)'
              )),
            e(),
            window.addEventListener('resize', e));
        })
        .catch(t => {
          console.error('加载外观设置失败:', t);
        }));
  }
  ((window.loadAttachments = async function () {
    try {
      const d = new URLSearchParams({ limit: n, offset: (e - 1) * n }),
        l = o.fileTypeFilter.value,
        r = o.visibilityFilter.value,
        m = o.passageFilter.value,
        u = o.searchInput.value.trim();
      (l && d.append('file_type', l),
        r && d.append('visibility', r),
        m && d.append('passage_id', m));
      const p = await fetch(`/api/admin/attachments?${d.toString()}`),
        h = await p.json();
      h.success
        ? ((t = h.data || []),
          (a = h.total || 0),
          u &&
            ((t = t.filter(t => t.file_name.toLowerCase().includes(u.toLowerCase()))),
            (a = t.length)),
          c(),
          (function () {
            o.totalCount.textContent = a;
            let e = 0,
              n = 0,
              d = 0;
            (t.forEach(t => {
              ((e += t.file_size),
                'image' === t.file_type && n++,
                'document' === t.file_type && d++);
            }),
              (o.totalSize.textContent = i(e)),
              (o.imageCount.textContent = n),
              (o.documentCount.textContent = d));
          })(),
          s())
        : g('加载附件列表失败: ' + h.message, 'error');
    } catch (t) {
      (console.error('加载附件列表失败:', t), g('加载附件列表失败，请检查网络连接', 'error'));
    }
  }),
    (window.updateBatchActions = function () {
      const t = selectedAttachments.size;
      ((o.selectedCount.textContent = t), (o.batchActions.style.display = t > 0 ? 'flex' : 'none'));
    }),
    (window.closeUploadModal = function () {
      const t = document.getElementById('uploadModal');
      t && t.remove();
    }),
    (window.viewAttachment = function (e) {
      const n = t.find(t => t.id === e);
      n && window.open(n.file_path, '_blank');
    }),
    (window.editAttachment = async function (t) {
      console.log('打开编辑模态框，ID:', t);
      const e = document.getElementById('editModal');
      e && e.remove();
      try {
        const e = await fetch(`/api/admin/attachments/${t}`),
          n = await e.json();
        if (n.success && n.data) {
          const e = Array.isArray(n.data) ? n.data[0] : n.data;
          console.log('从API获取的附件数据:', e);
          const a = `\n          <div class="modal active" id="editModal">\n            <div class="modal-content">\n              <div class="modal-header">\n                <h3>编辑附件权限</h3>\n                <button class="modal-close" onclick="closeEditModal()">×</button>\n              </div>\n              <div class="modal-body">\n                <div class="form-group">\n                  <label>文件名</label>\n                  <input type="text" class="form-control" value="${e.file_name}" disabled>\n                </div>\n                <div class="form-group">\n                  <label for="editVisibility">可见性</label>\n                  <select id="editVisibility" class="form-control">\n                    <option value="public" ${'public' === (e.visibility || 'public') ? 'selected' : ''}>公开</option>\n                    <option value="private" ${'private' === (e.visibility || 'public') ? 'selected' : ''}>私密</option>\n                    <option value="protected" ${'protected' === (e.visibility || 'public') ? 'selected' : ''}>受保护</option>\n                  </select>\n                </div>\n                <div class="form-group">\n                  <label for="editShowInPassage">\n                    <input type="checkbox" id="editShowInPassage" ${e.show_in_passage ? 'checked' : ''}>\n                    在文章中显示\n                  </label>\n                </div>\n              </div>\n              <div class="btn-group" style="padding: 0 30px 30px;">\n                <button class="btn-secondary" onclick="closeEditModal()">取消</button>\n                <button class="btn-primary" onclick="saveAttachmentSettings(${t})">保存</button>\n              </div>\n            </div>\n          </div>\n        `;
          document.body.insertAdjacentHTML('beforeend', a);
        } else g('获取附件信息失败: ' + (n.message || '未知错误'), 'error');
      } catch (t) {
        (console.error('获取附件信息失败:', t), g('获取附件信息失败', 'error'));
      }
    }),
    (window.saveAttachmentSettings = async function (t) {
      const e = document.getElementById('editVisibility').value,
        n = document.getElementById('editShowInPassage').checked;
      console.log('保存附件设置:', { id: t, visibility: e, showInPassage: n });
      try {
        const a = await fetch(`/api/admin/attachments/${t}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ visibility: e, show_in_passage: n }),
          }),
          o = await a.json();
        (console.log('保存响应:', o),
          o.success
            ? (g('保存成功', 'success'),
              closeEditModal(),
              console.log('正在重新加载附件列表...'),
              await loadAttachments(),
              console.log('附件列表重新加载完成'))
            : g('保存失败: ' + o.message, 'error'));
      } catch (t) {
        (console.error('保存失败:', t), g('保存失败', 'error'));
      }
    }),
    (window.closeEditModal = function () {
      const t = document.getElementById('editModal');
      t &&
        (t.classList.add('closing'),
        setTimeout(() => {
          t.remove();
        }, 300));
    }),
    (window.deleteAttachment = async function (t) {
      ((currentAction = 'delete-attachment'), (currentItemId = t));
      const e = `确定要删除附件 #${t} 吗？此操作不可撤销。`;
      ((document.getElementById('confirmMessage').textContent = e), openModal('confirmModal'));
    }),
    'loading' === document.readyState ? document.addEventListener('DOMContentLoaded', y) : y());
})();
