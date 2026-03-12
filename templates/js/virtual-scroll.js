/**
 * VirtualScroll Class
 * 实现虚拟滚动，优化大量数据的渲染性能
 */
class VirtualScroll {
  constructor(options = {}) {
    this.container = options.container;
    this.itemHeight = options.itemHeight || 40;
    this.bufferSize = options.bufferSize || 5;
    this.onRenderItem = options.onRenderItem;
    this.items = [];
    this.visibleStart = 0;
    this.visibleEnd = 0;
    this.totalHeight = 0;
    this.scrollTop = 0;

    // 创建滚动容器
    this.init();
  }

  init() {
    // 创建内容容器
    this.contentContainer = document.createElement('div');
    this.contentContainer.style.cssText = `
      position: relative;
      height: 100%;
      overflow: auto;
    `;

    // 创建虚拟内容容器（用于设置总高度）
    this.virtualContent = document.createElement('div');
    this.virtualContent.style.cssText = `
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      will-change: transform;
    `;

    this.contentContainer.appendChild(this.virtualContent);
    this.container.innerHTML = '';
    this.container.appendChild(this.contentContainer);

    // 监听滚动事件
    this.contentContainer.addEventListener('scroll', this.handleScroll.bind(this));

    // 监听窗口大小变化
    window.addEventListener('resize', this.handleResize.bind(this));
  }

  /**
   * 设置数据
   */
  setItems(items) {
    this.items = items;
    this.totalHeight = items.length * this.itemHeight;
    this.virtualContent.style.height = `${this.totalHeight}px`;
    this.render();
  }

  /**
   * 处理滚动事件
   */
  handleScroll() {
    this.scrollTop = this.contentContainer.scrollTop;
    this.render();
  }

  /**
   * 处理窗口大小变化
   */
  handleResize() {
    this.render();
  }

  /**
   * 渲染可见项目
   */
  render() {
    const containerHeight = this.contentContainer.clientHeight;

    // 计算可见范围
    const startIndex = Math.max(0, Math.floor(this.scrollTop / this.itemHeight) - this.bufferSize);
    const endIndex = Math.min(
      this.items.length,
      Math.ceil((this.scrollTop + containerHeight) / this.itemHeight) + this.bufferSize
    );

    // 如果范围没有变化，不重新渲染
    if (startIndex === this.visibleStart && endIndex === this.visibleEnd) {
      return;
    }

    this.visibleStart = startIndex;
    this.visibleEnd = endIndex;

    // 清空虚拟内容
    this.virtualContent.innerHTML = '';

    // 渲染可见项目
    for (let i = startIndex; i < endIndex; i++) {
      const item = this.items[i];
      const offsetY = i * this.itemHeight;

      const itemElement = document.createElement('div');
      itemElement.style.cssText = `
        position: absolute;
        top: ${offsetY}px;
        left: 0;
        right: 0;
        height: ${this.itemHeight}px;
        will-change: transform;
      `;

      if (this.onRenderItem) {
        this.onRenderItem(itemElement, item, i);
      }

      this.virtualContent.appendChild(itemElement);
    }
  }

  /**
   * 滚动到指定项目
   */
  scrollToItem(index, behavior = 'smooth') {
    if (index < 0 || index >= this.items.length) return;

    const targetScrollTop = index * this.itemHeight;
    this.contentContainer.scrollTo({
      top: targetScrollTop,
      behavior: behavior,
    });
  }

  /**
   * 滚动到顶部
   */
  scrollToTop(behavior = 'smooth') {
    this.contentContainer.scrollTo({
      top: 0,
      behavior: behavior,
    });
  }

  /**
   * 获取当前滚动位置
   */
  getScrollPosition() {
    return this.scrollTop;
  }

  /**
   * 销毁虚拟滚动
   */
  destroy() {
    this.contentContainer.removeEventListener('scroll', this.handleScroll.bind(this));
    window.removeEventListener('resize', this.handleResize.bind(this));
    this.container.innerHTML = '';
  }
}

/**
 * SidebarVirtualScroll
 * 专门为文件树侧边栏优化的虚拟滚动
 */
class SidebarVirtualScroll extends VirtualScroll {
  constructor(options = {}) {
    super({
      ...options,
      itemHeight: options.itemHeight || 42, // 默认项目高度 42px
    });

    this.folders = options.folders || [];
    this.onFolderToggle = options.onFolderToggle;
    this.onFileClick = options.onFileClick;
    this.flattenCache = new Map();
  }

  /**
   * 设置文件夹数据
   */
  setFolders(folders) {
    this.folders = folders;
    this.flattenCache.clear();
    this.updateFlattenedItems();
  }

  /**
   * 更新扁平化的项目列表
   */
  updateFlattenedItems() {
    const flattened = this.flattenFolders(this.folders);
    this.setItems(flattened);
  }

  /**
   * 递归扁平化文件夹
   */
  flattenFolders(folders, level = 0, parentOpen = true) {
    const flattened = [];

    folders.forEach(folder => {
      // 添加文件夹项
      flattened.push({
        type: 'folder',
        id: folder.id,
        name: folder.name,
        level: level,
        open: folder.open,
        parentId: folder.parentId,
        originalFolder: folder,
        fileCount: this.countFilesInFolder(folder),
      });

      // 如果文件夹展开，添加其内容
      if (folder.open && parentOpen) {
        // 添加子文件夹
        if (folder.subfolders && folder.subfolders.length > 0) {
          flattened.push(...this.flattenFolders(folder.subfolders, level + 1, true));
        }

        // 添加文件
        if (folder.files && folder.files.length > 0) {
          folder.files.forEach(file => {
            flattened.push({
              type: 'file',
              id: file.id,
              name: file.title,
              level: level + 1,
              file: file,
              parentId: folder.id,
            });
          });
        }
      }
    });

    return flattened;
  }

  /**
   * 计算文件夹中的文件数量
   */
  countFilesInFolder(folder) {
    let count = folder.files ? folder.files.length : 0;
    if (folder.subfolders) {
      folder.subfolders.forEach(subfolder => {
        count += this.countFilesInFolder(subfolder);
      });
    }
    return count;
  }

  /**
   * 切换文件夹展开/折叠
   */
  toggleFolder(folderId) {
    const folder = this.findFolderById(this.folders, folderId);
    if (folder) {
      folder.open = !folder.open;
      this.flattenCache.clear();
      this.updateFlattenedItems();

      if (this.onFolderToggle) {
        this.onFolderToggle(folderId, folder.open);
      }
    }
  }

  /**
   * 递归查找文件夹
   */
  findFolderById(folders, id) {
    for (const folder of folders) {
      if (folder.id === id) {
        return folder;
      }
      if (folder.subfolders) {
        const found = this.findFolderById(folder.subfolders, id);
        if (found) return found;
      }
    }
    return null;
  }

  /**
   * 展开所有文件夹
   */
  expandAll() {
    this.expandFoldersRecursive(this.folders);
    this.flattenCache.clear();
    this.updateFlattenedItems();
  }

  /**
   * 递归展开文件夹
   */
  expandFoldersRecursive(folders) {
    folders.forEach(folder => {
      folder.open = true;
      if (folder.subfolders) {
        this.expandFoldersRecursive(folder.subfolders);
      }
    });
  }

  /**
   * 折叠所有文件夹
   */
  collapseAll() {
    this.collapseFoldersRecursive(this.folders);
    this.flattenCache.clear();
    this.updateFlattenedItems();
  }

  /**
   * 递归折叠文件夹
   */
  collapseFoldersRecursive(folders) {
    folders.forEach(folder => {
      folder.open = false;
      if (folder.subfolders) {
        this.collapseFoldersRecursive(folder.subfolders);
      }
    });
  }

  /**
   * 渲染项目
   */
  renderItem(element, item, index) {
    if (item.type === 'folder') {
      this.renderFolderItem(element, item, index);
    } else if (item.type === 'file') {
      this.renderFileItem(element, item, index);
    }
  }

  /**
   * 渲染文件夹项
   */
  renderFolderItem(element, item, index) {
    const paddingLeft = 10 + item.level * 15;
    const iconRotation = item.open ? '90deg' : '0deg';
    const fileCountText = item.open ? item.fileCount : `${item.fileCount}+`;

    element.className = 'virtual-folder-item';
    element.style.cssText = `
      position: absolute;
      top: ${index * this.itemHeight}px;
      left: 0;
      right: 0;
      height: ${this.itemHeight}px;
      padding-left: ${paddingLeft}px;
      display: flex;
      align-items: center;
      cursor: pointer;
      transition: background-color 0.2s;
    `;

    element.innerHTML = `
      <span class="folder-icon" style="transform: rotate(${iconRotation}); transition: transform 0.3s;">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
        </svg>
      </span>
      <span class="folder-name" style="flex: 1; font-weight: 500; color: #f0f0f0; font-size: 0.9em;">${item.name}</span>
      <span class="file-count" style="background-color: rgba(255, 255, 255, 0.1); color: #aaa; font-size: 0.8em; padding: 2px 8px; border-radius: 10px;">${fileCountText}</span>
    `;

    // 添加悬停效果
    element.addEventListener('mouseenter', () => {
      element.style.backgroundColor = 'rgba(255, 255, 255, 0.05)';
    });
    element.addEventListener('mouseleave', () => {
      element.style.backgroundColor = 'transparent';
    });

    // 添加点击事件
    element.addEventListener('click', () => {
      this.toggleFolder(item.id);
    });
  }

  /**
   * 渲染文件项
   */
  renderFileItem(element, item, index) {
    const paddingLeft = 10 + item.level * 15;
    const isUnpublished = item.file.status !== 'published';

    element.className = 'virtual-file-item' + (isUnpublished ? ' file-unpublished' : '');
    element.style.cssText = `
      position: absolute;
      top: ${index * this.itemHeight}px;
      left: 0;
      right: 0;
      height: ${this.itemHeight}px;
      padding-left: ${paddingLeft}px;
      padding-right: 10px;
      display: flex;
      align-items: center;
      cursor: pointer;
      transition: all 0.2s;
      ${isUnpublished ? 'opacity: 0.7;' : ''}
    `;

    const icon = isUnpublished
      ? `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
        </svg>`
      : `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
          <polyline points="13 2 13 9 20 9"></polyline>
        </svg>`;

    const dateOnly = item.file.date ? item.file.date.split(' ')[0] : '';

    element.innerHTML = `
      <span class="file-icon" style="margin-right: 8px; color: ${isUnpublished ? '#ff9800' : '#007bff'};">${icon}</span>
      <span class="file-name" style="flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #ffffff; font-size: 0.95em; ${isUnpublished ? 'color: #ff9800; font-style: italic;' : ''}">${item.name}</span>
      <span class="file-date" style="font-size: 0.8em; color: #aaa; margin-left: 8px;">${dateOnly}</span>
    `;

    // 添加悬停效果
    element.addEventListener('mouseenter', () => {
      element.style.backgroundColor = 'rgba(255, 255, 255, 0.05)';
    });
    element.addEventListener('mouseleave', () => {
      element.style.backgroundColor = 'transparent';
    });

    // 添加点击事件
    element.addEventListener('click', () => {
      if (this.onFileClick) {
        this.onFileClick(item.file);
      }
    });

    // 添加双击事件（在新标签页打开）
    element.addEventListener('dblclick', () => {
      if (this.onFileClick) {
        this.onFileClick(item.file, true);
      }
    });
  }
}

// 导出类
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { VirtualScroll, SidebarVirtualScroll };
}
