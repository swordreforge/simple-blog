// 文件管理器状态
const FileManager = {
  currentPath: 'img',
  currentRoot: 'img',
  selectedFile: null,
  filesToUpload: [],

  // 获取认证头
  getAuthHeader() {
    const token = this.getCookie('auth_token');
    return `Bearer ${token}`;
  },

  // 获取Cookie
  getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(';').shift();
    return '';
  },

  // 初始化
  init() {
    this.bindEvents();
    this.loadFiles();
  },

  // 绑定事件
  bindEvents() {
    // 返回按钮
    document.getElementById('backBtn').addEventListener('click', () => this.goBack());

    // 上传按钮
    document.getElementById('uploadBtn').addEventListener('click', () => this.openUploadModal());

    // 新建文件夹按钮
    document
      .getElementById('createDirBtn')
      .addEventListener('click', () => this.openCreateDirModal());

    // 根目录切换
    document.querySelectorAll('.fm-root-btn').forEach(btn => {
      btn.addEventListener('click', e => {
        const path = e.currentTarget.dataset.path;
        this.switchRoot(path);
      });
    });

    // 模态框关闭
    document.querySelectorAll('.modal-close, .fm-modal-close-btn, .fm-modal-close').forEach(btn => {
      btn.addEventListener('click', e => {
        const modal = e.target.closest('.modal') || e.target.closest('.fm-modal');
        if (modal) {
          this.closeModal(modal);
        }
      });
    });

    // 上传区域
    const uploadArea = document.getElementById('uploadArea');
    const fileInput = document.getElementById('fileInput');

    if (uploadArea && fileInput) {
      // 点击上传区域触发文件选择
      uploadArea.addEventListener('click', e => {
        e.stopPropagation();
        e.preventDefault();
        fileInput.click();
      });

      // 文件选择变化事件
      fileInput.addEventListener('change', e => {
        e.stopPropagation();
        this.handleFileSelect(e);
      });

      // 拖拽悬停效果
      uploadArea.addEventListener('dragover', e => {
        e.preventDefault();
        e.stopPropagation();
        uploadArea.classList.add('dragover');
      });

      // 拖拽离开效果
      uploadArea.addEventListener('dragleave', e => {
        e.preventDefault();
        e.stopPropagation();
        uploadArea.classList.remove('dragover');
      });

      // 拖拽放下事件
      uploadArea.addEventListener('drop', e => {
        e.preventDefault();
        e.stopPropagation();
        uploadArea.classList.remove('dragover');
        this.handleFileDrop(e);
      });
    } else {
      console.error('上传区域或文件输入框未找到');
    }

    // 确认上传
    document.getElementById('confirmUploadBtn').addEventListener('click', () => this.uploadFiles());

    // 确认创建目录
    document
      .getElementById('confirmCreateDirBtn')
      .addEventListener('click', () => this.createDirectory());

    // 确认重命名
    document.getElementById('confirmRenameBtn').addEventListener('click', () => this.renameFile());

    // 确认删除
    document.getElementById('confirmDeleteBtn').addEventListener('click', () => this.deleteFile());

    // 点击外部关闭上下文菜单
    document.addEventListener('click', e => {
      if (!e.target.closest('.context-menu') && !e.target.closest('.file-item')) {
        this.hideContextMenu();
      }
    });

    // 键盘事件
    document.addEventListener('keydown', e => {
      if (e.key === 'Escape') {
        this.hideContextMenu();
        document.querySelectorAll('.modal.active, .fm-modal.active').forEach(modal => {
          this.closeModal(modal);
        });
      }
    });
  },

  // 加载文件列表
  async loadFiles() {
    // 如果是附件管理，加载附件列表
    if (this.currentRoot === 'attachments') {
      await this.loadAttachments();
      return;
    }

    try {
      const response = await fetch(`/api/files?path=${encodeURIComponent(this.currentPath)}`, {
        headers: {
          Authorization: this.getAuthHeader(),
        },
      });
      const result = await response.json();

      if (result.success) {
        this.renderFiles(result.data.files);
        this.updateBreadcrumb(result.data.current_path);
        this.updateBackButton(result.data.parent_path);
        this.updateFileCount(result.data.files.length);
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('加载文件失败:', error);
      this.showToast('加载文件失败', 'error');
    }
  },

  // 加载附件列表
  async loadAttachments() {
    try {
      const response = await fetch('/api/admin/attachments', {
        headers: {
          Authorization: this.getAuthHeader(),
        },
      });
      const result = await response.json();

      if (result.success) {
        this.currentAttachments = result.data;
        this.renderAttachments(result.data);
        this.updateBreadcrumb('/attachments');
        this.updateBackButton(null);
        this.updateFileCount(result.total);
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('加载附件失败:', error);
      this.showToast('加载附件失败', 'error');
    }
  },

  // 渲染附件列表
  renderAttachments(attachments) {
    const fileGrid = document.getElementById('fileGrid');
    const emptyState = document.getElementById('emptyState');

    if (attachments.length === 0) {
      fileGrid.innerHTML = '';
      emptyState.style.display = 'flex';
      return;
    }

    emptyState.style.display = 'none';

    fileGrid.innerHTML = attachments.map(att => this.createAttachmentItem(att)).join('');

    // 绑定附件项事件
    fileGrid.querySelectorAll('.file-item').forEach(item => {
      item.addEventListener('click', e => {
        e.stopPropagation();
        const id = item.dataset.id;
        this.showAttachmentMenu(id, e);
      });
    });
  },

  // 创建附件项
  createAttachmentItem(attachment) {
    const visibilityIcon =
      {
        public: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>',
        private: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>',
        protected: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>',
      }[attachment.visibility] || '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>';

    const visibilityLabel =
      {
        public: '公开',
        private: '私密',
        protected: '受保护',
      }[attachment.visibility] || '公开';

    const showInPassageBadge = attachment.show_in_passage
      ? '<span class="badge badge-success">显示</span>'
      : '<span class="badge badge-secondary">隐藏</span>';

    return `
      <div class="file-item" data-id="${attachment.id}">
        <div class="file-icon">${this.getFileIcon(attachment.file_type)}</div>
        <div class="file-info">
          <div class="file-name">${attachment.file_name}</div>
          <div class="file-meta">
            <span>${visibilityIcon} ${visibilityLabel}</span>
            ${showInPassageBadge}
            <span>${this.formatFileSize(attachment.file_size)}</span>
          </div>
        </div>
      </div>
    `;
  },

  // 显示附件管理菜单
  showAttachmentMenu(id, event) {
    const attachment = this.currentAttachments?.find(a => a.id === parseInt(id));
    if (!attachment) return;

    // 创建菜单
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.style.position = 'absolute';
    menu.style.left = `${event.clientX}px`;
    menu.style.top = `${event.clientY}px`;

    menu.innerHTML = `
      <div class="context-menu-item" data-action="toggle-visibility">
        <span>切换可见性</span>
        <span class="context-menu-icon">${attachment.visibility === 'public' ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>' : '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>'}</span>
      </div>
      <div class="context-menu-item" data-action="toggle-show">
        <span>${attachment.show_in_passage ? '在文章中隐藏' : '在文章中显示'}</span>
        <span class="context-menu-icon">${attachment.show_in_passage ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>' : '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>'}</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item context-menu-danger" data-action="delete">
        <span>删除附件</span>
        <span class="context-menu-icon"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></span>
      </div>
    `;

    document.body.appendChild(menu);

    // 绑定菜单项事件
    menu.querySelectorAll('.context-menu-item').forEach(item => {
      item.addEventListener('click', e => {
        e.stopPropagation();
        const action = item.dataset.action;
        this.handleAttachmentAction(id, action);
        menu.remove();
      });
    });

    // 点击其他地方关闭菜单
    setTimeout(() => {
      document.addEventListener('click', function closeMenu() {
        menu.remove();
        document.removeEventListener('click', closeMenu);
      });
    }, 0);
  },

  // 处理附件操作
  async handleAttachmentAction(id, action) {
    switch (action) {
      case 'toggle-visibility':
        await this.toggleAttachmentVisibility(id);
        break;
      case 'toggle-show':
        await this.toggleAttachmentShow(id);
        break;
      case 'delete':
        await this.deleteAttachment(id);
        break;
    }
  },

  // 切换附件可见性
  async toggleAttachmentVisibility(id) {
    const attachment = this.currentAttachments?.find(a => a.id === parseInt(id));
    if (!attachment) return;

    const newVisibility = attachment.visibility === 'public' ? 'private' : 'public';

    try {
      const response = await fetch(`/api/admin/attachments/${id}`, {
        method: 'PATCH',
        headers: {
          Authorization: this.getAuthHeader(),
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          visibility: newVisibility,
        }),
      });
      const result = await response.json();

      if (result.success) {
        this.showToast('更新成功', 'success');
        this.loadAttachments();
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('更新附件失败:', error);
      this.showToast('更新附件失败', 'error');
    }
  },

  // 切换附件在文章中的显示
  async toggleAttachmentShow(id) {
    const attachment = this.currentAttachments?.find(a => a.id === parseInt(id));
    if (!attachment) return;

    try {
      const response = await fetch(`/api/admin/attachments?id=${id}`, {
        method: 'PATCH',
        headers: {
          Authorization: this.getAuthHeader(),
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          show_in_passage: !attachment.show_in_passage,
        }),
      });
      const result = await response.json();

      if (result.success) {
        this.showToast('更新成功', 'success');
        this.loadAttachments();
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('更新附件失败:', error);
      this.showToast('更新附件失败', 'error');
    }
  },

  // 删除附件
  async deleteAttachment(id) {
    if (!confirm('确定要删除此附件吗？此操作不可恢复。')) {
      return;
    }

    try {
      const response = await fetch(`/api/admin/attachments/${id}`, {
        method: 'DELETE',
        headers: {
          Authorization: this.getAuthHeader(),
        },
      });
      const result = await response.json();

      if (result.success) {
        this.showToast('删除成功', 'success');
        this.loadAttachments();
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('删除附件失败:', error);
      this.showToast('删除附件失败', 'error');
    }
  },

  // 格式化文件大小
  formatFileSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  },

  // 获取文件图标
  getFileIcon(fileType) {
    const icons = {
      // 图片 - 精美的相机/图片图标
      image: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="3" ry="3"/>
        <circle cx="8.5" cy="8.5" r="1.5"/>
        <path d="M21 15l-5-5L5 21"/>
      </svg>`,
      // 视频 - 精美的播放图标
      video: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="5 3 19 12 5 21 5 3" fill="currentColor"/>
        <rect x="2" y="2" width="20" height="20" rx="3" ry="3"/>
      </svg>`,
      // 音频 - 精美的音符图标
      audio: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 18V5l12-2v13"/>
        <circle cx="6" cy="18" r="3" fill="currentColor"/>
        <circle cx="18" cy="16" r="3" fill="currentColor"/>
      </svg>`,
      // 文档 - 精美的文件图标
      document: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <polyline points="10 9 9 9 8 9"/>
      </svg>`,
      // 压缩包 - 精美的盒子图标
      archive: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
        <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
        <line x1="12" y1="22.08" x2="12" y2="12"/>
      </svg>`,
      // Markdown - 精美的代码图标
      markdown: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <path d="M10 13l-2 2 2 2"/>
        <path d="M14 13l2 2-2 2"/>
      </svg>`,
      // 代码文件 - 精美的代码编辑器图标
      code: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M16 18l6-6-6-6"/>
        <path d="M8 6l-6 6 6 6"/>
      </svg>`,
      // PDF - 精美的PDF图标
      pdf: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <text x="7" y="17" font-size="6" font-weight="bold" fill="currentColor">PDF</text>
      </svg>`,
      // Word文档 - 精美的Word图标
      word: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <text x="4" y="17" font-size="5" font-weight="bold" fill="currentColor">DOC</text>
      </svg>`,
      // Excel表格 - 精美的Excel图标
      excel: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <rect x="7" y="10" width="10" height="2"/>
        <rect x="7" y="14" width="10" height="2"/>
        <rect x="7" y="18" width="10" height="2"/>
      </svg>`,
      // PPT演示文稿 - 精美的PPT图标
      ppt: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <rect x="8" y="11" width="8" height="6" rx="1"/>
      </svg>`,
      // 文本文件 - 精美的文本图标
      text: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <line x1="10" y1="9" x2="8" y2="9"/>
      </svg>`,
      // 字体文件 - 精美的字体图标
      font: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <text x="6" y="18" font-size="10" font-weight="bold" fill="currentColor">Aa</text>
      </svg>`,
      // 数据库文件 - 精美的数据库图标
      database: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <ellipse cx="12" cy="5" rx="9" ry="3"/>
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
      </svg>`,
      // 可执行文件 - 精美的齿轮图标
      executable: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <circle cx="12" cy="14" r="3"/>
        <line x1="12" y1="11" x2="12" y2="17"/>
        <line x1="9" y1="14" x2="15" y2="14"/>
      </svg>`,
    };
    return icons[fileType] || `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
      <polyline points="14 2 14 8 20 8"/>
      <line x1="12" y1="18" x2="12" y2="12"/>
      <line x1="9" y1="15" x2="15" y2="15"/>
    </svg>`;
  },

  // 渲染文件列表
  renderFiles(files) {
    const fileGrid = document.getElementById('fileGrid');
    const emptyState = document.getElementById('emptyState');

    if (files.length === 0) {
      fileGrid.innerHTML = '';
      emptyState.style.display = 'flex';
      return;
    }

    emptyState.style.display = 'none';

    // 先显示目录，再显示文件
    const sortedFiles = [...files].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });

    fileGrid.innerHTML = sortedFiles.map(file => this.createFileItem(file)).join('');

    // 绑定文件项事件
    fileGrid.querySelectorAll('.file-item').forEach(item => {
      item.addEventListener('click', e => {
        e.stopPropagation();
        const path = item.dataset.path;
        const isDir = item.dataset.isDir === 'true';

        if (isDir) {
          this.navigateTo(path);
        } else {
          // 检查是否是 markdown 文件
          if (path.toLowerCase().endsWith('.md')) {
            // Markdown 文件使用模态框预览
            this.previewMarkdownFile(path);
          } else {
            // 其他文件使用原有的打开逻辑
            this.openFile(path);
          }
        }
      });

      item.addEventListener('contextmenu', e => {
        e.preventDefault();
        const path = item.dataset.path;
        const isDir = item.dataset.isDir === 'true';
        this.showContextMenu(e, path, isDir);
      });
    });
  },

  // 创建文件项HTML
  createFileItem(file) {
    let icon;
    let typeClass = '';

    if (file.is_dir) {
      // 文件夹 - 精美的文件夹图标
      icon = `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>`;
      typeClass = 'directory';
    } else if (
      ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.svg', '.ico', '.tiff', '.tif', '.avif', '.jxl'].includes(
        file.extension
      )
    ) {
      // 图片文件 - 显示缩略图或使用精美图标
      icon = `<img src="/${file.path}" alt="${file.name}" onerror="this.parentElement.innerHTML='<svg width=\\'24\\' height=\\'24\\' viewBox=\\'0 0 24 24\\' fill=\\'none\\' stroke=\\'currentColor\\' stroke-width=\\'2\\'><rect x=\\'3\\' y=\\'3\\' width=\\'18\\' height=\\'18\\' rx=\\'3\\' ry=\\'3\\'></rect><circle cx=\\'8.5\\' cy=\\'8.5\\' r=\\'1.5\\'></circle><path d=\\'21 15l-5-5L5 21\\'></path></svg>'">`;
      typeClass = 'image';
    } else if (
      ['.mp3', '.flac', '.wav', '.ogg', '.m4a', '.aac', '.wma', '.opus', '.ape', '.wv', '.tta'].includes(
        file.extension
      )
    ) {
      icon = this.getFileIcon('audio');
      typeClass = 'audio';
    } else if (
      ['.mp4', '.webm', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.m4v', '.3gp', '.ts', '.m2ts'].includes(
        file.extension
      )
    ) {
      icon = this.getFileIcon('video');
      typeClass = 'video';
    } else if (file.extension === '.md') {
      icon = this.getFileIcon('markdown');
      typeClass = 'markdown';
    } else if (
      ['.pdf'].includes(file.extension)
    ) {
      icon = this.getFileIcon('pdf');
      typeClass = 'document pdf';
    } else if (
      ['.doc', '.docx', '.odt', '.rtf'].includes(file.extension)
    ) {
      icon = this.getFileIcon('word');
      typeClass = 'document word';
    } else if (
      ['.xls', '.xlsx', '.ods', '.csv'].includes(file.extension)
    ) {
      icon = this.getFileIcon('excel');
      typeClass = 'document excel';
    } else if (
      ['.ppt', '.pptx', '.odp'].includes(file.extension)
    ) {
      icon = this.getFileIcon('ppt');
      typeClass = 'document ppt';
    } else if (
      ['.txt', '.log', '.md'].includes(file.extension)
    ) {
      icon = this.getFileIcon('text');
      typeClass = 'document text';
    } else if (
      ['.zip', '.rar', '.7z', '.tar', '.gz', '.bz2', '.xz', '.tar.gz', '.tar.bz2', '.tar.xz'].includes(
        file.extension
      )
    ) {
      icon = this.getFileIcon('archive');
      typeClass = 'archive';
    } else if (
      ['.html', '.htm', '.css', '.js', '.ts', '.jsx', '.tsx', '.vue', '.svelte', '.json', '.xml', '.yaml', '.yml', '.toml', '.ini', '.cfg', '.conf'].includes(
        file.extension
      )
    ) {
      icon = this.getFileIcon('code');
      typeClass = 'code';
    } else if (
      ['.ttf', '.otf', '.woff', '.woff2', '.eot'].includes(file.extension)
    ) {
      icon = this.getFileIcon('font');
      typeClass = 'font';
    } else if (
      ['.db', '.sqlite', '.sqlite3', '.mdb', '.sql'].includes(file.extension)
    ) {
      icon = this.getFileIcon('database');
      typeClass = 'database';
    } else if (
      ['.exe', '.app', '.dmg', '.msi', '.deb', '.rpm', '.sh', '.bat', '.cmd', '.ps1'].includes(file.extension)
    ) {
      icon = this.getFileIcon('executable');
      typeClass = 'executable';
    } else {
      icon = this.getFileIcon('default');
      typeClass = 'file';
    }

    const size = this.formatFileSize(file.size);

    return `
      <div class="file-item ${typeClass}" data-path="${file.path}" data-is-dir="${file.is_dir}">
        <div class="file-icon">${icon}</div>
        <div class="file-name">${file.name}</div>
        <div class="file-meta">${file.is_dir ? '文件夹' : size}</div>
      </div>
    `;
  },

  // 格式化文件大小
  formatFileSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  },

  // 更新面包屑
  updateBreadcrumb(path) {
    document.getElementById('currentPath').textContent = path || '/';
  },

  // 更新返回按钮状态
  updateBackButton(parentPath) {
    const backBtn = document.getElementById('backBtn');
    backBtn.disabled = !parentPath;
  },

  // 更新文件计数
  updateFileCount(count) {
    document.getElementById('fileCount').textContent = `${count} 个项目`;
  },

  // 切换根目录
  switchRoot(root) {
    this.currentRoot = root;
    this.currentPath = root;

    // 更新根目录按钮状态
    document.querySelectorAll('.fm-root-btn').forEach(btn => {
      btn.classList.toggle('fm-root-btn-active', btn.dataset.path === root);
    });

    this.loadFiles();
  },

  // 导航到目录
  navigateTo(path) {
    this.currentPath = path;
    this.loadFiles();
  },

  // 返回上级目录
  goBack() {
    const parentPath = this.getParentPath(this.currentPath);
    if (parentPath) {
      this.navigateTo(parentPath);
    }
  },

  // 获取父目录路径
  getParentPath(path) {
    if (path === this.currentRoot) {
      return null;
    }
    const parts = path.split('/');
    parts.pop();
    const parent = parts.join('/');
    return parent || this.currentRoot;
  },

  // 打开文件
  async openFile(path) {
    const extension = path.split('.').pop().toLowerCase();
    const fileName = path.split('/').pop();

    // Markdown 文件 - 使用模态框预览
    if (extension === 'md') {
      this.previewMarkdownFile(path);
      return;
    }

    // 图片文件 - 在线预览
    if (
      ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.svg', '.ico', '.tiff', '.tif'].includes(
        '.' + extension
      )
    ) {
      this.openImagePreview(path, fileName);
      return;
    }

    // 音频文件 - 在线播放
    if (['.mp3', '.flac', '.wav', '.ogg', '.m4a', '.aac', '.wma'].includes('.' + extension)) {
      this.openAudioPreview(path, fileName);
      return;
    }

    // 视频文件 - 在线播放
    if (['.mp4', '.webm', '.mkv', '.avi', '.mov', '.wmv', '.flv'].includes('.' + extension)) {
      this.openVideoPreview(path, fileName);
      return;
    }

    // 文档文件 - 在线预览
    if (
      ['.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx', '.txt'].includes('.' + extension)
    ) {
      this.openDocumentPreview(path, fileName, extension);
      return;
    }

    // 其他文件 - 直接下载
    try {
      const response = await fetch(`/api/files/download?path=${encodeURIComponent(path)}`, {
        headers: {
          Authorization: this.getAuthHeader(),
        },
      });

      if (!response.ok) {
        const result = await response.json();
        this.showToast(result.message || '下载失败', 'error');
        return;
      }

      // 创建 blob 并下载
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('下载失败:', error);
      this.showToast('下载失败', 'error');
    }
  },

  // 打开图片预览
  openImagePreview(path, fileName) {
    const imageUrl = `/${path}`;
    const modal = document.createElement('div');
    modal.className = 'fm-modal preview-modal';
    modal.innerHTML = `
      <div class="fm-modal-content preview-content">
        <div class="fm-modal-header">
          <h3>${fileName}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <img src="${imageUrl}" alt="${fileName}" class="preview-image">
        </div>
      </div>
    `;
    document.body.appendChild(modal);

    // 关闭事件
    const closeBtn = modal.querySelector('.fm-modal-close');
    const closeModal = () => {
      document.body.removeChild(modal);
    };
    closeBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', e => {
      if (e.target === modal) closeModal();
    });

    // ESC 键关闭
    const escHandler = e => {
      if (e.key === 'Escape') {
        closeModal();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);

    modal.classList.add('active');
  },

  // 打开音频预览
  openAudioPreview(path, fileName) {
    const audioUrl = `/${path}`;
    const modal = document.createElement('div');
    modal.className = 'fm-modal preview-modal';
    modal.innerHTML = `
      <div class="fm-modal-content preview-content audio-preview">
        <div class="fm-modal-header">
          <h3>${fileName}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <div class="audio-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg></div>
          <audio controls autoplay class="preview-audio">
            <source src="${audioUrl}" type="audio/${path.split('.').pop()}">
            您的浏览器不支持音频播放
          </audio>
        </div>
      </div>
    `;
    document.body.appendChild(modal);

    // 关闭事件
    const closeBtn = modal.querySelector('.fm-modal-close');
    const closeModal = () => {
      const audio = modal.querySelector('audio');
      if (audio) audio.pause();
      document.body.removeChild(modal);
    };
    closeBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', e => {
      if (e.target === modal) closeModal();
    });

    // ESC 键关闭
    const escHandler = e => {
      if (e.key === 'Escape') {
        closeModal();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);

    modal.classList.add('active');
  },

  // 打开视频预览 - 全屏播放
  openVideoPreview(path, fileName) {
    const videoUrl = `/${path}`;
    const modal = document.createElement('div');
    modal.className = 'fm-modal preview-modal';
    modal.innerHTML = `
      <div class="fm-modal-content preview-content video-preview">
        <div class="fm-modal-header">
          <h3>${fileName}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          <video controls autoplay class="preview-video">
            <source src="${videoUrl}" type="video/${path.split('.').pop()}">
            您的浏览器不支持视频播放
          </video>
        </div>
      </div>
    `;
    document.body.appendChild(modal);

    // 关闭事件
    const closeBtn = modal.querySelector('.fm-modal-close');
    const closeModal = () => {
      const video = modal.querySelector('video');
      if (video) {
        video.pause();
        video.currentTime = 0;
      }
      document.body.removeChild(modal);
    };
    closeBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', e => {
      if (e.target === modal) closeModal();
    });

    // ESC 键关闭
    const escHandler = e => {
      if (e.key === 'Escape') {
        closeModal();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);

    // 添加淡入动画
    requestAnimationFrame(() => {
      modal.classList.add('active');
    });
  },

  // 打开文档预览
  async openDocumentPreview(path, fileName, extension) {
    const documentUrl = `/${path}`;
    const modal = document.createElement('div');
    modal.className = 'fm-modal preview-modal';

    let previewContent = '';
    let previewClass = 'document-preview';

    // 根据文件类型生成不同的预览内容
    switch (extension) {
      case 'pdf':
        previewContent = `
          <embed src="${documentUrl}" type="application/pdf" class="preview-embed" />
        `;
        previewClass = 'pdf-preview';
        break;
      case 'txt':
        previewContent = `
          <iframe src="${documentUrl}" class="preview-iframe"></iframe>
        `;
        previewClass = 'txt-preview';
        break;
      case 'doc':
      case 'docx':
      case 'xls':
      case 'xlsx':
      case 'ppt':
      case 'pptx':
        // Office 文档使用 Google Docs Viewer
        previewContent = `
          <iframe src="https://docs.google.com/viewer?url=${encodeURIComponent(window.location.origin + '/' + path)}&embedded=true" class="preview-iframe"></iframe>
        `;
        previewClass = 'office-preview';
        break;
      default:
        previewContent = `
          <div class="preview-placeholder">
            <div class="placeholder-icon"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg></div>
            <p>此文件类型暂不支持在线预览</p>
            <button class="fm-btn fm-btn-primary" onclick="FileManager.downloadFile('${path}')">下载文件</button>
          </div>
        `;
    }

    modal.innerHTML = `
      <div class="fm-modal-content preview-content ${previewClass}">
        <div class="fm-modal-header">
          <h3>${fileName}</h3>
          <button class="fm-modal-close">&times;</button>
        </div>
        <div class="fm-modal-body preview-body">
          ${previewContent}
        </div>
        <div class="fm-modal-footer">
          <button class="fm-btn fm-btn-secondary fm-modal-close-btn">关闭</button>
          <button class="fm-btn fm-btn-primary" onclick="FileManager.downloadFile('${path}')">下载文件</button>
        </div>
      </div>
    `;
    document.body.appendChild(modal);

    // 关闭事件
    const closeBtn = modal.querySelector('.fm-modal-close');
    const closeModal = () => {
      document.body.removeChild(modal);
    };
    closeBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', e => {
      if (e.target === modal) closeModal();
    });

    // 绑定关闭按钮
    modal.querySelectorAll('.fm-modal-close-btn').forEach(btn => {
      btn.addEventListener('click', closeModal);
    });

    // ESC 键关闭
    const escHandler = e => {
      if (e.key === 'Escape') {
        closeModal();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);

    modal.classList.add('active');
  },

  // 预览 Markdown 文件
  async previewMarkdownFile(path) {
    // 确保 markdown-preview-modal 已加载
    if (!window.MarkdownPreviewModal) {
      // 动态加载 markdown-preview-modal.js
      const script = document.createElement('script');
      script.src = '/js/markdown-preview-modal.js';
      script.onload = () => {
        this.openMarkdownPreview(path);
      };
      script.onerror = () => {
        this.showToast('预览组件加载失败', 'error');
      };
      document.head.appendChild(script);
    } else {
      this.openMarkdownPreview(path);
    }
  },

  // 打开 Markdown 预览
  openMarkdownPreview(path) {
    // 从路径中提取 markdown 文件的相对路径
    let markdownPath = path;

    // 处理绝对路径（如：/home/user/project/markdown/2026/02/19/test.md）
    if (markdownPath.startsWith('/')) {
      const markdownIndex = markdownPath.indexOf('/markdown/');
      if (markdownIndex !== -1) {
        markdownPath = markdownPath.substring(markdownIndex + 10); // 去掉 '/markdown/' 前缀
      } else {
        console.error('无效的 Markdown 路径:', path);
        this.showToast('无效的 Markdown 路径', 'error');
        return;
      }
    }
    // 处理相对路径（如：markdown/2026/02/19/test.md）
    else if (markdownPath.startsWith('markdown/')) {
      markdownPath = markdownPath.substring(9); // 去掉 'markdown/' 前缀
    } else {
      console.error('无效的 Markdown 路径:', path);
      this.showToast('无效的 Markdown 路径', 'error');
      return;
    }

    // 验证提取后的路径不为空且不是根路径
    if (!markdownPath || markdownPath === '/' || markdownPath.trim() === '') {
      console.error('提取后的 Markdown 路径无效:', markdownPath);
      this.showToast('无效的 Markdown 路径', 'error');
      return;
    }

    console.log('Markdown 预览路径:', markdownPath);

    if (window.MarkdownPreviewModal) {
      window.MarkdownPreviewModal.open(markdownPath);
    } else {
      this.showToast('预览功能不可用', 'error');
    }
  },

  // 下载文件
  async downloadFile(path) {
    try {
      const response = await fetch(`/api/files/download?path=${encodeURIComponent(path)}`, {
        headers: {
          Authorization: this.getAuthHeader(),
        },
      });

      if (!response.ok) {
        const result = await response.json();
        this.showToast(result.message || '下载失败', 'error');
        return;
      }

      // 获取文件名
      const fileName = path.split('/').pop();

      // 创建 blob 并下载
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('下载失败:', error);
      this.showToast('下载失败', 'error');
    }
  },

  // 显示上下文菜单
  showContextMenu(event, path, isDir) {
    this.selectedFile = { path, isDir };

    const menu = document.getElementById('contextMenu');
    menu.style.left = event.pageX + 'px';
    menu.style.top = event.pageY + 'px';
    menu.classList.add('active');

    // 检查是否是 markdown 文件
    const isMarkdown = path.toLowerCase().endsWith('.md');

    // 根据文件类型显示不同的菜单项
    const items = menu.querySelectorAll('.context-menu-item');
    items.forEach(item => {
      const action = item.dataset.action;
      if (action === 'download' && isDir) {
        item.style.display = 'none';
      } else if (action === 'preview') {
        // 预览选项只在 markdown 文件时显示
        item.style.display = isMarkdown && !isDir ? 'flex' : 'none';
      } else {
        item.style.display = 'flex';
      }
    });

    // 绑定菜单项点击事件
    items.forEach(item => {
      item.onclick = () => {
        this.handleContextAction(item.dataset.action);
        this.hideContextMenu();
      };
    });
  },

  // 隐藏上下文菜单
  hideContextMenu() {
    document.getElementById('contextMenu').classList.remove('active');
  },

  // 处理上下文菜单操作
  handleContextAction(action) {
    if (!this.selectedFile) return;

    switch (action) {
      case 'open':
        if (this.selectedFile.isDir) {
          this.navigateTo(this.selectedFile.path);
        } else {
          this.openFile(this.selectedFile.path);
        }
        break;
      case 'preview':
        this.previewMarkdownFile(this.selectedFile.path);
        break;
      case 'download':
        this.downloadFile(this.selectedFile.path);
        break;
      case 'rename':
        this.openRenameModal();
        break;
      case 'delete':
        this.openDeleteModal();
        break;
    }
  },

  // 打开上传模态框
  openUploadModal() {
    this.filesToUpload = [];
    this.updateUploadList();
    const modal = document.getElementById('uploadModal');
    modal.classList.add('active');
  },

  // 处理文件选择
  handleFileSelect(event) {
    const files = Array.from(event.target.files);
    this.addFilesToUpload(files);
    event.target.value = '';
  },

  // 处理文件拖放
  handleFileDrop(event) {
    const files = Array.from(event.dataTransfer.files);
    this.addFilesToUpload(files);
  },

  // 添加文件到上传列表
  addFilesToUpload(files) {
    files.forEach(file => {
      if (!this.filesToUpload.find(f => f.name === file.name)) {
        this.filesToUpload.push(file);
      }
    });
    this.updateUploadList();
  },

  // 更新上传列表
  updateUploadList() {
    const list = document.getElementById('uploadList');
    const confirmBtn = document.getElementById('confirmUploadBtn');

    if (this.filesToUpload.length === 0) {
      list.innerHTML = '';
      confirmBtn.disabled = true;
      return;
    }

    confirmBtn.disabled = false;
    list.innerHTML = this.filesToUpload
      .map(
        (file, index) => `
      <div class="upload-item">
        <div class="upload-item-name">${file.name}</div>
        <div class="upload-item-size">${this.formatFileSize(file.size)}</div>
        <button class="upload-item-remove" onclick="FileManager.removeFileFromUpload(${index})">✕</button>
      </div>
    `
      )
      .join('');
  },

  // 从上传列表移除文件
  removeFileFromUpload(index) {
    this.filesToUpload.splice(index, 1);
    this.updateUploadList();
  },

  // 上传文件
  async uploadFiles() {
    if (this.filesToUpload.length === 0) return;

    const confirmBtn = document.getElementById('confirmUploadBtn');
    confirmBtn.disabled = true;
    confirmBtn.textContent = '上传中...';

    let successCount = 0;
    let failCount = 0;

    for (const file of this.filesToUpload) {
      try {
        const formData = new FormData();
        formData.append('file', file);

        const response = await fetch(`/api/files?path=${encodeURIComponent(this.currentPath)}`, {
          method: 'POST',
          headers: {
            Authorization: this.getAuthHeader(),
          },
          body: formData,
        });

        const result = await response.json();

        if (result.success) {
          successCount++;
        } else {
          failCount++;
          console.error('上传失败:', result.message);
        }
      } catch (error) {
        console.error('上传失败:', error);
        failCount++;
      }
    }

    this.closeModal(document.getElementById('uploadModal'));
    this.loadFiles();

    if (successCount > 0 && failCount === 0) {
      this.showToast(`成功上传 ${successCount} 个文件`, 'success');
    } else if (successCount > 0) {
      this.showToast(`成功上传 ${successCount} 个文件，失败 ${failCount} 个`, 'warning');
    } else {
      this.showToast('上传失败', 'error');
    }

    confirmBtn.disabled = false;
    confirmBtn.textContent = '上传';
  },

  // 打开创建目录模态框
  openCreateDirModal() {
    document.getElementById('dirNameInput').value = '';
    const modal = document.getElementById('createDirModal');
    modal.classList.add('active');
    setTimeout(() => {
      document.getElementById('dirNameInput').focus();
    }, 100);
  },

  // 创建目录
  async createDirectory() {
    const dirName = document.getElementById('dirNameInput').value.trim();

    if (!dirName) {
      this.showToast('请输入文件夹名称', 'warning');
      return;
    }

    try {
      const response = await fetch('/api/files/create-dir', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: this.getAuthHeader(),
        },
        body: JSON.stringify({
          path: this.currentPath,
          dir_name: dirName,
        }),
      });

      const result = await response.json();

      if (result.success) {
        this.showToast('文件夹创建成功', 'success');
        this.closeModal(document.getElementById('createDirModal'));
        this.loadFiles();
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('创建目录失败:', error);
      this.showToast('创建文件夹失败', 'error');
    }
  },

  // 打开重命名模态框
  openRenameModal() {
    if (!this.selectedFile) return;

    const oldName = this.selectedFile.path.split('/').pop();
    document.getElementById('renameInput').value = oldName;
    const modal = document.getElementById('renameModal');
    modal.classList.add('active');
    setTimeout(() => {
      document.getElementById('renameInput').focus();
      document.getElementById('renameInput').select();
    }, 100);
  },

  // 重命名文件
  async renameFile() {
    if (!this.selectedFile) return;

    const newName = document.getElementById('renameInput').value.trim();

    if (!newName) {
      this.showToast('请输入新名称', 'warning');
      return;
    }

    try {
      const response = await fetch('/api/files', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          Authorization: this.getAuthHeader(),
        },
        body: JSON.stringify({
          old_path: this.selectedFile.path,
          new_name: newName,
        }),
      });

      const result = await response.json();

      if (result.success) {
        this.showToast('重命名成功', 'success');
        this.closeModal(document.getElementById('renameModal'));
        this.loadFiles();
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('重命名失败:', error);
      this.showToast('重命名失败', 'error');
    }
  },

  // 打开删除确认模态框
  openDeleteModal() {
    if (!this.selectedFile) return;

    const fileName = this.selectedFile.path.split('/').pop();
    document.getElementById('deleteFileName').textContent = fileName;
    const modal = document.getElementById('deleteModal');
    modal.classList.add('active');
  },

  // 删除文件
  async deleteFile() {
    if (!this.selectedFile) return;

    try {
      const response = await fetch(
        `/api/files?path=${encodeURIComponent(this.selectedFile.path)}`,
        {
          method: 'DELETE',
          headers: {
            Authorization: this.getAuthHeader(),
          },
        }
      );

      const result = await response.json();

      if (result.success) {
        this.showToast('删除成功', 'success');
        this.closeModal(document.getElementById('deleteModal'));
        this.loadFiles();
      } else {
        this.showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('删除失败:', error);
      this.showToast('删除失败', 'error');
    }
  },

  // 关闭模态框
  closeModal(modal) {
    modal.classList.remove('active');
    modal.classList.add('closing');
    setTimeout(() => {
      modal.classList.remove('closing');
    }, 300);
  },

  // 显示Toast通知
  showToast(message, type = 'success') {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.className = `toast ${type} active`;

    setTimeout(() => {
      toast.classList.remove('active');
    }, 3000);
  },
};

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', () => {
  FileManager.init();
});
