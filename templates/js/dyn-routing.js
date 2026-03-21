let currentPage = 1,
    pageSize = 20,
    totalRoutes = 0,
    routes = [];
async 
function loadRoutes() {
    document.getElementById("routesTableBody").innerHTML = '<tr><td colspan="9" class="loading">加载中...</td></tr>';
    try {
        let e = document.getElementById("filterType").value,
            t = document.getElementById("filterStatus").value,
            a = `/api/admin/dynamic-routes?page=${currentPage}&limit=` + pageSize;
        e && (a += "&handler_type=" + e), "enabled" === t && (a += "&enabled=true"), "disabled" === t && (a += "&enabled=false");
        var o = await(await fetch(a)).json();
        o.success ? (routes = o.data.routes, totalRoutes = o.data.total, renderRoutes(), renderPagination()) : showMessage("error", o.message || "加载失败")
    } catch (e) {
        showMessage("error", "网络错误: " + e.message)
    }
}
async 
function loadStats() {
    try {
        var e, t, a, o, n, s, r = await(await fetch("/api/admin/dynamic-routes?page=1&limit=1000")).json();
        r.success && (t = (e = r.data.routes).filter(e => e.enabled).length, a = e.filter(e => !e.enabled).length, (o = document.getElementById("totalRoutes")) && (o.textContent = e.length), (n = document.getElementById("enabledRoutes")) && (n.textContent = t), s = document.getElementById("disabledRoutes")) && (s.textContent = a), loadStorageStats()
    } catch (e) {
        console.error("加载统计信息失败:", e)
    }
}
async 
function loadStorageStats() {
    try {
        var e, t, a, o = await(await fetch("/api/admin/dynamic-routes/storage/stats")).json();
        o.database && (e = document.getElementById("databaseRoutes")) && (e.textContent = o.database.total_routes), o.memory && (t = document.getElementById("memoryRoutes")) && (t.textContent = o.memory.total_routes), o.file && (a = document.getElementById("fileRoutes")) && (a.textContent = o.file.total_routes)
    } catch (e) {
        console.error("加载存储统计失败:", e)
    }
}

function renderRoutes() {
    var e = document.getElementById("routesTableBody");
    const o = document.getElementById("searchInput").value.toLowerCase();
    var t = routes.filter(e => {
        var t = (e.route_name || "").toLowerCase(),
            a = e.path.toLowerCase(),
            e = e.handler_type.toLowerCase();
        return t.includes(o) || a.includes(o) || e.includes(o)
    });
    0 === t.length ? e.innerHTML = '<tr><td colspan="9" class="empty-state">暂无路由数据</td></tr>' : e.innerHTML = t.map((e, t) => {
        var a = e.route_name ? `<div style="font-weight: 500; margin-bottom: 2px;">${escapeHtml(e.route_name)}</div>` : "",
            o = getHandlerTypeLabel(e.handler_type),
            n = getStorageTypeLabel(e.route_type || "database");
        return `
        <tr>
            <td>${t+1}</td>
            <td>${a}</td>
            <td><span class="route-path">${escapeHtml(e.path)}</span></td>
            <td><span class="badge badge-warning handler-type-display">${o}</span></td>
            <td>
                <span class="badge ${e.enabled?"badge-success":"badge-danger"}">
                    ${e.enabled?"已启用":"已禁用"}
                </span>
            </td>
            <td><span class="priority-display">${e.priority}</span></td>
            <td><span class="badge badge-info storage-type-display">${n}</span></td>
            <td class="actions">
                <button class="btn btn-sm btn-primary" onclick="editRoute(${e.id})" title="编辑">编辑</button>
                <button class="btn btn-sm ${e.enabled?"btn-secondary":"btn-success"}"
                        onclick="toggleRoute(${e.id}, ${!e.enabled})"
                        title="${e.enabled?"禁用":"启用"}">
                    ${e.enabled?"禁用":"启用"}
                </button>
                <button class="btn btn-sm btn-danger" onclick="deleteRoute(${e.id})" title="删除">删除</button>
            </td>
        </tr>
    `
    }).join("")
}

function getHandlerTypeLabel(e) {
    return {
        redirect: "重定向",
        static: "静态内容",
        proxy: "代理",
        custom: "自定义"
    }[e] || e
}

function getStorageTypeLabel(e) {
    return {
        database: "数据库",
        memory: "内存",
        file: "文件"
    }[e] || e
}

function renderPagination() {
    var e, t = Math.ceil(totalRoutes / pageSize),
        a = document.getElementById("pagination");
    t <= 1 ? a.innerHTML = "" : (e = `<button class="btn btn-secondary" onclick="goToPage(${currentPage-1})"
                    ${1===currentPage?"disabled":""}>上一页</button>`, e = (e += `<span>第 ${currentPage} / ${t} 页</span>`) + `<button class="btn btn-secondary" onclick="goToPage(${currentPage+1})"
                    ${currentPage===t?"disabled":""}>下一页</button>`, a.innerHTML = e)
}

function goToPage(e) {
    var t = Math.ceil(totalRoutes / pageSize);
    e < 1 || t < e || (currentPage = e, loadRoutes())
}

function searchRoutes() {
    renderRoutes()
}

function filterRoutes() {
    currentPage = 1, loadRoutes()
}

function refreshRoutes() {
    currentPage = 1, loadRoutes(), loadStats()
}

function updateHandlerFields() {
    var e = document.getElementById("handlerType").value,
        t = document.getElementById("routeType").value,
        a = document.getElementById("contentTypeGroup"),
        o = document.getElementById("inlineTemplateGroup"),
        n = document.getElementById("templatePathGroup"),
        s = document.getElementById("uploadFileBtn");
    a && (a.style.display = "none"), o && (o.style.display = "none"), n && (n.style.display = "none"), s && (s.style.display = "none"), "static" === e && (a && (a.style.display = "block"), "database" === t || "memory" === t ? (o && (o.style.display = "block"), s && (s.style.display = "inline-block")) : "file" === t && n && (n.style.display = "block")), loadTemplate()
}

function updateRouteTypeFields() {
    var e = document.getElementById("routeType").value,
        t = document.getElementById("handlerType").value,
        a = document.getElementById("inlineTemplateGroup"),
        o = document.getElementById("templatePathGroup"),
        n = document.getElementById("uploadFileBtn");
    a && (a.style.display = "none"), o && (o.style.display = "none"), n && (n.style.display = "none"), "static" === t && ("database" === e || "memory" === e ? (a && (a.style.display = "block"), n && (n.style.display = "inline-block")) : "file" === e && o && (o.style.display = "block"))
}

function updateContentTypeTemplate() {
    var e = document.getElementById("contentType").value;
    "static" === document.getElementById("handlerType").value && e && loadTemplate()
}

function showAddModal() {
    var e, t;
    for ([e, t] of Object.entries({
            modalTitle: "textContent",
            routeId: "value",
            routeName: "value",
            routeType: "value",
            routePath: "value",
            handlerType: "value",
            handlerConfig: "value",
            contentType: "value",
            routePriority: "value",
            routeEnabled: "checked",
            modalMessage: "innerHTML",
            contentTypeGroup: "style.display",
            inlineTemplateGroup: "style.display",
            templatePathGroup: "style.display",
            routeModal: "classList",
            groupId: "value",
            isPrimaryEntry: "checked",
            groupName: "value"
        }))
        if (!document.getElementById(e)) return console.error(`元素 ${e} 不存在`), void alert(`页面加载错误：找不到元素 ${e}，请刷新页面重试`);
    document.getElementById("modalTitle").textContent = "添加路由", document.getElementById("routeId").value = "", document.getElementById("routeName").value = "", document.getElementById("routeType").value = "database", document.getElementById("routePath").value = "", document.getElementById("handlerType").value = "", document.getElementById("handlerConfig").value = "", document.getElementById("contentType").value = "", document.getElementById("inlineTemplate").value = "", document.getElementById("templatePath").value = "", document.getElementById("routeMetadata").value = "", document.getElementById("routePriority").value = "0", document.getElementById("routeEnabled").checked = !0, document.getElementById("modalMessage").innerHTML = "", document.getElementById("contentTypeGroup").style.display = "none", document.getElementById("inlineTemplateGroup").style.display = "none", document.getElementById("templatePathGroup").style.display = "none", document.getElementById("groupId").value = "", document.getElementById("isPrimaryEntry").checked = !1, document.getElementById("groupName").value = "";
    var a = document.getElementById("uploadFileBtn");
    a && (a.style.display = "none"), document.getElementById("routeModal").classList.add("active")
}

function editRoute(t) {
    var e = routes.find(e => e.id === t);
    if (e) {
        for (const a of["modalTitle", "routeId", "routeName", "routeType", "routePath", "handlerType", "contentType", "handlerConfig", "routePriority", "routeEnabled", "inlineTemplate", "templatePath", "routeMetadata", "modalMessage", "routeModal", "groupId", "isPrimaryEntry", "groupName", "groupSettings", "groupNameGroup"])
            if (!document.getElementById(a)) return console.error(`元素 ${a} 不存在`), void alert(`页面加载错误：找不到元素 ${a}，请刷新页面重试`);
        document.getElementById("modalTitle").textContent = "编辑路由", document.getElementById("routeId").value = e.id, document.getElementById("routeName").value = e.route_name || "", document.getElementById("routeType").value = e.route_type || "database", document.getElementById("routePath").value = e.path, document.getElementById("handlerType").value = e.handler_type, document.getElementById("contentType").value = e.content_type_hint || "", document.getElementById("handlerConfig").value = JSON.stringify(e.handler_config, null, 2), document.getElementById("routePriority").value = e.priority, document.getElementById("routeEnabled").checked = e.enabled, document.getElementById("inlineTemplate").value = e.inline_template || "", document.getElementById("templatePath").value = e.template_path || "", document.getElementById("routeMetadata").value = e.metadata ? JSON.stringify(e.metadata, null, 2) : "", document.getElementById("modalMessage").innerHTML = "", updateHandlerFields(), document.getElementById("routeModal").classList.add("active");
        var n = e.metadata || {};
        document.getElementById("groupId").value = e.group_id || n.group_id || "", document.getElementById("isPrimaryEntry").checked = e.is_primary_entry || n.is_primary_entry || !1, document.getElementById("groupName").value = n.group_name || "", document.getElementById("groupId").dispatchEvent(new Event("input"))
    }
}

function closeModal() {
    document.getElementById("routeModal").classList.remove("active")
}
document.addEventListener("DOMContentLoaded", function() {
    for (const e of["routeForm", "message", "routesTableBody", "pagination", "totalRoutes", "enabledRoutes", "disabledRoutes", "searchInput", "filterType", "filterStatus", "databaseRoutes", "memoryRoutes", "fileRoutes"])
        if (!document.getElementById(e)) return console.error("页面加载错误：找不到元素 " + e), void alert(`页面加载错误：找不到元素 ${e}，请刷新页面重试`);
    loadRoutes(), loadStats(), document.getElementById("routeForm").addEventListener("submit", function(e) {
        e.preventDefault(), saveRoute()
    })
});
let isSubmitting = !1;

function containsControlChars(t, a = !1) {
    for (let e = 0; e < t.length; e++) {
        var o = t.charCodeAt(e);
        if ((o < 32 || 127 === o) && (!a || 9 !== o && 10 !== o && 13 !== o)) return !0
    }
    return !1
}
async 
function saveRoute() {
    if (isSubmitting) showMessage("modalError", "正在提交中，请稍候...");
    else {
        var t = document.getElementById("routeId").value,
            a = document.getElementById("routeName").value.trim(),
            o = document.getElementById("routeType").value,
            n = document.getElementById("routePath").value.trim(),
            s = document.getElementById("handlerType").value,
            r = document.getElementById("handlerConfig").value,
            l = document.getElementById("contentType").value,
            d = document.getElementById("inlineTemplate").value.trim(),
            u = document.getElementById("templatePath").value.trim(),
            i = document.getElementById("routeMetadata").value.trim(),
            m = parseInt(document.getElementById("routePriority").value),
            c = document.getElementById("routeEnabled").checked;
        if (a && containsControlChars(a)) showMessage("modalError", "路由名称不能包含控制字符");
        else if (n && containsControlChars(n)) showMessage("modalError", "路由路径不能包含控制字符");
        else if (u && containsControlChars(u)) showMessage("modalError", "模板路径不能包含控制字符");
        else if (i && containsControlChars(i)) showMessage("modalError", "扩展元数据不能包含控制字符");
        else if (n)
            if (o)
                if (s) {
                    try {
                        JSON.parse(r)
                    } catch (e) {
                        return void showMessage("modalError", "处理器配置必须是有效的JSON格式")
                    }
                    let e = null;
                    if (i) try {
                        e = JSON.parse(i)
                    } catch (e) {
                        return void showMessage("modalError", "扩展元数据必须是有效的JSON格式")
                    }
                    var p = document.getElementById("groupId").value.trim(),
                        g = document.getElementById("isPrimaryEntry").checked,
                        y = document.getElementById("groupName").value.trim();
                    p && (e = e || {}, e.group_name = y || null);
                    if (i = JSON.parse(r), "database" === o || "memory" === o) {
                        if (u) return void showMessage("modalError", o + " 类型路由不支持 template_path 字段")
                    } else if ("file" === o) {
                        if (d) return void showMessage("modalError", "file 类型路由不支持 inline_template 字段");
                        if (!u) return void showMessage("modalError", "file 类型路由必须提供 template_path")
                    }
                    r = {
                        route_name: a || null,
                        route_type: o || "database",
                        path: n,
                        handler_type: s,
                        handler_config: i,
                        content_type_hint: l || null,
                        inline_template: d || null,
                        template_path: u || null,
                        metadata: e,
                        enabled: c,
                        priority: m,
                        group_id: p || null,
                        is_primary_entry: g || null
                    }, isSubmitting = !0, (a = document.querySelector("#routeForm .btn-primary")) && (a.disabled = !0, a.textContent = "提交中...");
                    try {
                        var y = await(await fetch(t ? "/api/admin/dynamic-routes/" + t : "/api/admin/dynamic-routes", {
                            method: t ? "PUT" : "POST",
                            headers: {
                                "Content-Type": "application/json"
                            },
                            body: JSON.stringify(r)
                        })).json();
                        y.success ? (showMessage("success", y.message || "保存成功"), closeModal(), loadRoutes(), loadStats()) : showMessage("modalError", y.message || "保存失败")
                    } catch (e) {
                        showMessage("modalError", "网络错误: " + e.message)
                    } finally {
                        isSubmitting = !1, a && (a.disabled = !1, a.textContent = "保存")
                    }
                } else showMessage("modalError", "请选择处理器类型");
        else showMessage("modalError", "请选择路由类型");
        else showMessage("modalError", "路由路径不能为空")
    }
}
async 
function testRoute() {
    var e = document.getElementById("routePath"),
        t = document.getElementById("routeType"),
        a = document.getElementById("handlerType"),
        o = document.getElementById("handlerConfig"),
        n = document.getElementById("contentSource");
    if (e && t && a && o)
        if (e = e.value.trim(), t = t.value, a = a.value, o = o.value, n = n ? n.value : null, e && t && a) try {
            var s = await(await fetch("/api/admin/dynamic-routes/test", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    route_type: t || "database",
                    path: e,
                    handler_type: a,
                    handler_config: JSON.parse(o),
                    content_source: n || null
                })
            })).json();
            if (s.success) {
                let e = s.data,
                    t = "路由测试成功";
                e.conflict && (t += " (路径冲突)"), e.response_preview && (t += "\n响应预览: " + JSON.stringify(e.response_preview, null, 2)), alert(t)
            } else alert("测试失败: " + s.message)
        } catch (e) {
            alert("测试失败: " + e.message)
        } else alert("请先填写路由路径、选择路由类型和处理器类型");
        else alert("缺少必要的表单元素")
}
async 
function toggleRoute(e, t) {
    t = t ? "enable" : "disable";
    try {
        var a = await(await fetch(`/api/admin/dynamic-routes/${e}/` + t, {
            method: "POST"
        })).json();
        a.success ? (showMessage("success", a.message || "操作成功"), loadRoutes(), loadStats()) : showMessage("error", a.message || "操作失败")
    } catch (e) {
        showMessage("error", "网络错误: " + e.message)
    }
}
async 
function deleteRoute(e) {
    if (confirm("确定要删除这个路由吗？")) try {
        var t = await(await fetch("/api/admin/dynamic-routes/" + e, {
            method: "DELETE"
        })).json();
        t.success ? (showMessage("success", t.message || "删除成功"), loadRoutes(), loadStats()) : showMessage("error", t.message || "删除失败")
    } catch (e) {
        showMessage("error", "网络错误: " + e.message)
    }
}
async 
function exportRoutes() {
    try {
        var e, t, a, o = await(await fetch("/api/admin/dynamic-routes/export")).json();
        o.success ? (e = new Blob([JSON.stringify(o.data, null, 2)], {
            type: "application/json"
        }), t = URL.createObjectURL(e), (a = document.createElement("a")).href = t, a.download = `routes-export-${(new Date).toISOString().split("T")[0]}.json`, document.body.appendChild(a), a.click(), document.body.removeChild(a), URL.revokeObjectURL(t)) : showMessage("error", o.message || "导出失败")
    } catch (e) {
        showMessage("error", "网络错误: " + e.message)
    }
}

function showImportModal() {
    document.getElementById("importConfig").value = "", document.getElementById("importMessage").innerHTML = "", document.getElementById("importModal").classList.add("active")
}

function closeImportModal() {
    document.getElementById("importModal").classList.remove("active")
}
async 
function importRoutes() {
    var e = document.getElementById("importConfig").value.trim();
    if (e) try {
        var t, a = JSON.parse(e),
            o = await(await fetch("/api/admin/dynamic-routes/import", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify(a)
            })).json();
        o.success ? (t = o.data, alert(`导入完成: 成功 ${t.imported}, 跳过 ${t.skipped}, 失败 ` + t.failed), closeImportModal(), loadRoutes(), loadStats()) : showMessage("importError", o.message || "导入失败")
    } catch (e) {
        showMessage("importError", "配置格式错误: " + e.message)
    } else showMessage("importError", "请输入配置内容")
}

function formatConfig() {
    var e = document.getElementById("handlerConfig");
    try {
        var t = JSON.parse(e.value);
        e.value = JSON.stringify(t, null, 2)
    } catch (e) {
        alert("JSON格式错误")
    }
}

function validateConfig() {
    var e = document.getElementById("handlerConfig");
    try {
        JSON.parse(e.value), alert("配置格式正确")
    } catch (e) {
        alert("JSON格式错误: " + e.message)
    }
}

function formatMetadata() {
    var e = document.getElementById("routeMetadata");
    try {
        var t = JSON.parse(e.value);
        e.value = JSON.stringify(t, null, 2)
    } catch (e) {
        alert("JSON格式错误")
    }
}

function validateMetadata() {
    var e = document.getElementById("routeMetadata");
    try {
        JSON.parse(e.value), alert("元数据格式正确")
    } catch (e) {
        alert("JSON格式错误: " + e.message)
    }
}

function loadTemplate() {
    var t = document.getElementById("handlerType"),
        a = document.getElementById("contentSource"),
        o = document.getElementById("contentType");
    if (t) {
        var t = t.value,
            n = (a && a.value, o ? o.value : "");
        let e = {};
        switch (t) {
            case "redirect":
                e = {
                    type: "redirect",
                    target: "/new-location",
                    status_code: 302,
                    preserve_query: !0
                };
                break;
            case "static":
                e = {
                    type: "static",
                    content_type: n || "text/html; charset=utf-8",
                    headers: {
                        "Cache-Control": "public, max-age=3600"
                    }
                };
                break;
            case "proxy":
                e = {
                    type: "proxy",
                    target: "http://backend-service:8080",
                    timeout: 5e3,
                    strip_prefix: !1
                };
                break;
            case "custom":
                e = {
                    type: "custom",
                    script: "lua",
                    source: 'function handle(req) return {status=200, body="OK"} end'
                };
                break;
            default:
                return
        }(a = document.getElementById("handlerConfig")) ? a.value.trim() || (a.value = JSON.stringify(e, null, 2)): console.error("loadTemplate: handlerConfig元素不存在")
    } else console.error("loadTemplate: handlerType元素不存在")
}

function handleFileUpload(e) {
    const o = e.target.files[0];
    var t;
    o && (1048576 < o.size ? alert("文件大小不能超过1MB") : ((t = new FileReader).onload = function(e) {
        var e = e.target.result,
            t = o.name;
        let a = "text/plain; charset=utf-8";
        switch (t.split(".").pop().toLowerCase()) {
            case "html":
            case "htm":
                a = "text/html; charset=utf-8";
                break;
            case "svg":
                a = "image/svg+xml; charset=utf-8";
                break;
            case "xml":
                a = "application/xml; charset=utf-8";
                break;
            case "css":
                a = "text/css; charset=utf-8";
                break;
            case "js":
                a = "application/javascript; charset=utf-8";
                break;
            case "json":
                a = "application/json; charset=utf-8";
                break;
            default:
                a = "text/plain; charset=utf-8"
        }
        document.getElementById("handlerType").value = "static", document.getElementById("contentType").value = a, updateHandlerFields(), document.getElementById("inlineTemplate").value = e, document.getElementById("handlerConfig").value = JSON.stringify({
            type: "static",
            headers: {
                "Cache-Control": "public, max-age=3600"
            }
        }, null, 2), showMessage("success", `文件 "${t}" 上传成功，已自动设置为静态内容处理器，内容已填充到内联模板字段`)
    }, t.onerror = function() {
        alert("文件读取失败"), e.target.value = ""
    }, t.readAsText(o)), e.target.value = "")
}

function showMessage(e, t) {
    const a = document.getElementById("modalError" === e ? "modalMessage" : "importError" === e ? "importMessage" : "message");
    a.className = e.includes("error") ? "error" : "success", a.textContent = t, a.style.display = "block", setTimeout(() => {
        a.style.display = "none"
    }, 5e3)
}

function escapeHtml(e) {
    var t = document.createElement("div");
    return t.textContent = e, t.innerHTML
}

function openStorageModal() {
    document.getElementById("storageModal").classList.add("active"), refreshStorageStats()
}

function closeStorageModal() {
    document.getElementById("storageModal").classList.remove("active"), document.getElementById("storageMessage").style.display = "none"
}
async 
function refreshStorageStats() {
    try {
        var e = await(await fetch("/api/admin/dynamic-routes/storage/stats")).json();
        e.database && (document.getElementById("storageDatabaseTotal").textContent = e.database.total_routes, document.getElementById("storageDatabaseEnabled").textContent = e.database.enabled_routes, document.getElementById("storageDatabaseDisabled").textContent = e.database.disabled_routes, document.getElementById("storageDatabaseMemory").textContent = e.database.memory_usage_bytes), e.memory && (document.getElementById("storageMemoryTotal").textContent = e.memory.total_routes, document.getElementById("storageMemoryEnabled").textContent = e.memory.enabled_routes, document.getElementById("storageMemoryDisabled").textContent = e.memory.disabled_routes, document.getElementById("storageMemoryMemory").textContent = e.memory.memory_usage_bytes), e.file && (document.getElementById("storageFileTotal").textContent = e.file.total_routes, document.getElementById("storageFileEnabled").textContent = e.file.enabled_routes, document.getElementById("storageFileDisabled").textContent = e.file.disabled_routes, document.getElementById("storageFileMemory").textContent = e.file.memory_usage_bytes), e.database && (document.getElementById("databaseRoutes").textContent = e.database.total_routes), e.memory && (document.getElementById("memoryRoutes").textContent = e.memory.total_routes), e.file && (document.getElementById("fileRoutes").textContent = e.file.total_routes)
    } catch (e) {
        console.error("加载存储统计失败:", e), showMessage("storageError", "加载存储统计失败: " + e.message)
    }
}
async 
function batchMigrateRoutes() {
    var e = document.getElementById("migrateFrom").value,
        t = document.getElementById("migrateTo").value;
    if (e === t) showMessage("storageError", "源存储类型和目标存储类型不能相同");
    else if (confirm(`确定要将所有路由从 ${e} 迁移到 ${t} 吗？此操作不可逆。`)) try {
        var a = await(await fetch("/api/admin/dynamic-routes/storage/batch-migrate", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                source_type: e,
                target_type: t
            })
        })).json();
        a.success ? (showMessage("storageSuccess", a.message), refreshStorageStats(), loadRoutes(), loadStats()) : showMessage("storageError", a.message || "迁移失败")
    } catch (e) {
        showMessage("storageError", "网络错误: " + e.message)
    }
}
async 
function clearStorage() {
    var e = document.getElementById("clearStorageType").value;
    if (confirm(`确定要清空 ${e} 存储中的所有路由吗？此操作不可逆。`)) try {
        var t = await(await fetch("/api/admin/dynamic-routes/storage/clear/" + e, {
            method: "POST"
        })).json();
        t.success ? (showMessage("storageSuccess", t.message), refreshStorageStats(), loadRoutes(), loadStats()) : showMessage("storageError", t.message || "清空失败")
    } catch (e) {
        showMessage("storageError", "网络错误: " + e.message)
    }
}
const originalShowMessage = showMessage;
showMessage = function(e, t) {
    if ("storageSuccess" === e || "storageError" === e) {
        const a = document.getElementById("storageMessage");
        a.className = "storageError" === e ? "error" : "success", a.textContent = t, a.style.display = "block", setTimeout(() => {
            a.style.display = "none"
        }, 5e3)
    } else originalShowMessage(e, t)
};
