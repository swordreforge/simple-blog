let currentPage = 1,
  pageSize = 20,
  totalRoutes = 0,
  routes = [];
async function loadRoutes() {
  document.getElementById('routesTableBody').innerHTML =
    '<tr><td colspan="9" class="loading">加载中...</td></tr>';
  try {
    let t = document.getElementById('filterType').value,
      a = document.getElementById('filterStatus').value,
      n = `/api/admin/dynamic-routes?page=${currentPage}&limit=` + pageSize;
    (t && (n += '&handler_type=' + t),
      'enabled' === a && (n += '&enabled=true'),
      'disabled' === a && (n += '&enabled=false'));
    var e = await (await fetch(n)).json();
    e.success
      ? ((routes = e.data.routes), (totalRoutes = e.data.total), renderRoutes(), renderPagination())
      : showMessage('error', e.message || '加载失败');
  } catch (e) {
    showMessage('error', '网络错误: ' + e.message);
  }
}
async function loadStats() {
  try {
    var e,
      t,
      a,
      n,
      o,
      s,
      r = await (await fetch('/api/admin/dynamic-routes?page=1&limit=1000')).json();
    (r.success &&
      ((t = (e = r.data.routes).filter(e => e.enabled).length),
      (a = e.filter(e => !e.enabled).length),
      (n = document.getElementById('totalRoutes')) && (n.textContent = e.length),
      (o = document.getElementById('enabledRoutes')) && (o.textContent = t),
      (s = document.getElementById('disabledRoutes'))) &&
      (s.textContent = a),
      loadStorageStats());
  } catch (e) {
    console.error('加载统计信息失败:', e);
  }
}
async function loadStorageStats() {
  try {
    var e,
      t,
      a,
      n = await (await fetch('/api/admin/dynamic-routes/storage/stats')).json();
    (n.database &&
      (e = document.getElementById('databaseRoutes')) &&
      (e.textContent = n.database.total_routes),
      n.memory &&
        (t = document.getElementById('memoryRoutes')) &&
        (t.textContent = n.memory.total_routes),
      n.file &&
        (a = document.getElementById('fileRoutes')) &&
        (a.textContent = n.file.total_routes));
  } catch (e) {
    console.error('加载存储统计失败:', e);
  }
}
function renderRoutes() {
  var e = document.getElementById('routesTableBody');
  const t = document.getElementById('searchInput').value.toLowerCase();
  var a = routes.filter(e => {
    var a = (e.route_name || '').toLowerCase(),
      n = e.path.toLowerCase();
    return ((e = e.handler_type.toLowerCase()), a.includes(t) || n.includes(t) || e.includes(t));
  });
  0 === a.length
    ? (e.innerHTML = '<tr><td colspan="9" class="empty-state">暂无路由数据</td></tr>')
    : (e.innerHTML = a
        .map((e, t) => {
          var a = e.route_name
              ? `<div style="font-weight: 500; margin-bottom: 2px;">${escapeHtml(e.route_name)}</div>`
              : '',
            n = getHandlerTypeLabel(e.handler_type),
            o = getStorageTypeLabel(e.route_type || 'database');
          return `\n        <tr>\n            <td>${t + 1}</td>\n            <td>${a}</td>\n            <td><span class="route-path">${escapeHtml(e.path)}</span></td>\n            <td><span class="badge badge-warning handler-type-display">${n}</span></td>\n            <td>\n                <span class="badge ${e.enabled ? 'badge-success' : 'badge-danger'}">\n                    ${e.enabled ? '已启用' : '已禁用'}\n                </span>\n            </td>\n            <td><span class="priority-display">${e.priority}</span></td>\n            <td><span class="badge badge-info storage-type-display">${o}</span></td>\n            <td class="actions">\n                <button class="btn btn-sm btn-primary" onclick="editRoute(${e.id})" title="编辑">编辑</button>\n                <button class="btn btn-sm ${e.enabled ? 'btn-secondary' : 'btn-success'}"\n                        onclick="toggleRoute(${e.id}, ${!e.enabled})"\n                        title="${e.enabled ? '禁用' : '启用'}">\n                    ${e.enabled ? '禁用' : '启用'}\n                </button>\n                <button class="btn btn-sm btn-danger" onclick="deleteRoute(${e.id})" title="删除">删除</button>\n            </td>\n        </tr>\n    `;
        })
        .join(''));
}
function getHandlerTypeLabel(e) {
  return { redirect: '重定向', static: '静态内容', proxy: '代理', custom: '自定义' }[e] || e;
}
function getStorageTypeLabel(e) {
  return { database: '数据库', memory: '内存', file: '文件' }[e] || e;
}
function renderPagination() {
  var e,
    t = Math.ceil(totalRoutes / pageSize),
    a = document.getElementById('pagination');
  t <= 1
    ? (a.innerHTML = '')
    : ((e = `<button class="btn btn-secondary" onclick="goToPage(${currentPage - 1})"\n                    ${1 === currentPage ? 'disabled' : ''}>上一页</button>`),
      (e =
        (e += `<span>第 ${currentPage} / ${t} 页</span>`) +
        `<button class="btn btn-secondary" onclick="goToPage(${currentPage + 1})"\n                    ${currentPage === t ? 'disabled' : ''}>下一页</button>`),
      (a.innerHTML = e));
}
function goToPage(e) {
  var t = Math.ceil(totalRoutes / pageSize);
  e < 1 || t < e || ((currentPage = e), loadRoutes());
}
function searchRoutes() {
  renderRoutes();
}
function filterRoutes() {
  ((currentPage = 1), loadRoutes());
}
function refreshRoutes() {
  ((currentPage = 1), loadRoutes(), loadStats());
}
function updateHandlerFields() {
  var e = document.getElementById('handlerType').value,
    t = document.getElementById('routeType').value,
    a = document.getElementById('contentTypeGroup'),
    n = document.getElementById('inlineTemplateGroup'),
    o = document.getElementById('templatePathGroup'),
    s = document.getElementById('uploadFileBtn');
  (a && (a.style.display = 'none'),
    n && (n.style.display = 'none'),
    o && (o.style.display = 'none'),
    s && (s.style.display = 'none'),
    'static' === e &&
      (a && (a.style.display = 'block'),
      'database' === t || 'memory' === t
        ? (n && (n.style.display = 'block'), s && (s.style.display = 'inline-block'))
        : 'file' === t && o && (o.style.display = 'block')),
    loadTemplate());
}
function updateRouteTypeFields() {
  var e = document.getElementById('routeType').value,
    t = document.getElementById('handlerType').value,
    a = document.getElementById('inlineTemplateGroup'),
    n = document.getElementById('templatePathGroup'),
    o = document.getElementById('uploadFileBtn');
  (a && (a.style.display = 'none'),
    n && (n.style.display = 'none'),
    o && (o.style.display = 'none'),
    'static' === t &&
      ('database' === e || 'memory' === e
        ? (a && (a.style.display = 'block'), o && (o.style.display = 'inline-block'))
        : 'file' === e && n && (n.style.display = 'block')));
}
function updateContentTypeTemplate() {
  var e = document.getElementById('contentType').value;
  'static' === document.getElementById('handlerType').value && e && loadTemplate();
}
function showAddModal() {
  var e, t;
  for ([e, t] of Object.entries({
    modalTitle: 'textContent',
    routeId: 'value',
    routeName: 'value',
    routeType: 'value',
    routePath: 'value',
    handlerType: 'value',
    handlerConfig: 'value',
    contentType: 'value',
    routePriority: 'value',
    routeEnabled: 'checked',
    modalMessage: 'innerHTML',
    contentTypeGroup: 'style.display',
    inlineTemplateGroup: 'style.display',
    templatePathGroup: 'style.display',
    routeModal: 'classList',
    groupId: 'value',
    isPrimaryEntry: 'checked',
    groupName: 'value',
  }))
    if (!document.getElementById(e))
      return (
        console.error(`元素 ${e} 不存在`),
        void alert(`页面加载错误：找不到元素 ${e}，请刷新页面重试`)
      );
  ((document.getElementById('modalTitle').textContent = '添加路由'),
    (document.getElementById('routeId').value = ''),
    (document.getElementById('routeName').value = ''),
    (document.getElementById('routeType').value = 'database'),
    (document.getElementById('routePath').value = ''),
    (document.getElementById('handlerType').value = ''),
    (document.getElementById('handlerConfig').value = ''),
    (document.getElementById('contentType').value = ''),
    (document.getElementById('inlineTemplate').value = ''),
    (document.getElementById('templatePath').value = ''),
    (document.getElementById('routeMetadata').value = ''),
    (document.getElementById('routePriority').value = '0'),
    (document.getElementById('routeEnabled').checked = !0),
    (document.getElementById('modalMessage').innerHTML = ''),
    (document.getElementById('contentTypeGroup').style.display = 'none'),
    (document.getElementById('inlineTemplateGroup').style.display = 'none'),
    (document.getElementById('templatePathGroup').style.display = 'none'),
    (document.getElementById('groupId').value = ''),
    (document.getElementById('isPrimaryEntry').checked = !1),
    (document.getElementById('groupName').value = ''));
  var a = document.getElementById('uploadFileBtn');
  (a && (a.style.display = 'none'), document.getElementById('routeModal').classList.add('active'));
}
function editRoute(e) {
  var t = routes.find(t => t.id === e);
  if (t) {
    for (const e of [
      'modalTitle',
      'routeId',
      'routeName',
      'routeType',
      'routePath',
      'handlerType',
      'contentType',
      'handlerConfig',
      'routePriority',
      'routeEnabled',
      'inlineTemplate',
      'templatePath',
      'routeMetadata',
      'modalMessage',
      'routeModal',
      'groupId',
      'isPrimaryEntry',
      'groupName',
      'groupSettings',
      'groupNameGroup',
    ])
      if (!document.getElementById(e))
        return (
          console.error(`元素 ${e} 不存在`),
          void alert(`页面加载错误：找不到元素 ${e}，请刷新页面重试`)
        );
    ((document.getElementById('modalTitle').textContent = '编辑路由'),
      (document.getElementById('routeId').value = t.id),
      (document.getElementById('routeName').value = t.route_name || ''),
      (document.getElementById('routeType').value = t.route_type || 'database'),
      (document.getElementById('routePath').value = t.path),
      (document.getElementById('handlerType').value = t.handler_type),
      (document.getElementById('contentType').value = t.content_type_hint || ''),
      (document.getElementById('handlerConfig').value = JSON.stringify(t.handler_config, null, 2)),
      (document.getElementById('routePriority').value = t.priority),
      (document.getElementById('routeEnabled').checked = t.enabled),
      (document.getElementById('inlineTemplate').value = t.inline_template || ''),
      (document.getElementById('templatePath').value = t.template_path || ''),
      (document.getElementById('routeMetadata').value = t.metadata
        ? JSON.stringify(t.metadata, null, 2)
        : ''),
      (document.getElementById('modalMessage').innerHTML = ''),
      updateHandlerFields(),
      document.getElementById('routeModal').classList.add('active'));
    var a = t.metadata || {};
    ((document.getElementById('groupId').value = t.group_id || a.group_id || ''),
      (document.getElementById('isPrimaryEntry').checked =
        t.is_primary_entry || a.is_primary_entry || !1),
      (document.getElementById('groupName').value = a.group_name || ''),
      document.getElementById('groupId').dispatchEvent(new Event('input')));
  }
}
function closeModal() {
  document.getElementById('routeModal').classList.remove('active');
}
document.addEventListener('DOMContentLoaded', function () {
  for (const e of [
    'routeForm',
    'message',
    'routesTableBody',
    'pagination',
    'totalRoutes',
    'enabledRoutes',
    'disabledRoutes',
    'searchInput',
    'filterType',
    'filterStatus',
    'databaseRoutes',
    'memoryRoutes',
    'fileRoutes',
  ])
    if (!document.getElementById(e))
      return (
        console.error('页面加载错误：找不到元素 ' + e),
        void alert(`页面加载错误：找不到元素 ${e}，请刷新页面重试`)
      );
  (loadRoutes(),
    loadStats(),
    document.getElementById('routeForm').addEventListener('submit', function (e) {
      (e.preventDefault(), saveRoute());
    }));
});
let isSubmitting = !1;
function containsControlChars(e, t = !1) {
  for (let n = 0; n < e.length; n++) {
    var a = e.charCodeAt(n);
    if ((a < 32 || 127 === a) && (!t || (9 !== a && 10 !== a && 13 !== a))) return !0;
  }
  return !1;
}
async function saveRoute() {
  if (isSubmitting) showMessage('modalError', '正在提交中，请稍候...');
  else {
    var e = document.getElementById('routeId').value,
      t = document.getElementById('routeName').value.trim(),
      a = document.getElementById('routeType').value,
      n = document.getElementById('routePath').value.trim(),
      o = document.getElementById('handlerType').value,
      s = document.getElementById('handlerConfig').value,
      r = document.getElementById('contentType').value,
      l = document.getElementById('inlineTemplate').value.trim(),
      d = document.getElementById('templatePath').value.trim(),
      u = document.getElementById('routeMetadata').value.trim(),
      m = parseInt(document.getElementById('routePriority').value),
      i = document.getElementById('routeEnabled').checked;
    if (t && containsControlChars(t)) showMessage('modalError', '路由名称不能包含控制字符');
    else if (n && containsControlChars(n)) showMessage('modalError', '路由路径不能包含控制字符');
    else if (d && containsControlChars(d)) showMessage('modalError', '模板路径不能包含控制字符');
    else if (u && containsControlChars(u, !0))
      showMessage('modalError', '扩展元数据不能包含非法控制字符');
    else if (n)
      if (a)
        if (o) {
          try {
            JSON.parse(s);
          } catch (e) {
            return void showMessage('modalError', '处理器配置必须是有效的JSON格式');
          }
          let p = null;
          if (u)
            try {
              p = JSON.parse(u);
            } catch (e) {
              return void showMessage('modalError', '扩展元数据必须是有效的JSON格式');
            }
          var c = document.getElementById('groupId').value.trim(),
            y = document.getElementById('isPrimaryEntry').checked,
            g = document.getElementById('groupName').value.trim();
          if (
            (c && ((p = p || {}).group_name = g || null),
            (u = JSON.parse(s)),
            'database' === a || 'memory' === a)
          ) {
            if (d) return void showMessage('modalError', a + ' 类型路由不支持 template_path 字段');
          } else if ('file' === a) {
            if (l)
              return void showMessage('modalError', 'file 类型路由不支持 inline_template 字段');
            if (!d) return void showMessage('modalError', 'file 类型路由必须提供 template_path');
          }
          ((s = {
            route_name: t || null,
            route_type: a || 'database',
            path: n,
            handler_type: o,
            handler_config: u,
            content_type_hint: r || null,
            inline_template: l || null,
            template_path: d || null,
            metadata: p,
            enabled: i,
            priority: m,
            group_id: c || null,
            is_primary_entry: y,
          }),
            (isSubmitting = !0),
            (t = document.querySelector('#routeForm .btn-primary')) &&
              ((t.disabled = !0), (t.textContent = '提交中...')));
          try {
            (g = await (
              await fetch(e ? '/api/admin/dynamic-routes/' + e : '/api/admin/dynamic-routes', {
                method: e ? 'PUT' : 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(s),
              })
            ).json()).success
              ? (showMessage('success', g.message || '保存成功'),
                closeModal(),
                loadRoutes(),
                loadStats())
              : showMessage('modalError', g.message || '保存失败');
          } catch (e) {
            showMessage('modalError', '网络错误: ' + e.message);
          } finally {
            ((isSubmitting = !1), t && ((t.disabled = !1), (t.textContent = '保存')));
          }
        } else showMessage('modalError', '请选择处理器类型');
      else showMessage('modalError', '请选择路由类型');
    else showMessage('modalError', '路由路径不能为空');
  }
}
async function testRoute() {
  var e = document.getElementById('routePath'),
    t = document.getElementById('routeType'),
    a = document.getElementById('handlerType'),
    n = document.getElementById('handlerConfig'),
    o = document.getElementById('inlineTemplate'),
    s = document.getElementById('templatePath'),
    r = document.getElementById('contentType'),
    l = document.getElementById('routeMetadata');
  if (e && t && a && n)
    if (((e = e.value.trim()), (t = t.value), (a = a.value), (n = n.value), e && t && a))
      try {
        var d = await (
          await fetch('/api/admin/dynamic-routes/test', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              route_type: t || 'database',
              path: e,
              handler_type: a,
              handler_config: JSON.parse(n),
              inline_template: (o && o.value.trim()) || null,
              template_path: (s && s.value.trim()) || null,
              content_type_hint: (r && r.value.trim()) || null,
              metadata: l && l.value.trim() ? JSON.parse(l.value.trim()) : null,
            }),
          })
        ).json();
        if (d.success) {
          let e = d.data,
            t = '路由测试成功';
          (e.conflict && (t += ' (路径冲突)'),
            e.response_preview &&
              (t += '\n响应预览: ' + JSON.stringify(e.response_preview, null, 2)),
            alert(t));
        } else alert('测试失败: ' + d.message);
      } catch (e) {
        alert('测试失败: ' + e.message);
      }
    else alert('请先填写路由路径、选择路由类型和处理器类型');
  else alert('缺少必要的表单元素');
}
async function toggleRoute(e, t) {
  t = t ? 'enable' : 'disable';
  try {
    var a = await (await fetch(`/api/admin/dynamic-routes/${e}/` + t, { method: 'POST' })).json();
    a.success
      ? (showMessage('success', a.message || '操作成功'), loadRoutes(), loadStats())
      : showMessage('error', a.message || '操作失败');
  } catch (e) {
    showMessage('error', '网络错误: ' + e.message);
  }
}
async function deleteRoute(e) {
  if (confirm('确定要删除这个路由吗？'))
    try {
      var t = await (await fetch('/api/admin/dynamic-routes/' + e, { method: 'DELETE' })).json();
      t.success
        ? (showMessage('success', t.message || '删除成功'), loadRoutes(), loadStats())
        : showMessage('error', t.message || '删除失败');
    } catch (e) {
      showMessage('error', '网络错误: ' + e.message);
    }
}
async function exportRoutes() {
  try {
    var e,
      t,
      a,
      n = await (await fetch('/api/admin/dynamic-routes/export')).json();
    n.success
      ? ((e = new Blob([JSON.stringify(n.data, null, 2)], { type: 'application/json' })),
        (t = URL.createObjectURL(e)),
        ((a = document.createElement('a')).href = t),
        (a.download = `routes-export-${new Date().toISOString().split('T')[0]}.json`),
        document.body.appendChild(a),
        a.click(),
        document.body.removeChild(a),
        URL.revokeObjectURL(t))
      : showMessage('error', n.message || '导出失败');
  } catch (e) {
    showMessage('error', '网络错误: ' + e.message);
  }
}
function showImportModal() {
  ((document.getElementById('importConfig').value = ''),
    (document.getElementById('importMessage').innerHTML = ''),
    document.getElementById('importModal').classList.add('active'));
}
function closeImportModal() {
  document.getElementById('importModal').classList.remove('active');
}
async function importRoutes() {
  var e = document.getElementById('importConfig').value.trim();
  if (e)
    try {
      var t,
        a = JSON.parse(e),
        n = await (
          await fetch('/api/admin/dynamic-routes/import', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(a),
          })
        ).json();
      n.success
        ? ((t = n.data),
          alert(`导入完成: 成功 ${t.imported}, 跳过 ${t.skipped}, 失败 ` + t.failed),
          closeImportModal(),
          loadRoutes(),
          loadStats())
        : showMessage('importError', n.message || '导入失败');
    } catch (e) {
      showMessage('importError', '配置格式错误: ' + e.message);
    }
  else showMessage('importError', '请输入配置内容');
}
function formatConfig() {
  var e = document.getElementById('handlerConfig');
  try {
    var t = JSON.parse(e.value);
    e.value = JSON.stringify(t, null, 2);
  } catch (e) {
    alert('JSON格式错误');
  }
}
function validateConfig() {
  var e = document.getElementById('handlerConfig');
  try {
    (JSON.parse(e.value), alert('配置格式正确'));
  } catch (e) {
    alert('JSON格式错误: ' + e.message);
  }
}
function formatMetadata() {
  var e = document.getElementById('routeMetadata');
  try {
    var t = JSON.parse(e.value);
    e.value = JSON.stringify(t, null, 2);
  } catch (e) {
    alert('JSON格式错误');
  }
}
function validateMetadata() {
  var e = document.getElementById('routeMetadata');
  try {
    (JSON.parse(e.value), alert('元数据格式正确'));
  } catch (e) {
    alert('JSON格式错误: ' + e.message);
  }
}
function loadTemplate() {
  var e = document.getElementById('handlerType'),
    t = document.getElementById('contentSource'),
    a = document.getElementById('contentType');
  if (e) {
    e = e.value;
    var n = (t && t.value, a ? a.value : '');
    let o = {};
    switch (e) {
      case 'redirect':
        o = { type: 'redirect', target: '/new-location', status_code: 302, preserve_query: !0 };
        break;
      case 'static':
        o = {
          type: 'static',
          content_type: n || 'text/html; charset=utf-8',
          headers: { 'Cache-Control': 'public, max-age=3600' },
        };
        break;
      case 'proxy':
        o = {
          type: 'proxy',
          target: 'http://backend-service:8080',
          timeout: 5e3,
          strip_prefix: !1,
        };
        break;
      case 'custom':
        o = {
          type: 'custom',
          script: 'lua',
          source: 'function handle(req) return {status=200, body="OK"} end',
        };
        break;
      default:
        return;
    }
    (t = document.getElementById('handlerConfig'))
      ? t.value.trim() || (t.value = JSON.stringify(o, null, 2))
      : console.error('loadTemplate: handlerConfig元素不存在');
  } else console.error('loadTemplate: handlerType元素不存在');
}
function handleFileUpload(e) {
  const t = e.target.files[0];
  var a;
  t &&
    (1048576 < t.size
      ? alert('文件大小不能超过1MB')
      : (((a = new FileReader()).onload = function (e) {
          e = e.target.result;
          var a = t.name;
          let n = 'text/plain; charset=utf-8';
          switch (a.split('.').pop().toLowerCase()) {
            case 'html':
            case 'htm':
              n = 'text/html; charset=utf-8';
              break;
            case 'svg':
              n = 'image/svg+xml; charset=utf-8';
              break;
            case 'xml':
              n = 'application/xml; charset=utf-8';
              break;
            case 'css':
              n = 'text/css; charset=utf-8';
              break;
            case 'js':
              n = 'application/javascript; charset=utf-8';
              break;
            case 'json':
              n = 'application/json; charset=utf-8';
              break;
            default:
              n = 'text/plain; charset=utf-8';
          }
          ((document.getElementById('handlerType').value = 'static'),
            (document.getElementById('contentType').value = n),
            updateHandlerFields(),
            (document.getElementById('inlineTemplate').value = e),
            (document.getElementById('handlerConfig').value = JSON.stringify(
              { type: 'static', headers: { 'Cache-Control': 'public, max-age=3600' } },
              null,
              2
            )),
            showMessage(
              'success',
              `文件 "${a}" 上传成功，已自动设置为静态内容处理器，内容已填充到内联模板字段`
            ));
        }),
        (a.onerror = function () {
          (alert('文件读取失败'), (e.target.value = ''));
        }),
        a.readAsText(t)),
    (e.target.value = ''));
}
function showMessage(e, t) {
  const a = document.getElementById(
    'modalError' === e ? 'modalMessage' : 'importError' === e ? 'importMessage' : 'message'
  );
  ((a.className = e.includes('error') ? 'error' : 'success'),
    (a.textContent = t),
    (a.style.display = 'block'),
    setTimeout(() => {
      a.style.display = 'none';
    }, 5e3));
}
function escapeHtml(e) {
  var t = document.createElement('div');
  return ((t.textContent = e), t.innerHTML);
}
function openStorageModal() {
  (document.getElementById('storageModal').classList.add('active'), refreshStorageStats());
}
function closeStorageModal() {
  (document.getElementById('storageModal').classList.remove('active'),
    (document.getElementById('storageMessage').style.display = 'none'));
}
async function refreshStorageStats() {
  try {
    var e = await (await fetch('/api/admin/dynamic-routes/storage/stats')).json();
    (e.database &&
      ((document.getElementById('storageDatabaseTotal').textContent = e.database.total_routes),
      (document.getElementById('storageDatabaseEnabled').textContent = e.database.enabled_routes),
      (document.getElementById('storageDatabaseDisabled').textContent = e.database.disabled_routes),
      (document.getElementById('storageDatabaseMemory').textContent =
        e.database.memory_usage_bytes)),
      e.memory &&
        ((document.getElementById('storageMemoryTotal').textContent = e.memory.total_routes),
        (document.getElementById('storageMemoryEnabled').textContent = e.memory.enabled_routes),
        (document.getElementById('storageMemoryDisabled').textContent = e.memory.disabled_routes),
        (document.getElementById('storageMemoryMemory').textContent = e.memory.memory_usage_bytes)),
      e.file &&
        ((document.getElementById('storageFileTotal').textContent = e.file.total_routes),
        (document.getElementById('storageFileEnabled').textContent = e.file.enabled_routes),
        (document.getElementById('storageFileDisabled').textContent = e.file.disabled_routes),
        (document.getElementById('storageFileMemory').textContent = e.file.memory_usage_bytes)),
      e.database &&
        (document.getElementById('databaseRoutes').textContent = e.database.total_routes),
      e.memory && (document.getElementById('memoryRoutes').textContent = e.memory.total_routes),
      e.file && (document.getElementById('fileRoutes').textContent = e.file.total_routes));
  } catch (e) {
    (console.error('加载存储统计失败:', e),
      showMessage('storageError', '加载存储统计失败: ' + e.message));
  }
}
async function batchMigrateRoutes() {
  var e = document.getElementById('migrateFrom').value,
    t = document.getElementById('migrateTo').value;
  if (e === t) showMessage('storageError', '源存储类型和目标存储类型不能相同');
  else if (confirm(`确定要将所有路由从 ${e} 迁移到 ${t} 吗？此操作不可逆。`))
    try {
      var a = await (
        await fetch('/api/admin/dynamic-routes/storage/batch-migrate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ source_type: e, target_type: t }),
        })
      ).json();
      a.success
        ? (showMessage('storageSuccess', a.message),
          refreshStorageStats(),
          loadRoutes(),
          loadStats())
        : showMessage('storageError', a.message || '迁移失败');
    } catch (e) {
      showMessage('storageError', '网络错误: ' + e.message);
    }
}
async function clearStorage() {
  var e = document.getElementById('clearStorageType').value;
  if (confirm(`确定要清空 ${e} 存储中的所有路由吗？此操作不可逆。`))
    try {
      var t = await (
        await fetch('/api/admin/dynamic-routes/storage/clear/' + e, { method: 'POST' })
      ).json();
      t.success
        ? (showMessage('storageSuccess', t.message),
          refreshStorageStats(),
          loadRoutes(),
          loadStats())
        : showMessage('storageError', t.message || '清空失败');
    } catch (e) {
      showMessage('storageError', '网络错误: ' + e.message);
    }
}
const originalShowMessage = showMessage;
showMessage = function (e, t) {
  if ('storageSuccess' === e || 'storageError' === e) {
    const a = document.getElementById('storageMessage');
    ((a.className = 'storageError' === e ? 'error' : 'success'),
      (a.textContent = t),
      (a.style.display = 'block'),
      setTimeout(() => {
        a.style.display = 'none';
      }, 5e3));
  } else originalShowMessage(e, t);
};
