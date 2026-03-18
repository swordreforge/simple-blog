// 全局状态
let currentPage = 1;
let pageSize = 20;
let totalRoutes = 0;
let routes = [];

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', function() {
    // 检查必要的页面元素是否存在
    const requiredElements = ['routeForm', 'message', 'routesTableBody', 'pagination',
                             'totalRoutes', 'enabledRoutes', 'disabledRoutes',
                             'searchInput', 'filterType', 'filterStatus',
                             'databaseRoutes', 'memoryRoutes', 'fileRoutes'];
    for (const elementId of requiredElements) {
        if (!document.getElementById(elementId)) {
            console.error(`页面加载错误：找不到元素 ${elementId}`);
            alert(`页面加载错误：找不到元素 ${elementId}，请刷新页面重试`);
            return;
        }
    }

    loadRoutes();
    loadStats();

    // 表单提交事件
    document.getElementById('routeForm').addEventListener('submit', function(e) {
        e.preventDefault();
        saveRoute();
    });
});

// 加载路由列表
async function loadRoutes() {
    const tbody = document.getElementById('routesTableBody');
    tbody.innerHTML = '<tr><td colspan="9" class="loading">加载中...</td></tr>';

    try {
        const filterType = document.getElementById('filterType').value;
        const filterStatus = document.getElementById('filterStatus').value;

        let url = `/api/admin/dynamic-routes?page=${currentPage}&limit=${pageSize}`;
        if (filterType) url += `&handler_type=${filterType}`;
        if (filterStatus === 'enabled') url += `&enabled=true`;
        if (filterStatus === 'disabled') url += `&enabled=false`;

        const response = await fetch(url);
        const data = await response.json();

        if (data.success) {
            routes = data.data.routes;
            totalRoutes = data.data.total;
            renderRoutes();
            renderPagination();
        } else {
            showMessage('error', data.message || '加载失败');
        }
    } catch (error) {
        showMessage('error', '网络错误: ' + error.message);
    }
}

// 加载统计信息
async function loadStats() {
    try {
        const response = await fetch('/api/admin/dynamic-routes?page=1&limit=1000');
        const data = await response.json();

        if (data.success) {
            const allRoutes = data.data.routes;
            const enabled = allRoutes.filter(r => r.enabled).length;
            const disabled = allRoutes.filter(r => !r.enabled).length;

            const totalRoutesElement = document.getElementById('totalRoutes');
            if (totalRoutesElement) {
                totalRoutesElement.textContent = allRoutes.length;
            }
            const enabledRoutesElement = document.getElementById('enabledRoutes');
            if (enabledRoutesElement) {
                enabledRoutesElement.textContent = enabled;
            }
            const disabledRoutesElement = document.getElementById('disabledRoutes');
            if (disabledRoutesElement) {
                disabledRoutesElement.textContent = disabled;
            }
        }

        // 同时加载存储统计信息
        loadStorageStats();
    } catch (error) {
        console.error('加载统计信息失败:', error);
    }
}

// 加载存储统计信息（用于主页面）
async function loadStorageStats() {
    try {
        const response = await fetch('/api/admin/dynamic-routes/storage/stats');
        const data = await response.json();

        if (data.database) {
            const databaseRoutesElement = document.getElementById('databaseRoutes');
            if (databaseRoutesElement) {
                databaseRoutesElement.textContent = data.database.total_routes;
            }
        }
        if (data.memory) {
            const memoryRoutesElement = document.getElementById('memoryRoutes');
            if (memoryRoutesElement) {
                memoryRoutesElement.textContent = data.memory.total_routes;
            }
        }
        if (data.file) {
            const fileRoutesElement = document.getElementById('fileRoutes');
            if (fileRoutesElement) {
                fileRoutesElement.textContent = data.file.total_routes;
            }
        }
    } catch (error) {
        console.error('加载存储统计失败:', error);
    }
}

// 渲染路由列表
function renderRoutes() {
    const tbody = document.getElementById('routesTableBody');
    const searchTerm = document.getElementById('searchInput').value.toLowerCase();

    const filteredRoutes = routes.filter(route => {
        const routeName = (route.route_name || '').toLowerCase();
        const path = route.path.toLowerCase();
        const handlerType = route.handler_type.toLowerCase();
        return routeName.includes(searchTerm) ||
               path.includes(searchTerm) ||
               handlerType.includes(searchTerm);
    });

    if (filteredRoutes.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" class="empty-state">暂无路由数据</td></tr>';
        return;
    }

    tbody.innerHTML = filteredRoutes.map(route => {
        const routeNameDisplay = route.route_name ?
            `<div style="font-weight: 500; margin-bottom: 2px;">${escapeHtml(route.route_name)}</div>` : '';
        const handlerTypeLabel = getHandlerTypeLabel(route.handler_type);
        // 使用 route_type 作为存储类型
        const storageType = route.route_type || 'database';
        const storageTypeLabel = getStorageTypeLabel(storageType);

        return `
        <tr>
            <td>${route.id}</td>
            <td>${routeNameDisplay}</td>
            <td><span class="route-path">${escapeHtml(route.path)}</span></td>
            <td><span class="badge badge-warning handler-type-display">${handlerTypeLabel}</span></td>
            <td>
                <span class="badge ${route.enabled ? 'badge-success' : 'badge-danger'}">
                    ${route.enabled ? '已启用' : '已禁用'}
                </span>
            </td>
            <td><span class="priority-display">${route.priority}</span></td>
            <td><span class="badge badge-info storage-type-display">${storageTypeLabel}</span></td>
            <td class="actions">
                <button class="btn btn-sm btn-primary" onclick="editRoute(${route.id})" title="编辑">编辑</button>
                <button class="btn btn-sm ${route.enabled ? 'btn-secondary' : 'btn-success'}"
                        onclick="toggleRoute(${route.id}, ${!route.enabled})"
                        title="${route.enabled ? '禁用' : '启用'}">
                    ${route.enabled ? '禁用' : '启用'}
                </button>
                <button class="btn btn-sm btn-danger" onclick="deleteRoute(${route.id})" title="删除">删除</button>
            </td>
        </tr>
    `}).join('');
}

// 获取处理器类型标签
function getHandlerTypeLabel(handlerType) {
    const labels = {
        'redirect': '重定向',
        'static': '静态内容',
        'template': '模板渲染',
        'proxy': '代理',
        'custom': '自定义'
    };
    return labels[handlerType] || handlerType;
}

// 获取存储类型标签
function getStorageTypeLabel(routeType) {
    const labels = {
        'database': '数据库',
        'memory': '内存',
        'file': '文件'
    };
    return labels[routeType] || routeType;
}

// 渲染分页
function renderPagination() {
    const totalPages = Math.ceil(totalRoutes / pageSize);
    const pagination = document.getElementById('pagination');

    if (totalPages <= 1) {
        pagination.innerHTML = '';
        return;
    }

    let html = `<button class="btn btn-secondary" onclick="goToPage(${currentPage - 1})"
                    ${currentPage === 1 ? 'disabled' : ''}>上一页</button>`;

    html += `<span>第 ${currentPage} / ${totalPages} 页</span>`;

    html += `<button class="btn btn-secondary" onclick="goToPage(${currentPage + 1})"
                    ${currentPage === totalPages ? 'disabled' : ''}>下一页</button>`;

    pagination.innerHTML = html;
}

// 跳转到指定页面
function goToPage(page) {
    const totalPages = Math.ceil(totalRoutes / pageSize);
    if (page < 1 || page > totalPages) return;

    currentPage = page;
    loadRoutes();
}

// 搜索路由
function searchRoutes() {
    renderRoutes();
}

// 筛选路由
function filterRoutes() {
    currentPage = 1;
    loadRoutes();
}

// 刷新路由列表
function refreshRoutes() {
    currentPage = 1;
    loadRoutes();
    loadStats();
}

// 更新处理器字段显示
function updateHandlerFields() {
    const handlerType = document.getElementById('handlerType').value;
    const routeType = document.getElementById('routeType').value;
    const contentTypeGroup = document.getElementById('contentTypeGroup');
    const inlineTemplateGroup = document.getElementById('inlineTemplateGroup');
    const templatePathGroup = document.getElementById('templatePathGroup');
    const uploadFileBtn = document.getElementById('uploadFileBtn');

    // 重置所有字段显示
    if (contentTypeGroup) contentTypeGroup.style.display = 'none';
    if (inlineTemplateGroup) inlineTemplateGroup.style.display = 'none';
    if (templatePathGroup) templatePathGroup.style.display = 'none';
    if (uploadFileBtn) {
        uploadFileBtn.style.display = 'none';
    }

    // 根据处理器类型显示相应字段
    if (handlerType === 'static' || handlerType === 'template') {
        if (contentTypeGroup) contentTypeGroup.style.display = 'block';

        // 根据 route_type 显示相应的模板字段
        if (routeType === 'database' || routeType === 'memory') {
            if (inlineTemplateGroup) inlineTemplateGroup.style.display = 'block';
            if (uploadFileBtn) {
                uploadFileBtn.style.display = 'inline-block';
            }
        } else if (routeType === 'file') {
            if (templatePathGroup) templatePathGroup.style.display = 'block';
        }
    }

    // 加载对应的模板
    loadTemplate();
}

// 更新路由类型字段显示
function updateRouteTypeFields() {
    const routeType = document.getElementById('routeType').value;
    const handlerType = document.getElementById('handlerType').value;
    const inlineTemplateGroup = document.getElementById('inlineTemplateGroup');
    const templatePathGroup = document.getElementById('templatePathGroup');
    const uploadFileBtn = document.getElementById('uploadFileBtn');

    // 重置模板字段显示
    if (inlineTemplateGroup) inlineTemplateGroup.style.display = 'none';
    if (templatePathGroup) templatePathGroup.style.display = 'none';
    if (uploadFileBtn) {
        uploadFileBtn.style.display = 'none';
    }

    // 仅对 static/template 类型处理器显示模板字段
    if (handlerType === 'static' || handlerType === 'template') {
        if (routeType === 'database' || routeType === 'memory') {
            if (inlineTemplateGroup) inlineTemplateGroup.style.display = 'block';
            if (uploadFileBtn) {
                uploadFileBtn.style.display = 'inline-block';
            }
        } else if (routeType === 'file') {
            if (templatePathGroup) templatePathGroup.style.display = 'block';
        }
    }
}

// 更新内容类型模板
function updateContentTypeTemplate() {
    const contentType = document.getElementById('contentType').value;
    const handlerType = document.getElementById('handlerType').value;

    if (handlerType === 'static' && contentType) {
        loadTemplate();
    }
}

// 显示添加路由模态框
function showAddModal() {
    // 检查必要元素是否存在
    const requiredElements = {
        'modalTitle': 'textContent',
        'routeId': 'value',
        'routeName': 'value',
        'routeType': 'value',
        'routePath': 'value',
        'handlerType': 'value',
        'handlerConfig': 'value',
        'contentType': 'value',
        'routePriority': 'value',
        'routeEnabled': 'checked',
        'modalMessage': 'innerHTML',
        'contentTypeGroup': 'style.display',
        'inlineTemplateGroup': 'style.display',
        'templatePathGroup': 'style.display',
        'routeModal': 'classList'
    };

    // 检查所有必需的元素
    for (const [elementId, property] of Object.entries(requiredElements)) {
        const element = document.getElementById(elementId);
        if (!element) {
            console.error(`元素 ${elementId} 不存在`);
            alert(`页面加载错误：找不到元素 ${elementId}，请刷新页面重试`);
            return;
        }
    }

    // 设置表单值
    document.getElementById('modalTitle').textContent = '添加路由';
    document.getElementById('routeId').value = '';
    document.getElementById('routeName').value = '';
    document.getElementById('routeType').value = 'database';
    document.getElementById('routePath').value = '';
    document.getElementById('handlerType').value = '';
    document.getElementById('handlerConfig').value = '';
    document.getElementById('contentType').value = '';
    document.getElementById('inlineTemplate').value = '';
    document.getElementById('templatePath').value = '';
    document.getElementById('routeMetadata').value = '';
    document.getElementById('routePriority').value = '0';
    document.getElementById('routeEnabled').checked = true;
    document.getElementById('modalMessage').innerHTML = '';

    // 隐藏所有条件字段
    document.getElementById('contentTypeGroup').style.display = 'none';
    document.getElementById('inlineTemplateGroup').style.display = 'none';
    document.getElementById('templatePathGroup').style.display = 'none';
    const uploadFileBtn = document.getElementById('uploadFileBtn');
    if (uploadFileBtn) {
        uploadFileBtn.style.display = 'none';
    }

    document.getElementById('routeModal').classList.add('active');
}

// 编辑路由
function editRoute(id) {
    const route = routes.find(r => r.id === id);
    if (!route) return;

    // 检查必要元素是否存在
    const requiredElements = ['modalTitle', 'routeId', 'routeName', 'routeType', 'routePath', 'handlerType',
                             'contentType', 'handlerConfig', 'routePriority',
                             'routeEnabled', 'inlineTemplate', 'templatePath', 'routeMetadata', 'modalMessage', 'routeModal'];
    for (const elementId of requiredElements) {
        if (!document.getElementById(elementId)) {
            console.error(`元素 ${elementId} 不存在`);
            alert(`页面加载错误：找不到元素 ${elementId}，请刷新页面重试`);
            return;
        }
    }

    document.getElementById('modalTitle').textContent = '编辑路由';
    document.getElementById('routeId').value = route.id;
    document.getElementById('routeName').value = route.route_name || '';
    document.getElementById('routeType').value = route.route_type || 'database';
    document.getElementById('routePath').value = route.path;
    document.getElementById('handlerType').value = route.handler_type;
    document.getElementById('contentType').value = route.content_type_hint || '';
    document.getElementById('handlerConfig').value = JSON.stringify(route.handler_config, null, 2);
    document.getElementById('routePriority').value = route.priority;
    document.getElementById('routeEnabled').checked = route.enabled;
    document.getElementById('inlineTemplate').value = route.inline_template || '';
    document.getElementById('templatePath').value = route.template_path || '';
    document.getElementById('routeMetadata').value = route.metadata ? JSON.stringify(route.metadata, null, 2) : '';
    document.getElementById('modalMessage').innerHTML = '';

    // 更新字段显示
    updateHandlerFields();

    document.getElementById('routeModal').classList.add('active');
}

// 关闭模态框
function closeModal() {
    document.getElementById('routeModal').classList.remove('active');
}

// 保存路由
async function saveRoute() {
    const id = document.getElementById('routeId').value;
    const routeName = document.getElementById('routeName').value.trim();
    const routeType = document.getElementById('routeType').value;
    const path = document.getElementById('routePath').value.trim();
    const handlerType = document.getElementById('handlerType').value;
    const handlerConfig = document.getElementById('handlerConfig').value;
    const contentType = document.getElementById('contentType').value;
    const inlineTemplate = document.getElementById('inlineTemplate').value.trim();
    const templatePath = document.getElementById('templatePath').value.trim();
    const routeMetadata = document.getElementById('routeMetadata').value.trim();
    const priority = parseInt(document.getElementById('routePriority').value);
    const enabled = document.getElementById('routeEnabled').checked;

    // 验证必填字段
    if (!path) {
        showMessage('modalError', '路由路径不能为空');
        return;
    }

    if (!routeType) {
        showMessage('modalError', '请选择路由类型');
        return;
    }

    if (!handlerType) {
        showMessage('modalError', '请选择处理器类型');
        return;
    }

    // 验证配置
    try {
        JSON.parse(handlerConfig);
    } catch (e) {
        showMessage('modalError', '处理器配置必须是有效的JSON格式');
        return;
    }

    // 验证元数据（如果填写了）
    let metadata = null;
    if (routeMetadata) {
        try {
            metadata = JSON.parse(routeMetadata);
        } catch (e) {
            showMessage('modalError', '扩展元数据必须是有效的JSON格式');
            return;
        }
    }

    // 构建路由数据
    let parsedHandlerConfig = JSON.parse(handlerConfig);

    // 字段组合验证
    if (routeType === 'database' || routeType === 'memory') {
        // database/memory 类型应该使用 inline_template
        if (templatePath) {
            showMessage('modalError', `${routeType} 类型路由不支持 template_path 字段`);
            return;
        }
    } else if (routeType === 'file') {
        // file 类型应该使用 template_path
        if (inlineTemplate) {
            showMessage('modalError', 'file 类型路由不支持 inline_template 字段');
            return;
        }
        if (!templatePath) {
            showMessage('modalError', 'file 类型路由必须提供 template_path');
            return;
        }
    }

    const routeData = {
        route_name: routeName || null,
        route_type: routeType || 'database',
        path: path,
        handler_type: handlerType,
        handler_config: parsedHandlerConfig,
        content_type_hint: contentType || null,
        inline_template: inlineTemplate || null,
        template_path: templatePath || null,
        metadata: metadata,
        enabled: enabled,
        priority: priority
    };

    try {
        const url = id ? `/api/admin/dynamic-routes/${id}` : '/api/admin/dynamic-routes';
        const method = id ? 'PUT' : 'POST';

        const response = await fetch(url, {
            method: method,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(routeData)
        });

        const data = await response.json();

        if (data.success) {
            showMessage('success', data.message || '保存成功');
            closeModal();
            loadRoutes();
            loadStats();
        } else {
            showMessage('modalError', data.message || '保存失败');
        }
    } catch (error) {
        showMessage('modalError', '网络错误: ' + error.message);
    }
}

// 测试路由
async function testRoute() {
    const path = document.getElementById('routePath').value.trim();
    const routeType = document.getElementById('routeType').value;
    const handlerType = document.getElementById('handlerType').value;
    const handlerConfig = document.getElementById('handlerConfig').value;
    const contentSource = document.getElementById('contentSource').value;

    if (!path || !routeType || !handlerType) {
        alert('请先填写路由路径、选择路由类型和处理器类型');
        return;
    }

    try {
        const response = await fetch('/api/admin/dynamic-routes/test', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                route_type: routeType || 'database',
                path: path,
                handler_type: handlerType,
                handler_config: JSON.parse(handlerConfig),
                content_source: contentSource || null
            })
        });

        const data = await response.json();

        if (data.success) {
            const result = data.data;
            let message = '路由测试成功';

            if (result.conflict) {
                message += ' (路径冲突)';
            }

            if (result.response_preview) {
                message += '\n响应预览: ' + JSON.stringify(result.response_preview, null, 2);
            }

            alert(message);
        } else {
            alert('测试失败: ' + data.message);
        }
    } catch (error) {
        alert('测试失败: ' + error.message);
    }
}

// 切换路由状态
async function toggleRoute(id, enabled) {
    const action = enabled ? 'enable' : 'disable';
    try {
        const response = await fetch(`/api/admin/dynamic-routes/${id}/${action}`, {
            method: 'POST'
        });

        const data = await response.json();

        if (data.success) {
            showMessage('success', data.message || '操作成功');
            loadRoutes();
            loadStats();
        } else {
            showMessage('error', data.message || '操作失败');
        }
    } catch (error) {
        showMessage('error', '网络错误: ' + error.message);
    }
}

// 删除路由
async function deleteRoute(id) {
    if (!confirm('确定要删除这个路由吗？')) return;

    try {
        const response = await fetch(`/api/admin/dynamic-routes/${id}`, {
            method: 'DELETE'
        });

        const data = await response.json();

        if (data.success) {
            showMessage('success', data.message || '删除成功');
            loadRoutes();
            loadStats();
        } else {
            showMessage('error', data.message || '删除失败');
        }
    } catch (error) {
        showMessage('error', '网络错误: ' + error.message);
    }
}

// 导出路由配置
async function exportRoutes() {
    try {
        const response = await fetch('/api/admin/dynamic-routes/export');
        const data = await response.json();

        if (data.success) {
            const blob = new Blob([JSON.stringify(data.data, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `routes-export-${new Date().toISOString().split('T')[0]}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } else {
            showMessage('error', data.message || '导出失败');
        }
    } catch (error) {
        showMessage('error', '网络错误: ' + error.message);
    }
}

// 显示导入模态框
function showImportModal() {
    document.getElementById('importConfig').value = '';
    document.getElementById('importMessage').innerHTML = '';
    document.getElementById('importModal').classList.add('active');
}

// 关闭导入模态框
function closeImportModal() {
    document.getElementById('importModal').classList.remove('active');
}

// 导入路由配置
async function importRoutes() {
    const configText = document.getElementById('importConfig').value.trim();

    if (!configText) {
        showMessage('importError', '请输入配置内容');
        return;
    }

    try {
        const config = JSON.parse(configText);

        const response = await fetch('/api/admin/dynamic-routes/import', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(config)
        });

        const data = await response.json();

        if (data.success) {
            const result = data.data;
            alert(`导入完成: 成功 ${result.imported}, 跳过 ${result.skipped}, 失败 ${result.failed}`);
            closeImportModal();
            loadRoutes();
            loadStats();
        } else {
            showMessage('importError', data.message || '导入失败');
        }
    } catch (error) {
        showMessage('importError', '配置格式错误: ' + error.message);
    }
}

// 格式化配置
function formatConfig() {
    const textarea = document.getElementById('handlerConfig');
    try {
        const config = JSON.parse(textarea.value);
        textarea.value = JSON.stringify(config, null, 2);
    } catch (e) {
        alert('JSON格式错误');
    }
}

// 验证配置
function validateConfig() {
    const textarea = document.getElementById('handlerConfig');
    try {
        JSON.parse(textarea.value);
        alert('配置格式正确');
    } catch (e) {
        alert('JSON格式错误: ' + e.message);
    }
}

// 格式化元数据
function formatMetadata() {
    const textarea = document.getElementById('routeMetadata');
    try {
        const metadata = JSON.parse(textarea.value);
        textarea.value = JSON.stringify(metadata, null, 2);
    } catch (e) {
        alert('JSON格式错误');
    }
}

// 验证元数据
function validateMetadata() {
    const textarea = document.getElementById('routeMetadata');
    try {
        JSON.parse(textarea.value);
        alert('元数据格式正确');
    } catch (e) {
        alert('JSON格式错误: ' + e.message);
    }
}

// 加载模板
function loadTemplate() {
    const handlerTypeEl = document.getElementById('handlerType');
    const contentSourceEl = document.getElementById('contentSource');
    const contentTypeEl = document.getElementById('contentType');

    // 检查必需元素是否存在
    if (!handlerTypeEl) {
        console.error('loadTemplate: handlerType元素不存在');
        return;
    }
    
    const handlerType = handlerTypeEl.value;
    const contentSource = contentSourceEl ? contentSourceEl.value : '';
    const contentType = contentTypeEl ? contentTypeEl.value : '';

    let template = {};

    switch (handlerType) {
        case 'redirect':
            template = {
                type: 'redirect',
                target: '/new-location',
                status_code: 302,
                preserve_query: true
            };
            break;

        case 'static':
            if (contentSource === 'database') {
                // 数据库存储
                template = {
                    type: 'static',
                    content_source: 'database',
                    content: '<!DOCTYPE html>\n<html>\n<head>\n    <meta charset="UTF-8">\n    <title>自定义页面</title>\n</head>\n<body>\n    <h1>欢迎使用动态路由</h1>\n    <p>这是一个静态HTML页面示例（存储在数据库中）</p>\n</body>\n</html>',
                    content_type: contentType || 'text/html; charset=utf-8',
                    headers: {
                        'Cache-Control': 'public, max-age=3600'
                    }
                };
            } else if (contentSource === 'file') {
                // 文件系统
                template = {
                    type: 'static',
                    content_source: 'file',
                    content: '/path/to/static/file.html',
                    content_type: contentType || 'text/html; charset=utf-8',
                    headers: {
                        'Cache-Control': 'public, max-age=3600'
                    }
                };
            } else {
                // 默认模板
                template = {
                    type: 'static',
                    content: '<!DOCTYPE html>\n<html>\n<head>\n    <meta charset="UTF-8">\n    <title>自定义页面</title>\n</head>\n<body>\n    <h1>欢迎使用动态路由</h1>\n    <p>这是一个静态HTML页面示例</p>\n</body>\n</html>',
                    content_type: contentType || 'text/html; charset=utf-8',
                    headers: {
                        'Cache-Control': 'public, max-age=3600'
                    }
                };
            }
            break;

        case 'template':
            template = {
                type: 'template',
                template_name: 'custom_page.html',
                context: {
                    title: '自定义页面',
                    content: '页面内容'
                }
            };
            break;

        case 'proxy':
            template = {
                type: 'proxy',
                target: 'http://backend-service:8080',
                timeout: 5000,
                strip_prefix: false
            };
            break;

        case 'custom':
            template = {
                type: 'custom',
                script: 'lua',
                source: 'function handle(req) return {status=200, body="OK"} end'
            };
            break;

        default:
            return;
    }

    const handlerConfigEl = document.getElementById('handlerConfig');
    if (!handlerConfigEl) {
        console.error('loadTemplate: handlerConfig元素不存在');
        return;
    }
    
    handlerConfigEl.value = JSON.stringify(template, null, 2);
}

// 处理文件上传
function handleFileUpload(event) {
    const file = event.target.files[0];
    if (!file) return;

    // 验证文件大小（限制为1MB）
    if (file.size > 1024 * 1024) {
        alert('文件大小不能超过1MB');
        event.target.value = '';
        return;
    }

    const reader = new FileReader();
    reader.onload = function(e) {
        const content = e.target.result;
        const fileName = file.name;
        const fileExtension = fileName.split('.').pop().toLowerCase();

        // 根据文件扩展名确定content_type
        let contentType = 'text/plain; charset=utf-8';
        switch (fileExtension) {
            case 'html':
            case 'htm':
                contentType = 'text/html; charset=utf-8';
                break;
            case 'svg':
                contentType = 'image/svg+xml; charset=utf-8';
                break;
            case 'xml':
                contentType = 'application/xml; charset=utf-8';
                break;
            case 'css':
                contentType = 'text/css; charset=utf-8';
                break;
            case 'js':
                contentType = 'application/javascript; charset=utf-8';
                break;
            case 'json':
                contentType = 'application/json; charset=utf-8';
                break;
            default:
                contentType = 'text/plain; charset=utf-8';
        }

        // 自动切换到静态内容处理器类型
        document.getElementById('handlerType').value = 'static';
        document.getElementById('contentSource').value = 'database';
        document.getElementById('contentType').value = contentType;

        // 更新字段显示
        updateHandlerFields();

        // 将文件内容填充到内容模板字段
        document.getElementById('contentTemplate').value = content;

        // 创建适当的handler配置（不包含实际内容，只包含处理器参数）
        const config = {
            type: 'static',
            headers: {
                'Cache-Control': 'public, max-age=3600'
            }
        };

        // 将配置填充到处理器配置文本框
        document.getElementById('handlerConfig').value = JSON.stringify(config, null, 2);

        // 显示成功消息
        showMessage('success', `文件 "${fileName}" 上传成功，已自动设置为静态内容处理器，内容已填充到内容模板字段`);
    };

    reader.onerror = function() {
        alert('文件读取失败');
        event.target.value = '';
    };

    // 读取文件内容
    reader.readAsText(file);

    // 清空文件输入框，允许重复上传相同文件
    event.target.value = '';
}

// 显示消息
function showMessage(type, message) {
    const elementId = type === 'modalError' ? 'modalMessage' :
                      type === 'importError' ? 'importMessage' : 'message';

    const messageDiv = document.getElementById(elementId);
    messageDiv.className = type.includes('error') ? 'error' : 'success';
    messageDiv.textContent = message;
    messageDiv.style.display = 'block';

    setTimeout(() => {
        messageDiv.style.display = 'none';
    }, 5000);
}

// HTML转义
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// 存储管理相关函数

// 打开存储管理模态框
function openStorageModal() {
    document.getElementById('storageModal').style.display = 'block';
    refreshStorageStats();
}

// 关闭存储管理模态框
function closeStorageModal() {
    document.getElementById('storageModal').style.display = 'none';
    document.getElementById('storageMessage').style.display = 'none';
}

// 刷新存储统计信息
async function refreshStorageStats() {
    try {
        const response = await fetch('/api/admin/dynamic-routes/storage/stats');
        const data = await response.json();

        if (data.database) {
            document.getElementById('storageDatabaseTotal').textContent = data.database.total_routes;
            document.getElementById('storageDatabaseEnabled').textContent = data.database.enabled_routes;
            document.getElementById('storageDatabaseDisabled').textContent = data.database.disabled_routes;
            document.getElementById('storageDatabaseMemory').textContent = data.database.memory_usage_bytes;
        }

        if (data.memory) {
            document.getElementById('storageMemoryTotal').textContent = data.memory.total_routes;
            document.getElementById('storageMemoryEnabled').textContent = data.memory.enabled_routes;
            document.getElementById('storageMemoryDisabled').textContent = data.memory.disabled_routes;
            document.getElementById('storageMemoryMemory').textContent = data.memory.memory_usage_bytes;
        }

        if (data.file) {
            document.getElementById('storageFileTotal').textContent = data.file.total_routes;
            document.getElementById('storageFileEnabled').textContent = data.file.enabled_routes;
            document.getElementById('storageFileDisabled').textContent = data.file.disabled_routes;
            document.getElementById('storageFileMemory').textContent = data.file.memory_usage_bytes;
        }

        // 同时更新主页面的存储统计
        if (data.database) document.getElementById('databaseRoutes').textContent = data.database.total_routes;
        if (data.memory) document.getElementById('memoryRoutes').textContent = data.memory.total_routes;
        if (data.file) document.getElementById('fileRoutes').textContent = data.file.total_routes;

    } catch (error) {
        console.error('加载存储统计失败:', error);
        showMessage('storageError', '加载存储统计失败: ' + error.message);
    }
}

// 批量迁移路由
async function batchMigrateRoutes() {
    const fromType = document.getElementById('migrateFrom').value;
    const toType = document.getElementById('migrateTo').value;

    if (fromType === toType) {
        showMessage('storageError', '源存储类型和目标存储类型不能相同');
        return;
    }

    if (!confirm(`确定要将所有路由从 ${fromType} 迁移到 ${toType} 吗？此操作不可逆。`)) {
        return;
    }

    try {
        const response = await fetch('/api/admin/dynamic-routes/storage/batch-migrate', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                source_type: fromType,
                target_type: toType
            })
        });

        const data = await response.json();

        if (data.success) {
            showMessage('storageSuccess', data.message);
            refreshStorageStats();
            loadRoutes(); // 刷新路由列表
            loadStats();  // 刷新统计
        } else {
            showMessage('storageError', data.message || '迁移失败');
        }
    } catch (error) {
        showMessage('storageError', '网络错误: ' + error.message);
    }
}

// 清空存储
async function clearStorage() {
    const storageType = document.getElementById('clearStorageType').value;

    if (!confirm(`确定要清空 ${storageType} 存储中的所有路由吗？此操作不可逆。`)) {
        return;
    }

    try {
        const response = await fetch(`/api/admin/dynamic-routes/storage/clear/${storageType}`, {
            method: 'POST'
        });

        const data = await response.json();

        if (data.success) {
            showMessage('storageSuccess', data.message);
            refreshStorageStats();
            loadRoutes(); // 刷新路由列表
            loadStats();  // 刷新统计
        } else {
            showMessage('storageError', data.message || '清空失败');
        }
    } catch (error) {
        showMessage('storageError', '网络错误: ' + error.message);
    }
}

// 修改 showMessage 函数以支持存储管理消息
const originalShowMessage = showMessage;
showMessage = function(type, message) {
    if (type === 'storageSuccess' || type === 'storageError') {
        const messageDiv = document.getElementById('storageMessage');
        messageDiv.className = type === 'storageError' ? 'error' : 'success';
        messageDiv.textContent = message;
        messageDiv.style.display = 'block';

        setTimeout(() => {
            messageDiv.style.display = 'none';
        }, 5000);
    } else {
        originalShowMessage(type, message);
    }
};
