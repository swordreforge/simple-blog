// 全局状态
let currentPage = 1;
let pageSize = 20;
let totalRoutes = 0;
let routes = [];

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', function() {
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
    tbody.innerHTML = '<tr><td colspan="8" class="loading">加载中...</td></tr>';

    try {
        const filterType = document.getElementById('filterType').value;
        const filterStatus = document.getElementById('filterStatus').value;

        let url = `/api/admin/dynamic-routes?page=${currentPage}&limit=${pageSize}`;
        if (filterType) url += `&route_type=${filterType}`;
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

            document.getElementById('totalRoutes').textContent = allRoutes.length;
            document.getElementById('enabledRoutes').textContent = enabled;
            document.getElementById('disabledRoutes').textContent = disabled;

            // 计算平均响应时间（模拟数据）
            const avgTime = allRoutes.length > 0 ? '0.8ms' : '0ms';
            document.getElementById('avgResponseTime').textContent = avgTime;
        }
    } catch (error) {
        console.error('加载统计信息失败:', error);
    }
}

// 渲染路由列表
function renderRoutes() {
    const tbody = document.getElementById('routesTableBody');
    const searchTerm = document.getElementById('searchInput').value.toLowerCase();

    const filteredRoutes = routes.filter(route =>
        route.path.toLowerCase().includes(searchTerm) ||
        route.handler_type.toLowerCase().includes(searchTerm)
    );

    if (filteredRoutes.length === 0) {
        tbody.innerHTML = '<tr><td colspan="8" class="empty-state">暂无路由数据</td></tr>';
        return;
    }

    tbody.innerHTML = filteredRoutes.map(route => `
        <tr>
            <td>${route.id}</td>
            <td><span class="route-path">${escapeHtml(route.path)}</span></td>
            <td><span class="badge badge-info">${route.route_type}</span></td>
            <td><span class="badge badge-warning">${route.handler_type}</span></td>
            <td>
                <span class="badge ${route.enabled ? 'badge-success' : 'badge-danger'}">
                    ${route.enabled ? '已启用' : '已禁用'}
                </span>
            </td>
            <td>${route.priority}</td>
            <td>${route.stats ? route.stats.access_count : 0}</td>
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
    `).join('');
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

// 显示添加路由模态框
function showAddModal() {
    document.getElementById('modalTitle').textContent = '添加路由';
    document.getElementById('routeId').value = '';
    document.getElementById('routePath').value = '';
    document.getElementById('routeType').value = 'database';
    document.getElementById('handlerType').value = 'redirect';
    document.getElementById('handlerConfig').value = JSON.stringify({
        type: 'redirect',
        target: '/new-location',
        status_code: 302
    }, null, 2);
    document.getElementById('routePriority').value = '0';
    document.getElementById('routeEnabled').checked = true;
    document.getElementById('modalMessage').innerHTML = '';

    document.getElementById('routeModal').classList.add('active');
}

// 编辑路由
function editRoute(id) {
    const route = routes.find(r => r.id === id);
    if (!route) return;

    document.getElementById('modalTitle').textContent = '编辑路由';
    document.getElementById('routeId').value = route.id;
    document.getElementById('routePath').value = route.path;
    document.getElementById('routeType').value = route.route_type;
    document.getElementById('handlerType').value = route.handler_type;
    document.getElementById('handlerConfig').value = JSON.stringify(route.handler_config, null, 2);
    document.getElementById('routePriority').value = route.priority;
    document.getElementById('routeEnabled').checked = route.enabled;
    document.getElementById('modalMessage').innerHTML = '';

    document.getElementById('routeModal').classList.add('active');
}

// 关闭模态框
function closeModal() {
    document.getElementById('routeModal').classList.remove('active');
}

// 保存路由
async function saveRoute() {
    const id = document.getElementById('routeId').value;
    const path = document.getElementById('routePath').value.trim();
    const routeType = document.getElementById('routeType').value;
    const handlerType = document.getElementById('handlerType').value;
    const handlerConfig = document.getElementById('handlerConfig').value;
    const priority = parseInt(document.getElementById('routePriority').value);
    const enabled = document.getElementById('routeEnabled').checked;

    // 验证配置
    try {
        JSON.parse(handlerConfig);
    } catch (e) {
        showMessage('modalError', '处理器配置必须是有效的JSON格式');
        return;
    }

    const routeData = {
        route_type: routeType,
        path: path,
        handler_type: handlerType,
        handler_config: JSON.parse(handlerConfig),
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

    try {
        const response = await fetch('/api/admin/dynamic-routes/test', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                route_type: routeType,
                path: path,
                handler_type: handlerType,
                handler_config: JSON.parse(handlerConfig)
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

// 加载模板
function loadTemplate() {
    const handlerType = document.getElementById('handlerType').value;
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
            template = {
                type: 'static',
                content: '<!DOCTYPE html>\n<html>\n<head>\n    <meta charset="UTF-8">\n    <title>自定义页面</title>\n</head>\n<body>\n    <h1>欢迎使用动态路由</h1>\n    <p>这是一个静态HTML页面示例</p>\n</body>\n</html>',
                content_type: 'text/html; charset=utf-8',
                headers: {
                    'Cache-Control': 'public, max-age=3600'
                }
            };
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
                target: 'http://backend:8080/api',
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
    }

    document.getElementById('handlerConfig').value = JSON.stringify(template, null, 2);
}

// 更新处理器模板
function updateHandlerTemplate() {
    loadTemplate();
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

        // 创建静态内容配置
        const config = {
            type: 'static',
            content: content,
            content_type: contentType,
            headers: {
                'Cache-Control': 'public, max-age=3600'
            }
        };

        // 将配置填充到文本框
        document.getElementById('handlerConfig').value = JSON.stringify(config, null, 2);

        // 显示成功消息
        showMessage('success', `文件 "${fileName}" 上传成功，已自动设置为静态内容处理器`);
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
    const messageDiv = document.getElementById('message');
    messageDiv.className = type === 'error' ? 'error' : 'success';
    messageDiv.textContent = message;
    messageDiv.style.display = 'block';

    setTimeout(() => {
        messageDiv.style.display = 'none';
    }, 3000);
}

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