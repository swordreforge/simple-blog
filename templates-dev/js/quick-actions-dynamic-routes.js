!(function () {
  'use strict';
  const e = {
    default:
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <circle cx="12" cy="12" r="10"></circle>\n      <line x1="12" y1="16" x2="12" y2="12"></line>\n      <line x1="12" y1="8" x2="12.01" y2="8"></line>\n    </svg>',
    page: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>\n      <polyline points="14 2 14 8 20 8"></polyline>\n      <line x1="16" y1="13" x2="8" y2="13"></line>\n      <line x1="16" y1="17" x2="8" y2="17"></line>\n      <polyline points="10 9 9 9 8 9"></polyline>\n    </svg>',
    link: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>\n      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>\n    </svg>',
    star: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>\n    </svg>',
    user: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>\n      <circle cx="12" cy="7" r="4"></circle>\n    </svg>',
    settings:
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <circle cx="12" cy="12" r="3"></circle>\n      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>\n    </svg>',
    folder:
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>\n    </svg>',
    home: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>\n      <polyline points="9 22 9 12 15 12 15 22"></polyline>\n    </svg>',
    archive:
      '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">\n      <polyline points="21 8 21 21 3 21 3 8"></polyline>\n      <rect x="1" y="3" width="22" height="5"></rect>\n      <line x1="10" y1="12" x2="14" y2="12"></line>\n    </svg>',
  };
  async function t() {
    const t = document.querySelector('.quick-actions-content');
    if (t)
      try {
        var o,
          i,
          r,
          l = await fetch('/api/routes/public?page=1&limit=100&handler_type=static');
        l.ok
          ? (o = await l.json()).success && o.data && o.data.routes
            ? 0 ===
              (i = o.data.routes.filter(
                e =>
                  !(!e.enabled || 'static' !== e.handler_type) &&
                  (e.group_id
                    ? !0 === e.is_primary_entry
                    : !1 !== (e.metadata || {}).show_in_quick_menu)
              )).length
              ? console.log('没有找到主要入口路由')
              : (0 < t.querySelectorAll('.quick-action-item').length &&
                  (((r = document.createElement('div')).className = 'quick-action-divider'),
                  (r.style.cssText =
                    'height: 1px; background: rgba(255,255,255,0.1); margin: 8px 0;'),
                  t.appendChild(r)),
                i.forEach(o => {
                  var i = o.route_name || o.path,
                    r = (function (t) {
                      if (t.metadata && t.metadata.menu_icon) {
                        var n = t.metadata.menu_icon;
                        if (e[n]) return e[n];
                      }
                      return (
                        (n = t.path.toLowerCase()),
                        (t = (t.route_name || '').toLowerCase()),
                        n.includes('page') || t.includes('page')
                          ? e.page
                          : n.includes('link') || t.includes('link')
                            ? e.link
                            : n.includes('star') || t.includes('star') || n.includes('favorite')
                              ? e.star
                              : e.default
                      );
                    })(o),
                    l = o.metadata?.group_name,
                    a = document.createElement('a');
                  ((a.className = 'quick-action-item dynamic-route-item primary-entry'),
                    (a.href = o.path),
                    (a.title = l ? i + ' - ' + l : i),
                    a.setAttribute('data-route-id', o.id || ''),
                    a.setAttribute('data-group-id', o.group_id || ''),
                    (a.innerHTML = `\n          ${r}\n          <span>${n(i)}</span>\n          ${l ? `<small style="display:block;font-size:0.75em;color:rgba(255,255,255,0.6);margin-top:2px;">${n(l)}</small>` : ''}\n        `),
                    t.appendChild(a));
                }),
                console.log(`已添加 ${i.length} 个主要入口路由到快捷操作栏`))
            : console.warn('动态路由数据格式错误')
          : console.warn('获取动态路由失败:', l.status);
      } catch (o) {
        console.error('加载动态路由到快捷操作栏失败:', o);
      }
    else console.log('未找到快捷操作栏容器，跳过动态路由加载');
  }
  function n(e) {
    var t = document.createElement('div');
    return ((t.textContent = e), t.innerHTML);
  }
  'loading' === document.readyState ? document.addEventListener('DOMContentLoaded', t) : t();
})();
