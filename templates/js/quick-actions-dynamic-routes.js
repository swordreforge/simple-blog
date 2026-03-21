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
    </svg>`
  };

  // 根据路由路径或名称选择合适的图标
  function selectIcon(route) {
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
      
      // 过滤出静态模板类型且已启用的路由
      const staticRoutes = routes.filter(route => 
        route.handler_type === 'static' && 
        route.enabled === true &&
        route.path && 
        route.path.length > 0
      );

      if (staticRoutes.length === 0) {
        console.log('没有找到静态模板类型的动态路由');
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

      // 添加动态路由快捷操作项
      staticRoutes.forEach(route => {
        const displayName = route.route_name || route.path;
        const icon = selectIcon(route);
        
        const actionItem = document.createElement('a');
        actionItem.className = 'quick-action-item dynamic-route-item';
        actionItem.href = route.path;
        actionItem.title = displayName;
        actionItem.setAttribute('data-route-id', route.id || '');
        
        actionItem.innerHTML = `
          ${icon}
          <span>${escapeHtml(displayName)}</span>
        `;

        quickActionsContent.appendChild(actionItem);
      });

      console.log(`已添加 ${staticRoutes.length} 个动态路由到快捷操作栏`);

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