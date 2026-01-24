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
    document.getElementById('createDirBtn').addEventListener('click', () => this.openCreateDirModal());
    
    // 根目录切换
    document.querySelectorAll('.fm-root-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const path = e.currentTarget.dataset.path;
        this.switchRoot(path);
      });
    });
    
    // 模态框关闭
    document.querySelectorAll('.modal-close, .fm-modal-close-btn, .fm-modal-close').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const modal = e.target.closest('.modal') || e.target.closest('.fm-modal');
        if (modal) {
          this.closeModal(modal);
        }
      });
    });
    
    // 上传区域
    const uploadArea = document.getElementById('uploadArea');
    const fileInput = document.getElementById('fileInput');
    
    uploadArea.addEventListener('click', () => fileInput.click());
    fileInput.addEventListener('change', (e) => this.handleFileSelect(e));
    
    uploadArea.addEventListener('dragover', (e) => {
      e.preventDefault();
      uploadArea.classList.add('dragover');
    });
    
    uploadArea.addEventListener('dragleave', () => {
      uploadArea.classList.remove('dragover');
    });
    
    uploadArea.addEventListener('drop', (e) => {
      e.preventDefault();
      uploadArea.classList.remove('dragover');
      this.handleFileDrop(e);
    });
    
    // 确认上传
    document.getElementById('confirmUploadBtn').addEventListener('click', () => this.uploadFiles());
    
    // 确认创建目录
    document.getElementById('confirmCreateDirBtn').addEventListener('click', () => this.createDirectory());
    
    // 确认重命名
    document.getElementById('confirmRenameBtn').addEventListener('click', () => this.renameFile());
    
    // 确认删除
    document.getElementById('confirmDeleteBtn').addEventListener('click', () => this.deleteFile());
    
    // 点击外部关闭上下文菜单
    document.addEventListener('click', (e) => {
      if (!e.target.closest('.context-menu') && !e.target.closest('.file-item')) {
        this.hideContextMenu();
      }
    });
    
    // 键盘事件
    document.addEventListener('keydown', (e) => {
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
          'Authorization': this.getAuthHeader()
        }
      });
      const result = await response.json();

      if (result.success) {
        this.renderFiles(result.data.files);
        this.updateBreadcrumb(result.data.currentPath);
        this.updateBackButton(result.data.parentPath);
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
          'Authorization': this.getAuthHeader()
        }
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
      item.addEventListener('click', (e) => {
        e.stopPropagation();
        const id = item.dataset.id;
        this.showAttachmentMenu(id, e);
      });
    });
  },

  // 创建附件项
  createAttachmentItem(attachment) {
    const visibilityIcon = {
      'public': '🌐',
      'private': '🔒',
      'protected': '🛡️'
    }[attachment.visibility] || '🌐';

    const visibilityLabel = {
      'public': '公开',
      'private': '私密',
      'protected': '受保护'
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
        <span class="context-menu-icon">${attachment.visibility === 'public' ? '🔒' : '🌐'}</span>
      </div>
      <div class="context-menu-item" data-action="toggle-show">
        <span>${attachment.show_in_passage ? '在文章中隐藏' : '在文章中显示'}</span>
        <span class="context-menu-icon">${attachment.show_in_passage ? '👁️‍🗨️' : '👁️'}</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item context-menu-danger" data-action="delete">
        <span>删除附件</span>
        <span class="context-menu-icon">🗑️</span>
      </div>
    `;

    document.body.appendChild(menu);

    // 绑定菜单项事件
    menu.querySelectorAll('.context-menu-item').forEach(item => {
      item.addEventListener('click', (e) => {
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
      const response = await fetch(`/api/admin/attachments?id=${id}`, {
        method: 'PATCH',
        headers: {
          'Authorization': this.getAuthHeader(),
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          visibility: newVisibility
        })
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
          'Authorization': this.getAuthHeader(),
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          show_in_passage: !attachment.show_in_passage
        })
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
      const response = await fetch(`/api/admin/attachments?id=${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': this.getAuthHeader()
        }
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
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  },

  // 获取文件图标
  getFileIcon(fileType) {
    const icons = {
      'image': '🖼️',
      'video': '🎬',
      'audio': '🎵',
      'document': '📄',
      'archive': '📦'
    };
    return icons[fileType] || '📁';
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
      item.addEventListener('click', (e) => {
        e.stopPropagation();
        const path = item.dataset.path;
        const isDir = item.dataset.isDir === 'true';
        
        if (isDir) {
          this.navigateTo(path);
        } else {
          this.openFile(path);
        }
      });
      
      item.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        const path = item.dataset.path;
        const isDir = item.dataset.isDir === 'true';
        this.showContextMenu(e, path, isDir);
      });
    });
  },
  
  // 创建文件项HTML
  createFileItem(file) {
    let icon = '📄';
    let typeClass = '';
    
    if (file.is_dir) {
      icon = '📁';
      typeClass = 'directory';
    } else if (['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.svg', '.ico', '.tiff', '.tif'].includes(file.extension)) {
      icon = `<img src="/${file.path}/${file.name}" alt="${file.name}" onerror="this.parentElement.innerHTML='🖼️'">`;
      typeClass = 'image';
    } else if (['.mp3', '.flac', '.wav', '.ogg', '.m4a', '.aac', '.wma', '.opus', '.ape'].includes(file.extension)) {
      icon = '🎵';
      typeClass = 'audio';
    } else if (['.mp4', '.webm', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.m4v', '.3gp'].includes(file.extension)) {
      icon = '🎬';
      typeClass = 'video';
    } else if (['.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx', '.txt'].includes(file.extension)) {
      icon = '📄';
      typeClass = 'document';
    } else if (file.extension === '.md') {
      icon = '📝';
      typeClass = 'markdown';
    }
    
    const size = this.formatFileSize(file.size);
    const fullPath = `${file.path}/${file.name}`;
    
    return `
      <div class="file-item ${typeClass}" data-path="${fullPath}" data-is-dir="${file.is_dir}">
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

    // 图片文件 - 在线预览
    if (['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.svg', '.ico', '.tiff', '.tif'].includes('.' + extension)) {
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
    if (['.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx', '.txt'].includes('.' + extension)) {
      this.openDocumentPreview(path, fileName, extension);
      return;
    }

    // 其他文件 - 直接下载
    try {
      const response = await fetch(`/api/files/download?path=${encodeURIComponent(path)}`, {
        headers: {
          'Authorization': this.getAuthHeader()
        }
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
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    // ESC 键关闭
    const escHandler = (e) => {
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
          <div class="audio-icon">🎵</div>
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
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    // ESC 键关闭
    const escHandler = (e) => {
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
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    // ESC 键关闭
    const escHandler = (e) => {
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
            <div class="placeholder-icon">📄</div>
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
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    // 绑定关闭按钮
    modal.querySelectorAll('.fm-modal-close-btn').forEach(btn => {
      btn.addEventListener('click', closeModal);
    });

    // ESC 键关闭
    const escHandler = (e) => {
      if (e.key === 'Escape') {
        closeModal();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);

    modal.classList.add('active');
  },

  // 下载文件
  async downloadFile(path) {
    try {
      const response = await fetch(`/api/files/download?path=${encodeURIComponent(path)}`, {
        headers: {
          'Authorization': this.getAuthHeader()
        }
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
    
    // 根据文件类型显示不同的菜单项
    const items = menu.querySelectorAll('.context-menu-item');
    items.forEach(item => {
      const action = item.dataset.action;
      if (action === 'download' && isDir) {
        item.style.display = 'none';
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
    list.innerHTML = this.filesToUpload.map((file, index) => `
      <div class="upload-item">
        <div class="upload-item-name">${file.name}</div>
        <div class="upload-item-size">${this.formatFileSize(file.size)}</div>
        <button class="upload-item-remove" onclick="FileManager.removeFileFromUpload(${index})">✕</button>
      </div>
    `).join('');
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
            'Authorization': this.getAuthHeader()
          },
          body: formData
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
          'Authorization': this.getAuthHeader()
        },
        body: JSON.stringify({
          path: this.currentPath,
          dir_name: dirName
        })
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
          'Authorization': this.getAuthHeader()
        },
        body: JSON.stringify({
          old_path: this.selectedFile.path,
          new_name: newName
        })
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
      const response = await fetch(`/api/files?path=${encodeURIComponent(this.selectedFile.path)}`, {
        method: 'DELETE',
        headers: {
          'Authorization': this.getAuthHeader()
        }
      });
      
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
  }
};

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', () => {
  FileManager.init();
});