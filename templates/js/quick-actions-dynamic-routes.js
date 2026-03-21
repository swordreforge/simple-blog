/**
 * 动态路由快捷操作集成
 * 自动将静态模板类型的动态路由添加到快捷操作栏中
 */

(function() {
  'use strict';

  // 图标映射 - 为不同类型的路由提供默认图标
  const iconMap = {
    default: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="12" y1="16" x2="12" y2="12"></line>
      <line x1="12" y1="8" x2="12.01" y2="8"></line>
    </svg>`,
    page: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
      <polyline points="14 2 14 8 20 8"></polyline>
      <line x1="16" y1="13" x2="8" y2="13"></line>
      <line x1="16" y1="17" x2="8" y2="17"></line>
      <polyline points="10 9 9 9 8 9"></polyline>
    </svg>`,
    link: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
    </svg>`,
    star: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
    </svg>`,
    user: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
      <circle cx="12" cy="7" r="4"></circle>
    </svg>`,
    settings: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3"></circle>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
    </svg>`,
    folder: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
    </svg>`,
    home: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
      <polyline points="9 22 9 12 15 12 15 22"></polyline>
    </svg>`,
    archive: `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="21 8 21 21 3 21 3 8"></polyline>
      <rect x="1" y="3" width="22" height="5"></rect>
      <line x1="10" y1="12" x2="14" y2="12"></line>
    </svg>`
  };

  // 根据路由路径或名称选择合适的图标
  function selectIcon(route) {
    // 优先使用 metadata 中的自定义图标
    if (route.metadata && route.metadata.menu_icon) {
      const iconType = route.metadata.menu_icon;
      if (iconMap[iconType]) {
        return iconMap[iconType];
      }
    }

    const path = route.path.toLowerCase();
    const name = (route.route_name || '').toLowerCase();

    if (path.includes('page') || name.includes('page')) {
      return iconMap.page;
    }
    if (path.includes('link') || name.includes('link')) {
      return iconMap.link;
    }
    if (path.includes('star') || name.includes('star') || path.includes('favorite')) {
      return iconMap.star;
    }

    return iconMap.default;
  }

  // 加载动态路由并添加到快捷操作栏
  async function loadDynamicRoutesToQuickActions() {
    const quickActionsContent = document.querySelector('.quick-actions-content');
    
    if (!quickActionsContent) {
      console.log('未找到快捷操作栏容器，跳过动态路由加载');
      return;
    }

    try {
      // 获取动态路由列表（使用公开 API，无需登录）
      const response = await fetch('/api/routes/public?page=1&limit=100&handler_type=static');
      
      if (!response.ok) {
        console.warn('获取动态路由失败:', response.status);
        return;
      }

      const result = await response.json();
      
      if (!result.success || !result.data || !result.data.routes) {
        console.warn('动态路由数据格式错误');
        return;
      }

      const routes = result.data.routes;
      
      // 过滤出主要入口的路由
      const primaryRoutes = routes.filter(route => {
        if (!route.enabled || route.handler_type !== 'static') {
          return false;
        }

        // 检查路由组信息（直接访问独立字段）
        const groupId = route.group_id;

        // 如果没有路由组配置，使用原有的 metadata.show_in_quick_menu 字段
        if (!groupId) {
          const metadata = route.metadata || {};
          // 如果 show_in_quick_menu 未设置，默认显示该路由
          // 如果设置为 false，则不显示
          return metadata.show_in_quick_menu !== false;
        }

        // 如果有路由组配置，只显示主要入口
        return route.is_primary_entry === true;
      });

      if (primaryRoutes.length === 0) {
        console.log('没有找到主要入口路由');
        return;
      }

      // 创建分隔符（如果已有快捷操作项）
      const existingItems = quickActionsContent.querySelectorAll('.quick-action-item');
      if (existingItems.length > 0) {
        const divider = document.createElement('div');
        divider.className = 'quick-action-divider';
        divider.style.cssText = 'height: 1px; background: rgba(255,255,255,0.1); margin: 8px 0;';
        quickActionsContent.appendChild(divider);
      }

      // 添加主要入口路由快捷操作项
      primaryRoutes.forEach(route => {
        const displayName = route.route_name || route.path;
        const icon = selectIcon(route);
        const groupName = route.metadata?.group_name;
        
        const actionItem = document.createElement('a');
        actionItem.className = 'quick-action-item dynamic-route-item primary-entry';
        actionItem.href = route.path;
        actionItem.title = groupName ? `${displayName} - ${groupName}` : displayName;
        actionItem.setAttribute('data-route-id', route.id || '');
        actionItem.setAttribute('data-group-id', route.group_id || '');
        
        actionItem.innerHTML = `
          ${icon}
          <span>${escapeHtml(displayName)}</span>
          ${groupName ? `<small style="display:block;font-size:0.75em;color:rgba(255,255,255,0.6);margin-top:2px;">${escapeHtml(groupName)}</small>` : ''}
        `;

        quickActionsContent.appendChild(actionItem);
      });

      console.log(`已添加 ${primaryRoutes.length} 个主要入口路由到快捷操作栏`);

    } catch (error) {
      console.error('加载动态路由到快捷操作栏失败:', error);
    }
  }

  // HTML 转义函数，防止 XSS
  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // 在 DOM 加载完成后执行
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', loadDynamicRoutesToQuickActions);
  } else {
    // DOM 已经加载完成
    loadDynamicRoutesToQuickActions();
  }

})();