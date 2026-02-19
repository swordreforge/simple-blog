async function fetchCategories() {
    try {
        var e = localStorage.getItem("auth_token"),
            t = {
                "Content-Type": "application/json"
            };
        e && (t.Authorization = "Bearer " + e);
        var a = await (await fetch("/api/admin/categories", {
            headers: t
        })).json();
        return a.success && a.data ? a.data : (console.error("获取分类列表失败:", a.message), [])
    } catch (e) {
        return console.error("获取分类列表失败:", e), []
    }
}
async function fetchTags(t = null) {
    try {
        var a = localStorage.getItem("auth_token"),
            n = {
                "Content-Type": "application/json"
            };
        a && (n.Authorization = "Bearer " + a);
        let e = "/api/admin/tags";
        null !== t && "" !== t && (e += "?category_id=" + t);
        var o = await (await fetch(e, {
            headers: n
        })).json();
        return o.success && o.data ? o.data : (console.error("获取标签列表失败:", o.message), [])
    } catch (t) {
        return console.error("获取标签列表失败:", t), []
    }
}

function updateCategoriesTable(e) {
    const a = document.querySelector("#categories tbody");
    a && (a.innerHTML = "", 0 !== e.length ? (e.forEach(e => {
        var t = document.createElement("tr");
        t.innerHTML = `
      <td>
        <input type="checkbox" class="category-checkbox" data-id="${e.id}">
      </td>
      <td>${e.sort_order}</td>
      <td>${e.icon||""}</td>
      <td>${e.name}</td>
      <td>${e.description||"-"}</td>
      <td><span style="color: ${e.is_enabled?"#00b894":"#e74c3c"};">${e.is_enabled?"启用":"禁用"}</span></td>
      <td class="action-buttons">
        <button class="btn btn-sm btn-edit" data-action="edit-category" data-id="${e.id}">编辑</button>
        <button class="btn btn-sm btn-delete" data-action="delete-category" data-id="${e.id}">删除</button>
      </td>
    `, a.appendChild(t)
    }), bindActionButtons(), bindCategoryCheckboxes()) : a.innerHTML = '\n      <tr>\n        <td colspan="7" style="text-align: center; padding: 40px; color: #999;">\n          <div style="font-size: 48px; margin-bottom: 10px;"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg></div>\n          <div>暂无分类</div>\n        </td>\n      </tr>\n    ')
}

function updateTagsTable(e) {
    const a = document.querySelector("#tags tbody");
    a && (a.innerHTML = "", 0 !== e.length ? (e.forEach(e => {
        var t = document.createElement("tr");
        t.innerHTML = `
      <td>
        <input type="checkbox" class="tag-checkbox" data-id="${e.id}">
      </td>
      <td>${e.sort_order}</td>
      <td><span style="display: inline-block; width: 20px; height: 20px; background-color: ${e.color}; border-radius: 4px;"></span></td>
      <td>${e.name}</td>
      <td>${e.description||"-"}</td>
      <td>${0===e.category_id?"无分类":e.category_id}</td>
      <td><span style="color: ${e.is_enabled?"#00b894":"#e74c3c"};">${e.is_enabled?"启用":"禁用"}</span></td>
      <td class="action-buttons">
        <button class="btn btn-sm btn-edit" data-action="edit-tag" data-id="${e.id}">编辑</button>
        <button class="btn btn-sm btn-delete" data-action="delete-tag" data-id="${e.id}">删除</button>
      </td>
    `, a.appendChild(t)
    }), bindActionButtons(), bindTagCheckboxes()) : a.innerHTML = '\n      <tr>\n        <td colspan="8" style="text-align: center; padding: 40px; color: #999;">\n          <div style="font-size: 48px; margin-bottom: 10px;"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg></div>\n          <div>暂无标签</div>\n        </td>\n      </tr>\n    ')
}
async function loadCategoriesAndTags() {
    var e = await fetchCategories();
    updateCategoriesTable(e);
    const a = document.getElementById("tagCategoryFilter"),
        n = (a && (a.innerHTML = '<option value="">全部标签</option>', e.forEach(e => {
            var t = document.createElement("option");
            t.value = e.id, t.textContent = e.name, a.appendChild(t)
        })), document.getElementById("tagCategory"));
    n && (n.innerHTML = '<option value="0">无分类</option>', e.forEach(e => {
        var t = document.createElement("option");
        t.value = e.id, t.textContent = e.name, n.appendChild(t)
    })), updateTagsTable(await fetchTags())
}
async function populateCategorySelect(e, a = "") {
    const n = document.getElementById(e);
    var t;
    n && (e = await fetchCategories(), n.innerHTML = "", (t = document.createElement("option")).value = "", t.textContent = "未分类", "" !== a && "未分类" !== a || (t.selected = !0), n.appendChild(t), 0 < e.length) && e.forEach(e => {
        var t = document.createElement("option");
        t.value = e.name, t.textContent = e.name, e.icon && (t.textContent = e.icon + " " + e.name), e.name === a && (t.selected = !0), n.appendChild(t)
    })
}
async function populateTagSelector(a = []) {
    const n = document.getElementById("editTagSelector");
    var e;
    n && (e = await fetchTags(), n.innerHTML = "", 0 === e.length ? n.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无标签</div>' : (e.forEach(e => {
        var t;
        e.is_enabled && ((t = document.createElement("div")).className = "tag-option", t.dataset.tagId = e.id, t.dataset.tagName = e.name, t.innerHTML = `
      <span style="display: inline-block; width: 12px; height: 12px; background-color: ${e.color}; border-radius: 2px; margin-right: 6px;"></span>
      ${e.name}
    `, a.includes(e.name) && t.classList.add("selected"), t.addEventListener("click", function() {
            this.classList.toggle("selected"), updateSelectedTags()
        }), n.appendChild(t))
    }), (e = document.getElementById("editTags")) && (e.value = JSON.stringify(a))))
}

function updateSelectedTags() {
    const t = [];
    document.querySelectorAll("#editTagSelector .tag-option.selected").forEach(e => {
        t.push(e.dataset.tagName)
    });
    var e = document.getElementById("editTags");
    e && (e.value = JSON.stringify(t))
}
async function populateTagSelectorForNewArticle() {
    const a = document.getElementById("articleTagSelector");
    var e;
    a && (e = await fetchTags(), a.innerHTML = "", 0 !== e.length ? (e.forEach(e => {
        var t;
        e.is_enabled && ((t = document.createElement("div")).className = "tag-option", t.dataset.tagId = e.id, t.dataset.tagName = e.name, t.innerHTML = `
      <span style="display: inline-block; width: 12px; height: 12px; background-color: ${e.color}; border-radius: 2px; margin-right: 6px;"></span>
      ${e.name}
    `, t.addEventListener("click", function() {
            this.classList.toggle("selected"), updateSelectedTagsForNewArticle()
        }), a.appendChild(t))
    }), updateSelectedTagsForNewArticle()) : a.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无标签</div>')
}

function updateSelectedTagsForNewArticle() {
    const t = [];
    document.querySelectorAll("#articleTagSelector .tag-option.selected").forEach(e => {
        t.push(e.dataset.tagName)
    });
    var e = document.getElementById("articleTags");
    e && (e.value = JSON.stringify(t))
}
async function populateTagSelectorForUpload() {
    const a = document.getElementById("uploadTagSelector");
    var e;
    a && (e = await fetchTags(), a.innerHTML = "", 0 !== e.length ? e.forEach(e => {
        var t;
        e.is_enabled && ((t = document.createElement("div")).className = "tag-option", t.dataset.tagId = e.id, t.dataset.tagName = e.name, t.innerHTML = `
      <span style="display: inline-block; width: 12px; height: 12px; background-color: ${e.color}; border-radius: 2px; margin-right: 6px;"></span>
      ${e.name}
    `, t.addEventListener("click", function() {
            this.classList.toggle("selected")
        }), a.appendChild(t))
    }) : a.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无标签</div>')
}
async function populateCategorySelectorForUpload() {
    const a = document.getElementById("uploadCategorySelector");
    var e;
    a && (e = await fetchCategories(), a.innerHTML = "", 0 !== e.length ? e.forEach(e => {
        var t;
        e.is_enabled && ((t = document.createElement("div")).className = "category-option", t.dataset.categoryId = e.id, t.dataset.categoryName = e.name, t.innerHTML = `
      <span style="display: inline-block; width: 12px; height: 12px; background-color: #007bff; border-radius: 2px; margin-right: 6px;"></span>
      ${e.name}
    `, t.addEventListener("click", function() {
            this.classList.contains("selected") ? this.classList.remove("selected") : (a.querySelectorAll(".category-option").forEach(e => {
                e.classList.remove("selected")
            }), this.classList.add("selected"))
        }), a.appendChild(t))
    }) : a.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无分类</div>')
}
async function populateCategorySelectorForNewArticle() {
    const a = document.getElementById("articleCategory");
    if (a) {
        const e = await fetchCategories();
        if (a.innerHTML = "", 0 === e.length) {
            const e = document.createElement("option");
            e.value = "", e.textContent = "暂无分类", void a.appendChild(e)
        } else e.forEach(e => {
            var t = document.createElement("option");
            t.value = e.name, t.textContent = e.name, a.appendChild(t)
        })
    }
}
async function fetchAdminData(e = 1, t = 10, a = 1, n = 10, o = 1, c = 10) {
    try {
        var d = localStorage.getItem("auth_token"),
            s = {
                "Content-Type": "application/json"
            };
        d && (s.Authorization = "Bearer " + d);
        var r = await (await fetch(`/api/admin/passages?page=${e}&limit=` + t, {
            headers: s
        })).json();
        r.success && r.data && 0 < r.data.length ? (updateArticlesTable(r.data), updateStatCard("totalArticles", r.pagination?.total || r.data.length), updatePagination(r.pagination)) : (showEmptyState("articlesTableBody", "暂无文章"), updateStatCard("totalArticles", 0), hidePagination());
        var i = await (await fetch(`/api/admin/users?page=${a}&limit=` + n, {
            headers: s
        })).json();
        i.success && i.data && 0 < i.data.length ? (updateUsersTable(i.data), updateStatCard("totalUsers", i.pagination?.total || i.data.length), updateUserPagination(i.pagination)) : (showEmptyState("usersTableBody", "暂无用户", 7), updateStatCard("totalUsers", 0), hideUserPagination());
        var l = await (await fetch(`/api/admin/comments?page=${o}&limit=` + c, {
            headers: s
        })).json();
        l.success && l.data && 0 < l.data.length ? (updateCommentsTable(l.data), updateCommentsPagination(l.pagination)) : (showEmptyState("commentsTableBody", "暂无评论", 6), hideCommentsPagination());
        var m = await (await fetch("/api/admin/stats", {
            headers: s
        })).json();
        if (console.log("统计数据响应:", m), m.success && m.data) {
            updateStatCard("todayVisits", m.data.today_visits || 0);
            const e = document.querySelector("#todayVisits").closest(".stat-card").querySelector(".stat-change");
            if (e && void 0 !== m.data.yesterday_visits) {
                m.data.yesterday_visits;
                const t = m.data.visits_change_percent || 0,
                    a = m.data.visits_trend || "stable";
                "up" === a ? (e.className = "stat-change positive", e.textContent = `较昨日 +${t.toFixed(1)}%`) : "down" === a ? (e.className = "stat-change negative", e.textContent = `较昨日 ${t.toFixed(1)}%`) : (e.className = "stat-change neutral", e.textContent = "较昨日持平")
            }
        } else console.error("获取统计数据失败:", m)
    } catch (e) {
        console.error("获取管理数据失败:", e)
    }
}

function showEmptyState(t, a, n = 6) {
    t = document.querySelector("#" + t);
    if (t) {
        var o = localStorage.getItem("auth_token");
        let e = o ? a : "请先登录以查看数据";
        t.innerHTML = `
    <tr>
      <td colspan="${n}" style="text-align: center; padding: 40px; color: #999;">
        <div style="font-size: 48px; margin-bottom: 10px;"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg></div>
        <div>${e}</div>
        ${o?"":'<div style="margin-top: 10px; font-size: 0.9em;"><a href="#loginModal" onclick="document.getElementById(\'loginBtn\').click(); return false;" style="color: #007bff; text-decoration: none;">点击登录</a></div>'}
      </td>
    </tr>
  `
    }
}
window.showToast = function(t, a = "success") {
    var n = document.getElementById("toastContainer");
    if (n) {
        const o = document.createElement("div");
        o.className = "toast " + a;
        let e = "";
        switch (a) {
            case "success":
            default:
                e = "⭕";
                break;
            case "error":
                e = "❌";
                break;
            case "warning":
                e = "⚠️"
        }
        o.innerHTML = `
    <div class="toast-icon">${e}</div>
    <div class="toast-message">${t}</div>
    <button class="toast-close" onclick="this.parentElement.remove()">×</button>
  `, n.appendChild(o), setTimeout(() => {
            o.parentElement && o.remove()
        }, 3e3)
    } else console.error("Toast container not found")
};
let currentPage = 1,
    currentLimit = 100,
    totalPages = 1;

function updatePagination(e) {
    var t, a, n, o;
    e ? (t = parseInt(e.page) || 1, a = parseInt(e.limit) || 100, e = parseInt(e.total) || 0, currentPage = t, currentLimit = a, totalPages = Math.ceil(e / a), (o = document.getElementById("paginationContainer")) && (o.style.display = "flex"), o = (t - 1) * a + 1, a = Math.min(t * a, e), (n = document.getElementById("paginationInfo")) && (n.textContent = `显示 ${o}-${a} 条，共 ${e} 条`), n = document.getElementById("prevPageBtn"), o = document.getElementById("nextPageBtn"), n && (n.disabled = t <= 1), o && (o.disabled = t >= totalPages), generatePageButtons(t, totalPages)) : hidePagination()
}

function hidePagination() {
    var e = document.getElementById("paginationContainer");
    e && (e.style.display = "none")
}

function generatePageButtons(n, e) {
    var o = document.getElementById("paginationPages");
    if (o) {
        o.innerHTML = "";
        let t = Math.max(1, n - 2),
            a = Math.min(e, n + 2);
        a - t < 4 && (1 === t ? a = Math.min(e, 5) : a === e && (t = Math.max(1, e - 4))), 1 < t && (addPageButton(1, o), 2 < t) && addEllipsis(o);
        for (let e = t; e <= a; e++) addPageButton(e, o, e === n);
        a < e && (a < e - 1 && addEllipsis(o), addPageButton(e, o))
    }
}

function addPageButton(e, t, a = !1) {
    var n = document.createElement("button");
    n.className = "pagination-page " + (a ? "active" : ""), n.textContent = e, n.addEventListener("click", () => {
        e !== currentPage && fetchAdminData(e, currentLimit)
    }), t.appendChild(n)
}

function addEllipsis(e) {
    var t = document.createElement("span");
    t.className = "pagination-page ellipsis", t.textContent = "...", e.appendChild(t)
}

function goToPrevPage() {
    1 < currentPage && fetchAdminData(currentPage - 1, currentLimit)
}

function goToNextPage() {
    currentPage < totalPages && fetchAdminData(currentPage + 1, currentLimit)
}

function getUsernameFromToken() {
    var e = localStorage.getItem("auth_token");
    if (!e) return "管理员";
    try {
        var t, a, n = e.split(".");
        return 3 !== n.length ? "管理员" : (t = n[1].replace(/-/g, "+").replace(/_/g, "/"), a = atob(t), JSON.parse(a).username || "管理员")
    } catch (e) {
        return console.error("解析token失败:", e), "管理员"
    }
}
async function updateWelcomeMessage() {
    const e = getUsernameFromToken(),
        t = document.querySelector(".welcome-text h2");
    t && (t.textContent = "欢迎回来，" + e);
    let a = "/img/avatar.webp";
    try {
        const e = localStorage.getItem("auth_token"),
            t = {
                "Content-Type": "application/json"
            },
            n = (e && (t.Authorization = "Bearer " + e), await fetch("/api/settings/template", {
                method: "GET",
                headers: t
            }));
        if (n.ok) {
            const e = await n.json();
            e.global_avatar && (a = e.global_avatar)
        }
    } catch (e) {
        console.error("获取全局头像设置失败:", e)
    }
    const n = document.querySelector(".admin-avatar");
    n && (n.innerHTML = `<img src="${a}" alt="${e}" class="avatar-image">`)
}
document.addEventListener("DOMContentLoaded", function() {
    updateWelcomeMessage();
    var e = document.getElementById("prevPageBtn"),
        t = document.getElementById("nextPageBtn"),
        a = document.getElementById("refreshArticlesBtn"),
        e = (e && e.addEventListener("click", goToPrevPage), t && t.addEventListener("click", goToNextPage), a && a.addEventListener("click", () => {
            fetchAdminData(currentPage, currentLimit)
        }), document.getElementById("prevUserPageBtn")),
        t = document.getElementById("nextUserPageBtn"),
        a = (e && e.addEventListener("click", goToPrevUserPage), t && t.addEventListener("click", goToNextUserPage), document.getElementById("prevCommentsPageBtn")),
        e = document.getElementById("nextCommentsPageBtn"),
        t = document.getElementById("refreshCommentsBtn"),
        a = (a && a.addEventListener("click", goToPrevCommentsPage), e && e.addEventListener("click", goToNextCommentsPage), t && t.addEventListener("click", () => {
            fetchAdminData(currentPage, currentLimit, currentUserPage, currentUserLimit, currentCommentsPage, currentCommentsLimit)
        }), document.getElementById("batchDeleteCommentsBtn")),
        e = (a && a.addEventListener("click", batchDeleteComments), document.getElementById("addCategoryBtn")),
        t = document.getElementById("refreshCategoriesBtn"),
        a = (e && e.addEventListener("click", () => {
            document.getElementById("categoryModalTitle").textContent = "添加分类", document.getElementById("categoryForm").reset(), delete document.getElementById("categoryForm").dataset.categoryId, openModal("categoryModal")
        }), t && t.addEventListener("click", loadCategoriesAndTags), document.getElementById("batchDeleteCategoriesBtn")),
        e = (a && a.addEventListener("click", batchDeleteCategories), document.getElementById("addTagBtn")),
        t = document.getElementById("refreshTagsBtn"),
        a = document.getElementById("tagCategoryFilter"),
        e = (e && e.addEventListener("click", async () => {
            await loadCategoriesAndTags(), document.getElementById("tagModalTitle").textContent = "添加标签", document.getElementById("tagForm").reset(), delete document.getElementById("tagForm").dataset.tagId, openModal("tagModal")
        }), t && t.addEventListener("click", loadCategoriesAndTags), document.getElementById("batchDeleteTagsBtn"));
    e && e.addEventListener("click", batchDeleteTags), a && a.addEventListener("change", async function() {
        updateTagsTable(await fetchTags(this.value))
    }), document.getElementById("confirmAction").addEventListener("click", async function() {
        console.log('confirmAction clicked', { currentAction: window.currentAction, currentItemId: window.currentItemId });
        if (window.currentAction && window.currentItemId)
            if (window.currentAction.startsWith("batch-delete-")) await handleBatchDelete(window.currentAction, window.currentItemId);
            else {
                var e = localStorage.getItem("auth_token"),
                    t = {
                        "Content-Type": "application/json"
                    };
                e && (t.Authorization = "Bearer " + e);
                try {
                    let e = "";
                    if ("delete" === window.currentAction) e = "/api/admin/passages?id=" + window.currentItemId;
                    else if ("delete-comment" === window.currentAction) e = "/api/admin/comments/" + window.currentItemId;
                    else if ("delete-user" === window.currentAction) e = "/api/admin/users/" + window.currentItemId;
                    else if ("delete-category" === window.currentAction) e = "/api/admin/categories/" + window.currentItemId;
                    else if ("delete-tag" === window.currentAction) e = "/api/admin/tags/" + window.currentItemId;
                    else if ("delete-main-card" === window.currentAction) e = "/api/about/main-cards/delete?id=" + window.currentItemId;
                    else if ("delete-sub-card" === window.currentAction) e = "/api/about/sub-cards/delete?id=" + window.currentItemId;
                    else if ("delete-attachment" === window.currentAction) {
                        console.log('delete-attachment: sending DELETE request to', `/api/admin/attachments/${window.currentItemId}`);
                        e = "/api/admin/attachments/" + window.currentItemId;
                    }
                    else if ("batch-delete-attachment" === window.currentAction) {
                        var a = window.currentItemId.split(",");
                        let e = 0,
                            t = 0;
                        for (const o of a) try {
                            (await (await fetch("/api/admin/attachments/" + o, {
                                method: "DELETE"
                            })).json()).success ? e++ : t++
                        } catch (e) {
                            console.error("删除失败:", e), t++
                        }
                        return closeModal("confirmModal"), 0 < e && showToast(`成功删除 ${e} 个附件`, "success"), 0 < t && showToast(t + " 个附件删除失败", "error"), selectedAttachments.clear(), updateBatchActions(), void loadAttachments()
                    }
                    var n = await (await fetch(e, {
                        method: "DELETE",
                        headers: t
                    })).json();
                    console.log('DELETE response', n);
                    if (n.success) {
                        closeModal("confirmModal");
                        showToast("删除成功！", "success");
                        if ("delete-category" === window.currentAction || "delete-tag" === window.currentAction) {
                            loadCategoriesAndTags();
                        } else if ("delete-main-card" === window.currentAction) {
                            loadMainCards();
                            loadSubCards();
                        } else if ("delete-sub-card" === window.currentAction) {
                            loadSubCards();
                        } else if ("delete-attachment" === window.currentAction) {
                            console.log('delete-attachment success, reloading...');
                            selectedAttachments.delete(window.currentItemId);
                            updateBatchActions();
                            loadAttachments();
                        } else {
                            fetchAdminData();
                        }
                    } else {
                        console.error('DELETE failed', n);
                        showToast("删除失败：" + (n.message || "未知错误"), "error");
                    }
                } catch (e) {
                    console.error("删除失败:", e), alert("删除失败，请稍后重试")
                }
            }
    })
});
let currentUserPage = 1,
    currentUserLimit = 10,
    totalUserPages = 1;

function updateUserPagination(e) {
    var t, a, n, o;
    e ? (t = parseInt(e.page) || 1, a = parseInt(e.limit) || 100, e = parseInt(e.total) || 0, currentUserPage = t, currentUserLimit = a, totalUserPages = Math.ceil(e / a), (o = document.getElementById("userPaginationContainer")) && (o.style.display = "flex"), o = (t - 1) * a + 1, a = Math.min(t * a, e), (n = document.getElementById("userPaginationInfo")) && (n.textContent = `显示 ${o}-${a} 条，共 ${e} 条`), n = document.getElementById("prevUserPageBtn"), o = document.getElementById("nextUserPageBtn"), n && (n.disabled = t <= 1), o && (o.disabled = t >= totalUserPages), generateUserPageButtons(t, totalUserPages)) : hideUserPagination()
}

function hideUserPagination() {
    var e = document.getElementById("userPaginationContainer");
    e && (e.style.display = "none")
}

function generateUserPageButtons(n, e) {
    var o = document.getElementById("userPaginationPages");
    if (o) {
        o.innerHTML = "";
        let t = Math.max(1, n - 2),
            a = Math.min(e, n + 2);
        a - t < 4 && (1 === t ? a = Math.min(e, 5) : a === e && (t = Math.max(1, e - 4))), 1 < t && (addUserPageButton(1, o), 2 < t) && addUserEllipsis(o);
        for (let e = t; e <= a; e++) addUserPageButton(e, o, e === n);
        a < e && (a < e - 1 && addUserEllipsis(o), addUserPageButton(e, o))
    }
}

function addUserPageButton(e, t, a = !1) {
    var n = document.createElement("button");
    n.className = "pagination-page " + (a ? "active" : ""), n.textContent = e, n.addEventListener("click", () => {
        e !== currentUserPage && fetchAdminData(currentPage, currentLimit, e, currentUserLimit)
    }), t.appendChild(n)
}

function addUserEllipsis(e) {
    var t = document.createElement("span");
    t.className = "pagination-page ellipsis", t.textContent = "...", e.appendChild(t)
}

function goToPrevUserPage() {
    1 < currentUserPage && fetchAdminData(currentPage, currentLimit, currentUserPage - 1, currentUserLimit)
}

function goToNextUserPage() {
    currentUserPage < totalUserPages && fetchAdminData(currentPage, currentLimit, currentUserPage + 1, currentUserLimit)
}
let currentCommentsPage = 1,
    currentCommentsLimit = 10,
    totalCommentsPages = 1;

function updateCommentsTable(e) {
    const n = document.querySelector("#comments tbody");
    n && (n.innerHTML = "", e.forEach(e => {
        var t = document.createElement("tr"),
            a = generateAdminIdenticon(e.username || "anonymous", 24);
        t.innerHTML = `
      <td>
        <input type="checkbox" class="comment-checkbox" data-id="${e.id}">
      </td>
      <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${e.content}</td>
      <td>${e.passage_uuid}</td>
      <td>
        <div style="display: flex; align-items: center; gap: 8px;">
          <img src="${a}" alt="${e.username||"匿名用户"}" style="width: 24px; height: 24px; border-radius: 50%;"/>
          <span>${e.username}</span>
        </div>
      </td>
      <td>${e.created_at}</td>
      <td class="action-buttons">
        <button class="btn btn-sm btn-delete" data-action="delete-comment" data-id="${e.id}">删除</button>
      </td>
    `, n.appendChild(t)
    }), bindActionButtons(), bindCommentCheckboxes())
}

function generateAdminIdenticon(e, t = 24) {
    var a = simpleAdminHash(e),
        n = a % 360,
        o = 65 + a % 15,
        c = 55 + a % 10,
        d = `hsl(${n}, ${o}%, ${c}%)`,
        n = `hsl(${n}, ${o}%, ${c-25}%)`;
    let s = "";
    for (let t = 0; t < 5; t++)
        for (let e = 0; e < Math.ceil(2.5); e++) a >> 5 * t + e & 1 && (s = (s += `<rect x="${e}" y="${t}" width="1" height="1" fill="${d}"/>`) + `<rect x="${4-e}" y="${t}" width="1" height="1" fill="${d}"/>`);
    o = e ? e.charAt(0).toUpperCase() : "?";
    return "data:image/svg+xml;base64," + btoa(unescape(encodeURIComponent(`
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5 5" width="${t}" height="${t}">
      <rect width="5" height="5" fill="#f0f0f0"/>
      ${s}
      <text x="2.5" y="2.85" 
            text-anchor="middle" 
            font-size="2.5" 
            font-weight="bold" 
            fill="${n}"
            font-family="system-ui, -apple-system, sans-serif">${o}</text>
    </svg>
  `)))
}

function simpleAdminHash(t) {
    let a = 0;
    for (let e = 0; e < t.length; e++) a = (a << 5) - a + t.charCodeAt(e), a &= a;
    return Math.abs(a)
}

function bindCommentCheckboxes() {
    const e = document.getElementById("selectAllComments"),
        t = document.querySelectorAll(".comment-checkbox");
    e && e.addEventListener("change", function() {
        t.forEach(e => {
            e.checked = this.checked;
            e = e.closest("tr");
            e && (this.checked ? e.classList.add("selected") : e.classList.remove("selected"))
        }), updateBatchDeleteButton()
    }), t.forEach(e => {
        e.addEventListener("change", function() {
            var e = this.closest("tr");
            e && (this.checked ? e.classList.add("selected") : e.classList.remove("selected")), updateBatchDeleteButton(), updateSelectAllCheckbox()
        })
    })
}

function updateBatchDeleteButton() {
    var e = document.querySelectorAll(".comment-checkbox:checked").length,
        t = document.getElementById("batchDeleteCommentsBtn");
    t && (0 < e ? (t.style.display = "inline-block", t.textContent = `批量删除 (${e})`) : t.style.display = "none")
}

function updateSelectAllCheckbox() {
    var e, t = document.getElementById("selectAllComments"),
        a = document.querySelectorAll(".comment-checkbox");
    t && 0 < a.length && (e = Array.from(a).every(e => e.checked), a = Array.from(a).some(e => e.checked), t.checked = e, t.indeterminate = a && !e)
}

function clearCommentSelection() {
    var e = document.getElementById("selectAllComments"),
        t = document.querySelectorAll(".comment-checkbox");
    e && (e.checked = !1, e.indeterminate = !1), t.forEach(e => {
        e.checked = !1;
        e = e.closest("tr");
        e && e.classList.remove("selected")
    }), updateBatchDeleteButton()
}
async function batchDeleteComments() {
    var e = document.querySelectorAll(".comment-checkbox:checked"),
        e = Array.from(e).map(e => parseInt(e.dataset.id));
    0 !== e.length ? (window.currentAction = "batch-delete-comments", window.currentItemId = e.join(","), document.getElementById("confirmMessage").textContent = `确定要删除选中的 ${e.length} 条评论吗？此操作不可恢复！`, openModal("confirmModal")) : showToast("请选择要删除的评论", "warning")
}

function bindCategoryCheckboxes() {
    const e = document.getElementById("selectAllCategories"),
        t = document.querySelectorAll(".category-checkbox");
    e && e.addEventListener("change", function() {
        t.forEach(e => {
            e.checked = this.checked;
            e = e.closest("tr");
            e && (this.checked ? e.classList.add("selected") : e.classList.remove("selected"))
        }), updateBatchDeleteCategoriesButton()
    }), t.forEach(e => {
        e.addEventListener("change", function() {
            var e = this.closest("tr");
            e && (this.checked ? e.classList.add("selected") : e.classList.remove("selected")), updateBatchDeleteCategoriesButton(), updateSelectAllCategoriesCheckbox()
        })
    })
}

function updateBatchDeleteCategoriesButton() {
    var e = document.querySelectorAll(".category-checkbox:checked").length,
        t = document.getElementById("batchDeleteCategoriesBtn");
    t && (0 < e ? (t.style.display = "inline-block", t.textContent = `批量删除 (${e})`) : t.style.display = "none")
}

function updateSelectAllCategoriesCheckbox() {
    var e, t = document.getElementById("selectAllCategories"),
        a = document.querySelectorAll(".category-checkbox");
    t && 0 < a.length && (e = Array.from(a).every(e => e.checked), a = Array.from(a).some(e => e.checked), t.checked = e, t.indeterminate = a && !e)
}

function clearCategorySelection() {
    var e = document.getElementById("selectAllCategories"),
        t = document.querySelectorAll(".category-checkbox");
    e && (e.checked = !1, e.indeterminate = !1), t.forEach(e => {
        e.checked = !1;
        e = e.closest("tr");
        e && e.classList.remove("selected")
    }), updateBatchDeleteCategoriesButton()
}
async function batchDeleteCategories() {
    var e = document.querySelectorAll(".category-checkbox:checked"),
        e = Array.from(e).map(e => parseInt(e.dataset.id));
    0 !== e.length ? (window.currentAction = "batch-delete-categories", window.currentItemId = e.join(","), document.getElementById("confirmMessage").textContent = `确定要删除选中的 ${e.length} 个分类吗？此操作不可恢复！`, openModal("confirmModal")) : showToast("请选择要删除的分类", "warning")
}

function bindTagCheckboxes() {
    const e = document.getElementById("selectAllTags"),
        t = document.querySelectorAll(".tag-checkbox");
    e && e.addEventListener("change", function() {
        t.forEach(e => {
            e.checked = this.checked;
            e = e.closest("tr");
            e && (this.checked ? e.classList.add("selected") : e.classList.remove("selected"))
        }), updateBatchDeleteTagsButton()
    }), t.forEach(e => {
        e.addEventListener("change", function() {
            var e = this.closest("tr");
            e && (this.checked ? e.classList.add("selected") : e.classList.remove("selected")), updateBatchDeleteTagsButton(), updateSelectAllTagsCheckbox()
        })
    })
}

function updateBatchDeleteTagsButton() {
    var e = document.querySelectorAll(".tag-checkbox:checked").length,
        t = document.getElementById("batchDeleteTagsBtn");
    t && (0 < e ? (t.style.display = "inline-block", t.textContent = `批量删除 (${e})`) : t.style.display = "none")
}

function updateSelectAllTagsCheckbox() {
    var e, t = document.getElementById("selectAllTags"),
        a = document.querySelectorAll(".tag-checkbox");
    t && 0 < a.length && (e = Array.from(a).every(e => e.checked), a = Array.from(a).some(e => e.checked), t.checked = e, t.indeterminate = a && !e)
}

function clearTagSelection() {
    var e = document.getElementById("selectAllTags"),
        t = document.querySelectorAll(".tag-checkbox");
    e && (e.checked = !1, e.indeterminate = !1), t.forEach(e => {
        e.checked = !1;
        e = e.closest("tr");
        e && e.classList.remove("selected")
    }), updateBatchDeleteTagsButton()
}
async function batchDeleteTags() {
    var e = document.querySelectorAll(".tag-checkbox:checked"),
        e = Array.from(e).map(e => parseInt(e.dataset.id));
    0 !== e.length ? (window.currentAction = "batch-delete-tags", window.currentItemId = e.join(","), document.getElementById("confirmMessage").textContent = `确定要删除选中的 ${e.length} 个标签吗？此操作不可恢复！`, openModal("confirmModal")) : showToast("请选择要删除的标签", "warning")
}

function updateCommentsPagination(e) {
    var t, a, n, o;
    e ? (t = parseInt(e.page) || 1, a = parseInt(e.limit) || 100, e = parseInt(e.total) || 0, currentCommentsPage = t, currentCommentsLimit = a, totalCommentsPages = Math.ceil(e / a), (o = document.getElementById("commentsPaginationContainer")) && (o.style.display = "flex"), o = (t - 1) * a + 1, a = Math.min(t * a, e), (n = document.getElementById("commentsPaginationInfo")) && (n.textContent = `显示 ${o}-${a} 条，共 ${e} 条`), n = document.getElementById("prevCommentsPageBtn"), o = document.getElementById("nextCommentsPageBtn"), n && (n.disabled = t <= 1), o && (o.disabled = t >= totalCommentsPages), generateCommentsPageButtons(t, totalCommentsPages)) : hideCommentsPagination()
}

function hideCommentsPagination() {
    var e = document.getElementById("commentsPaginationContainer");
    e && (e.style.display = "none")
}

function generateCommentsPageButtons(n, e) {
    var o = document.getElementById("commentsPaginationPages");
    if (o) {
        o.innerHTML = "";
        let t = Math.max(1, n - 2),
            a = Math.min(e, n + 2);
        a - t < 4 && (1 === t ? a = Math.min(e, 5) : a === e && (t = Math.max(1, e - 4))), 1 < t && (addCommentsPageButton(1, o), 2 < t) && addCommentsEllipsis(o);
        for (let e = t; e <= a; e++) addCommentsPageButton(e, o, e === n);
        a < e && (a < e - 1 && addCommentsEllipsis(o), addCommentsPageButton(e, o))
    }
}

function addCommentsPageButton(e, t, a = !1) {
    var n = document.createElement("button");
    n.className = "pagination-page " + (a ? "active" : ""), n.textContent = e, n.addEventListener("click", () => {
        e !== currentCommentsPage && fetchAdminData(currentPage, currentLimit, currentUserPage, currentUserLimit, e, currentCommentsLimit)
    }), t.appendChild(n)
}

function addCommentsEllipsis(e) {
    var t = document.createElement("span");
    t.className = "pagination-page ellipsis", t.textContent = "...", e.appendChild(t)
}

function goToPrevCommentsPage() {
    1 < currentCommentsPage && fetchAdminData(currentPage, currentLimit, currentUserPage, currentUserLimit, currentCommentsPage - 1, currentCommentsLimit)
}

function goToNextCommentsPage() {
    currentCommentsPage < totalCommentsPages && fetchAdminData(currentPage, currentLimit, currentUserPage, currentUserLimit, currentCommentsPage + 1, currentCommentsLimit)
}

function updateArticlesTable(e) {
    const c = document.querySelector("#articles tbody");
    c && (c.innerHTML = "", e.forEach(e => {
        const t = document.createElement("tr");
        t.dataset.articleId = e.id;
        let a = `<span class="status ${e.status||"published"}">${getStatusText(e.status||"published")}</span>`;
        e.is_scheduled && (a += ` <span class="status scheduled" title="定时发布">⏰ ${e.published_at?formatDate(e.published_at):"未设置"}</span>`);
        var n = e.visibility || "public",
            o = "public" === n ? "公开" : "私密",
            n = "public" === n ? "visibility-public" : "visibility-private";
        t.innerHTML = `
      <td>
        <input type="checkbox" class="article-checkbox" data-id="${e.id}">
      </td>
      <td>${e.title}</td>
      <td>管理员</td>
      <td>${formatDate(e.created_at)||"2024-01-01"}</td>
      <td>${a}</td>
      <td><span class="visibility ${n}">${o}</span></td>
      <td class="action-buttons">
        <button class="btn btn-sm btn-view" data-action="view" data-id="${e.id}">查看</button>
        <button class="btn btn-sm btn-edit" data-action="edit" data-id="${e.id}">编辑</button>
        <button class="btn btn-sm btn-upload" data-action="upload" data-id="${e.id}">上传附件</button>
        <button class="btn btn-sm btn-delete" data-action="delete" data-id="${e.id}">删除</button>
      </td>
    `, t.addEventListener("click", e => {
            e.target.closest(".action-buttons") || e.target.classList.contains("article-checkbox") || ((e = t.querySelector(".article-checkbox")).checked = !e.checked, t.classList.toggle("selected", e.checked), updateBatchActionsBar())
        }), t.querySelector(".article-checkbox").addEventListener("change", e => {
            t.classList.toggle("selected", e.target.checked), updateBatchActionsBar()
        }), c.appendChild(t)
    }), bindActionButtons(), bindBatchActionEvents())
}

function bindBatchActionEvents() {
    var e = document.getElementById("selectAllCheckbox"),
        t = document.getElementById("batchDeleteBtn"),
        a = document.getElementById("clearSelectionBtn");
    e && e.addEventListener("change", t => {
        document.querySelectorAll(".article-checkbox").forEach(e => {
            e.checked = t.target.checked;
            e = e.closest("tr");
            e && e.classList.toggle("selected", t.target.checked)
        }), updateBatchActionsBar()
    }), t && t.addEventListener("click", batchDeleteArticles), a && a.addEventListener("click", clearSelection)
}

function updateBatchActionsBar() {
    const e = document.querySelectorAll(".article-checkbox:checked"),
        t = document.getElementById("batchActionsBar"),
        a = document.getElementById("selectedCount"),
        n = document.getElementById("selectAllCheckbox"),
        o = e.length;
    if (0 < o ? (t.style.display = "flex", a.textContent = `已选择 ${o} 篇文章`) : t.style.display = "none", n) {
        const e = document.querySelectorAll(".article-checkbox");
        0 < e.length && o === e.length ? (n.checked = !0, n.indeterminate = !1) : 0 < o ? (n.checked = !1, n.indeterminate = !0) : (n.checked = !1, n.indeterminate = !1)
    }
}
async function batchDeleteArticles() {
    var e = document.querySelectorAll(".article-checkbox:checked"),
        e = Array.from(e).map(e => parseInt(e.dataset.id));
    0 !== e.length ? (window.currentAction = "batch-delete-articles", window.currentItemId = e.join(","), document.getElementById("confirmMessage").textContent = `确定要删除选中的 ${e.length} 篇文章吗？此操作不可恢复！`, openModal("confirmModal")) : showToast("请选择要删除的文章", "warning")
}

function clearSelection() {
    document.querySelectorAll(".article-checkbox").forEach(e => {
        e.checked = !1;
        e = e.closest("tr");
        e && e.classList.remove("selected")
    });
    var e = document.getElementById("selectAllCheckbox");
    e && (e.checked = !1, e.indeterminate = !1), updateBatchActionsBar()
}

function bindUserBatchActionEvents() {
    var e = document.getElementById("selectAllUsersCheckbox"),
        t = document.getElementById("batchDeleteUsersBtn"),
        a = document.getElementById("clearUserSelectionBtn");
    e && e.addEventListener("change", t => {
        document.querySelectorAll(".user-checkbox:not(:disabled)").forEach(e => {
            e.checked = t.target.checked;
            e = e.closest("tr");
            e && e.classList.toggle("selected", t.target.checked)
        }), updateUserBatchActionsBar()
    }), t && t.addEventListener("click", batchDeleteUsers), a && a.addEventListener("click", clearUserSelection)
}

function updateUserBatchActionsBar() {
    const e = document.querySelectorAll(".user-checkbox:checked"),
        t = document.getElementById("userBatchActionsBar"),
        a = document.getElementById("userSelectedCount"),
        n = document.getElementById("selectAllUsersCheckbox"),
        o = e.length;
    if (0 < o ? (t.style.display = "flex", a.textContent = `已选择 ${o} 个用户`) : t.style.display = "none", n) {
        const e = document.querySelectorAll(".user-checkbox:not(:disabled)");
        0 < e.length && o === e.length ? (n.checked = !0, n.indeterminate = !1) : 0 < o ? (n.checked = !1, n.indeterminate = !0) : (n.checked = !1, n.indeterminate = !1)
    }
}
async function batchDeleteUsers() {
    var e = document.querySelectorAll(".user-checkbox:checked"),
        e = Array.from(e).map(e => parseInt(e.dataset.id));
    0 !== e.length ? (window.currentAction = "batch-delete-users", window.currentItemId = e.join(","), document.getElementById("confirmMessage").textContent = `确定要删除选中的 ${e.length} 个用户吗？此操作不可恢复！`, openModal("confirmModal")) : showToast("请选择要删除的用户", "warning")
}

function clearUserSelection() {
    document.querySelectorAll(".user-checkbox").forEach(e => {
        e.checked = !1;
        e = e.closest("tr");
        e && e.classList.remove("selected")
    });
    var e = document.getElementById("selectAllUsersCheckbox");
    e && (e.checked = !1, e.indeterminate = !1), updateUserBatchActionsBar()
}

function formatDate(e) {
    return e ? new Date(e).toLocaleString("zh-CN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit"
    }) : ""
}

function updateUsersTable(e) {
    const a = document.querySelector("#users tbody");
    a && (a.innerHTML = "", e.forEach(e => {
        const t = document.createElement("tr");
        t.dataset.userId = e.id, t.innerHTML = `
      <td>
        <input type="checkbox" class="user-checkbox" data-id="${e.id}" ${"admin"===e.role?"disabled":""}>
      </td>
      <td>${e.username}</td>
      <td>${e.email}</td>
      <td>${e.created_at||"2024-01-01"}</td>
      <td>${"admin"===e.role?"管理员":"editor"===e.role?"编辑":"普通用户"}</td>
      <td><span style="color:${"active"===e.status?"#00b894":"restricted"===e.status?"#fdcb6e":"#e74c3c"};">${"active"===e.status?"正常":"restricted"===e.status?"受限":"禁用"}</span></td>
      <td class="action-buttons">
        <button class="btn btn-sm btn-view" data-action="view-user" data-id="${e.id}">详情</button>
        <button class="btn btn-sm btn-edit" data-action="edit-user" data-id="${e.id}">编辑</button>
        ${"admin"!==e.role?`<button class="btn btn-sm btn-delete" data-action="delete-user" data-id="${e.id}">删除</button>`:""}
      </td>
    `, t.addEventListener("click", e => {
            e.target.closest(".action-buttons") || e.target.classList.contains("user-checkbox") || (e = t.querySelector(".user-checkbox")).disabled || (e.checked = !e.checked, t.classList.toggle("selected", e.checked), updateUserBatchActionsBar())
        }), t.querySelector(".user-checkbox").addEventListener("change", e => {
            t.classList.toggle("selected", e.target.checked), updateUserBatchActionsBar()
        }), a.appendChild(t)
    }), bindActionButtons(), bindUserBatchActionEvents())
}

function updateStatCard(e, t) {
    e = document.getElementById(e);
    e && (e.textContent = t)
}

function getAuthHeaders() {
    var e = localStorage.getItem("auth_token");
    return e ? {
        Authorization: "Bearer " + e
    } : {}
}
let viewTrendChart = null;
async function loadMostViewedArticles() {
    var e = document.getElementById("mostViewedLimit")?.value || 10,
        t = document.getElementById("mostViewedTableBody");
    if (t) {
        t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载中...</td></tr>';
        try {
            var a = await (await fetch("/api/admin/analytics?action=most-viewed&limit=" + e, {
                headers: getAuthHeaders()
            })).json();
            a.success && a.data ? 0 === a.data.length ? t.innerHTML = '<tr><td colspan="4" style="text-align: center;">暂无数据</td></tr>' : t.innerHTML = a.data.map((e, t) => `
        <tr>
          <td>${t+1}</td>
          <td>${e.title}</td>
          <td>${e.author}</td>
          <td>${e.view_count}</td>
        </tr>
      `).join("") : t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>'
        } catch (e) {
            console.error("加载热门文章失败:", e), t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>'
        }
    }
}
async function loadViewSources() {
    var e = document.getElementById("viewSourcesDays")?.value || 30,
        t = document.getElementById("viewSourcesTableBody");
    if (t) {
        t.innerHTML = '<tr><td colspan="3" style="text-align: center;">加载中...</td></tr>';
        try {
            var a = await (await fetch("/api/admin/analytics?action=view-sources&days=" + e, {
                headers: getAuthHeaders()
            })).json();
            a.success && a.data ? 0 === a.data.length ? t.innerHTML = '<tr><td colspan="3" style="text-align: center;">暂无数据</td></tr>' : t.innerHTML = a.data.map((e, t) => `
        <tr>
          <td>${t+1}</td>
          <td>${"unknown"===e.country?"未知":e.country}</td>
          <td>${e.count}</td>
        </tr>
      `).join("") : t.innerHTML = '<tr><td colspan="3" style="text-align: center;">加载失败</td></tr>'
        } catch (e) {
            console.error("加载访问来源失败:", e), t.innerHTML = '<tr><td colspan="3" style="text-align: center;">加载失败</td></tr>'
        }
    }
}
async function loadViewByCity() {
    var e = document.getElementById("viewByCityDays")?.value || 30,
        t = document.getElementById("viewByCityTableBody");
    if (t) {
        t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载中...</td></tr>';
        try {
            var a = await (await fetch("/api/admin/analytics?action=view-by-city&days=" + e, {
                headers: getAuthHeaders()
            })).json();
            a.success && a.data ? 0 === a.data.length ? t.innerHTML = '<tr><td colspan="4" style="text-align: center;">暂无数据</td></tr>' : t.innerHTML = a.data.map((e, t) => `
        <tr>
          <td>${t+1}</td>
          <td>${"unknown"===e.city?"未知":e.city}</td>
          <td>${"unknown"===e.country?"未知":e.country}</td>
          <td>${e.count}</td>
        </tr>
      `).join("") : t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>'
        } catch (e) {
            console.error("加载城市统计失败:", e), t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>'
        }
    }
}
async function loadViewByIP() {
    var e = document.getElementById("viewByIPDays")?.value || 30,
        t = document.getElementById("viewByIPTableBody");
    if (t) {
        t.innerHTML = '<tr><td colspan="8" style="text-align: center;">加载中...</td></tr>';
        try {
            var a = await (await fetch("/api/admin/analytics?action=view-by-ip&days=" + e, {
                headers: getAuthHeaders()
            })).json();
            a.success && a.data ? 0 === a.data.length ? t.innerHTML = '<tr><td colspan="8" style="text-align: center;">暂无数据</td></tr>' : t.innerHTML = a.data.map((e, t) => `
        <tr>
          <td>${t+1}</td>
          <td>${e.ip}</td>
          <td>${"unknown"===e.country?"未知":e.country||"-"}</td>
          <td>${"unknown"===e.city?"未知":e.city||"-"}</td>
          <td>${"unknown"===e.region?"未知":e.region||"-"}</td>
          <td>${e.count}</td>
          <td>${e.firstVisit}</td>
          <td>${e.lastVisit}</td>
        </tr>
      `).join("") : t.innerHTML = '<tr><td colspan="8" style="text-align: center;">加载失败</td></tr>'
        } catch (e) {
            console.error("加载IP统计失败:", e), t.innerHTML = '<tr><td colspan="8" style="text-align: center;">加载失败</td></tr>'
        }
    }
}
async function loadViewTrend() {
    var e = document.getElementById("viewTrendDays")?.value || 30;
    if (document.getElementById("viewTrendChart")) try {
        var t = await (await fetch("/api/admin/analytics?action=view-trend&days=" + e, {
            headers: getAuthHeaders()
        })).json();
        t.success && t.data && drawViewTrendChart(t.data)
    } catch (e) {
        console.error("加载阅读趋势失败:", e)
    }
}

function drawViewTrendChart(e) {
    const n = document.getElementById("viewTrendChart");
    if (n) {
        const o = n.getContext("2d");
        if (o.clearRect(0, 0, n.width, n.height), e && 0 !== e.length) {
            const t = n.width - 100,
                c = n.height - 100,
                d = Math.max(...e.map(e => e.count)),
                s = (o.strokeStyle = "rgba(255, 255, 255, 0.3)", o.lineWidth = 1, o.beginPath(), o.moveTo(50, 50), o.lineTo(50, n.height - 50), o.lineTo(n.width - 50, n.height - 50), o.stroke(), t / (e.length - 1 || 1));
            o.beginPath(), o.strokeStyle = "rgba(255, 255, 255, 0.8)", o.lineWidth = 2, e.forEach((e, t) => {
                var a = 50 + t * s,
                    e = n.height - 50 - e.count / d * c;
                0 === t ? o.moveTo(a, e) : o.lineTo(a, e)
            }), o.stroke(), e.forEach((e, t) => {
                var t = 50 + t * s,
                    a = n.height - 50 - e.count / d * c;
                o.beginPath(), o.fillStyle = "rgba(255, 255, 255, 0.9)", o.arc(t, a, 4, 0, 2 * Math.PI), o.fill(), o.fillStyle = "rgba(255, 255, 255, 0.8)", o.font = "12px Arial", o.textAlign = "center", o.fillText(e.count, t, a - 10), o.fillStyle = "rgba(255, 255, 255, 0.6)", o.font = "10px Arial", e.date && "string" == typeof e.date && o.fillText(e.date.substring(5), t, n.height - 50 + 15)
            })
        } else o.font = "16px Arial", o.fillStyle = "rgba(255, 255, 255, 0.7)", o.textAlign = "center", o.fillText("暂无数据", n.width / 2, n.height / 2)
    }
}

function initAnalytics() {
    var e = document.getElementById("mostViewedLimit"),
        e = (e && e.addEventListener("change", loadMostViewedArticles), document.getElementById("viewSourcesDays")),
        e = (e && e.addEventListener("change", loadViewSources), document.getElementById("viewByCityDays")),
        e = (e && e.addEventListener("change", loadViewByCity), document.getElementById("viewByIPDays")),
        e = (e && e.addEventListener("change", loadViewByIP), document.getElementById("viewTrendDays"));
    e && e.addEventListener("change", loadViewTrend), loadMostViewedArticles(), loadViewSources(), loadViewByCity(), loadViewByIP(), loadViewTrend()
}

function getStatusText(e) {
    return {
        published: "已发布",
        draft: "草稿",
        pending: "待审核"
    } [e] || e
}

function bindActionButtons() {
    document.querySelectorAll(".action-buttons button").forEach(e => {
        var t = e.cloneNode(!0);
        e.parentNode.replaceChild(t, e), t.addEventListener("click", async function() {
            const t = this.getAttribute("data-action"),
                a = this.getAttribute("data-id");
            if ("delete" === t || "delete-comment" === t || "delete-user" === t || "delete-category" === t || "delete-tag" === t || "delete-attachment" === t) {
                window.currentAction = t, window.currentItemId = a;
                let e = "";
                "delete" === t ? e = `确定要删除文章 #${a} 吗？此操作不可撤销。` : "delete-comment" === t ? e = `确定要删除评论 #${a} 吗？此操作不可撤销。` : "delete-user" === t ? e = `确定要删除用户 #${a} 吗？此操作不可撤销。` : "delete-category" === t ? e = `确定要删除分类 #${a} 吗？此操作不可撤销。` : "delete-tag" === t ? e = `确定要删除标签 #${a} 吗？此操作不可撤销。` : "delete-attachment" === t ? e = `确定要删除附件 #${a} 吗？此操作不可撤销。` : e = `确定要删除 #${a} 吗？此操作不可撤销。`, document.getElementById("confirmMessage").textContent = e, openModal("confirmModal")
            } else if ("edit" === t) try {
                    const t = localStorage.getItem("auth_token"),
                        i = {
                            "Content-Type": "application/json"
                        };
                    t && (i.Authorization = "Bearer " + t);
                    var n = await (await fetch("/api/admin/passages?id=" + a, {
                        headers: i
                    })).json();
                    if (n.success && n.data) {
                        await populateCategorySelect("editCategory", n.data.category || ""), document.getElementById("editTitle").value = n.data.title || "", document.getElementById("editAuthor").value = n.data.author || "管理员", document.getElementById("editContent").value = n.data.original_content || n.data.content || "", document.getElementById("editShowTitle").checked = !1 !== n.data.show_title, document.getElementById("editCoverImage").value = n.data.cover_image || "", document.getElementById("editStatus").value = n.data.status || "published", document.getElementById("editVisibility").value = n.data.visibility || "public";
                        const t = n.data.is_scheduled || !1;
                        document.getElementById("editIsScheduled").checked = t;
                        var o = document.getElementById("editPublishedAtGroup");
                        if (t) {
                            if (o.style.display = "block", n.data.published_at) {
                                const t = new Date(n.data.published_at),
                                    a = new Date(t.getTime() - 6e4 * t.getTimezoneOffset()).toISOString().slice(0, 16);
                                document.getElementById("editPublishedAt").value = a
                            }
                        } else o.style.display = "none", document.getElementById("editPublishedAt").value = "";
                        let e = [];
                        if (n.data.tags) {
                            const t = n.data.tags;
                            if (Array.isArray(t)) e = t;
                            else if ("string" == typeof t) try {
                                const a = JSON.parse(t);
                                e = Array.isArray(a) ? a : t.split(",").map(e => e.trim()).filter(e => e)
                            } catch (a) {
                                e = t.split(",").map(e => e.trim()).filter(e => e)
                            }
                        }
                        await populateTagSelector(e), document.getElementById("editForm").dataset.articleId = a, openModal("editModal")
                    } else showToast("获取文章详情失败：" + (n.message || "未知错误"), "error")
                } catch (t) {
                    console.error("获取文章详情失败:", t), showToast("获取文章详情失败，请稍后重试", "error")
                } else if ("upload" === t) document.getElementById("uploadAttachmentArticleId").textContent = "#" + a, document.getElementById("uploadAttachmentArticleId").dataset.articleId = a, document.getElementById("attachmentFile").value = "", document.getElementById("uploadAttachmentProgress").style.display = "none", document.getElementById("uploadAttachmentResult").style.display = "none", openModal("uploadAttachmentModal");
                else if ("view" === t) try {
                const t = localStorage.getItem("auth_token"),
                    l = {
                        "Content-Type": "application/json"
                    };
                t && (l.Authorization = "Bearer " + t);
                var e = await (await fetch("/api/admin/passages?id=" + a, {
                    headers: l
                })).json();
                e.success && e.data ? (document.getElementById("viewArticleId").textContent = "#" + e.data.id, document.getElementById("viewArticleTitle").textContent = e.data.title || "", document.getElementById("viewArticleAuthor").textContent = e.data.author || "管理员", document.getElementById("viewArticleStatus").textContent = getStatusText(e.data.status || "published"), document.getElementById("viewArticleDate").textContent = e.data.created_at || "", document.getElementById("viewArticleContent").textContent = e.data.content || "", openModal("viewArticleModal")) : showToast("获取文章详情失败：" + (e.message || "未知错误"), "error")
            } catch (t) {
                console.error("获取文章详情失败:", t), showToast("获取文章详情失败，请稍后重试", "error")
            } else if ("edit-user" === t) try {
                const t = localStorage.getItem("auth_token"),
                    m = {
                        "Content-Type": "application/json"
                    };
                t && (m.Authorization = "Bearer " + t);
                var c = await (await fetch("/api/admin/users/" + a, {
                    headers: m
                })).json();
                if (c.success && c.data) {
                    const t = c.data;
                    document.getElementById("editUserName").value = t.username || "", document.getElementById("editUserEmail").value = t.email || "", document.getElementById("editUserRole").value = t.role || "user", document.getElementById("editUserStatus").value = t.status || "active", document.getElementById("editUserPassword").value = "", document.getElementById("editUserForm").dataset.userId = a, openModal("editUserModal")
                } else showToast("获取用户详情失败：" + (c.message || "未知错误"), "error")
            } catch (t) {
                console.error("获取用户详情失败:", t), showToast("获取用户详情失败，请稍后重试", "error")
            } else if ("view-user" === t) try {
                    const t = localStorage.getItem("auth_token"),
                        u = {
                            "Content-Type": "application/json"
                        };
                    t && (u.Authorization = "Bearer " + t);
                    var d = await (await fetch("/api/admin/users/" + a, {
                        headers: u
                    })).json();
                    if (d.success && d.data) {
                        const t = d.data;
                        document.getElementById("viewUserId").textContent = "#" + t.id, document.getElementById("viewUserName").textContent = t.username || "", document.getElementById("viewUserEmail").textContent = t.email || "", document.getElementById("viewUserRole").textContent = t.role || "普通用户", document.getElementById("viewUserStatus").textContent = t.status || "正常", document.getElementById("viewUserDate").textContent = t.created_at || "", openModal("viewUserModal")
                    } else showToast("获取用户详情失败：" + (d.message || "未知错误"), "error")
                } catch (t) {
                    console.error("获取用户详情失败:", t), showToast("获取用户详情失败，请稍后重试", "error")
                } else if ("edit-comment" === t) alert("编辑评论 #" + a);
                else if ("view-comment" === t) {
                const t = this.closest("tr").querySelectorAll("td");
                6 <= t.length ? (document.getElementById("viewCommentId").textContent = t[0].textContent, document.getElementById("viewCommentContent").textContent = t[1].textContent, document.getElementById("viewCommentArticle").textContent = t[2].textContent, document.getElementById("viewCommentUser").textContent = t[3].textContent, document.getElementById("viewCommentDate").textContent = t[4].textContent, document.getElementById("viewCommentStatus").textContent = t[5].textContent.trim(), openModal("viewCommentModal")) : alert("查看评论 #${itemId} 的详细信息")
            } else if ("delete-tag" === t) currentAction = t, currentItemId = a, document.getElementById("confirmMessage").textContent = `确定要删除标签 #${a} 吗？此操作不可撤销。`, openModal("confirmModal");
            else if ("edit-tag" === t) try {
                const t = localStorage.getItem("auth_token"),
                    g = {
                        "Content-Type": "application/json"
                    };
                t && (g.Authorization = "Bearer " + t);
                var s = await (await fetch("/api/admin/tags/" + a, {
                    headers: g
                })).json();
                s.success && s.data ? (document.getElementById("tagModalTitle").textContent = "编辑标签", document.getElementById("tagName").value = s.data.name || "", document.getElementById("tagDescription").value = s.data.description || "", document.getElementById("tagColor").value = s.data.color || "#007bff", document.getElementById("tagCategory").value = s.data.category_id || 0, document.getElementById("tagSortOrder").value = s.data.sort_order || 0, document.getElementById("tagEnabled").checked = s.data.is_enabled, document.getElementById("tagForm").dataset.tagId = a, openModal("tagModal")) : showToast("获取标签详情失败：" + (s.message || "未知错误"), "error")
            } catch (t) {
                console.error("获取标签详情失败:", t), showToast("获取标签详情失败，请稍后重试", "error")
            } else if ("edit-category" === t) try {
                const t = localStorage.getItem("auth_token"),
                    h = {
                        "Content-Type": "application/json"
                    };
                t && (h.Authorization = "Bearer " + t);
                var r = await (await fetch("/api/admin/categories/" + a, {
                    headers: h
                })).json();
                r.success && r.data ? (document.getElementById("categoryModalTitle").textContent = "编辑分类", document.getElementById("categoryName").value = r.data.name || "", document.getElementById("categoryDescription").value = r.data.description || "", document.getElementById("categoryIcon").value = r.data.icon || "", document.getElementById("categorySortOrder").value = r.data.sort_order || 0, document.getElementById("categoryEnabled").checked = r.data.is_enabled, document.getElementById("categoryForm").dataset.categoryId = a, openModal("categoryModal")) : showToast("获取分类详情失败：" + (r.message || "未知错误"), "error")
            } catch (t) {
                console.error("获取分类详情失败:", t), showToast("获取分类详情失败，请稍后重试", "error")
            }
        })
    })
}
let lastScrollTop = 0,
    isNavHidden = !1;
const nav = document.getElementById("mainNav"),
    scrollIndicator = document.getElementById("scrollIndicator"),
    scrollProgress = document.getElementById("scrollProgress"),
    mainTitle = (nav.classList.add("scrolled-top"), window.addEventListener("scroll", function() {
        var e = window.pageYOffset || document.documentElement.scrollTop,
            t = e / (document.documentElement.scrollHeight - window.innerHeight) * 100;
        100 < e ? (scrollIndicator.classList.add("active"), scrollProgress.style.height = t + "%") : scrollIndicator.classList.remove("active"), e > lastScrollTop && 50 < e ? isNavHidden || (nav.classList.add("hidden"), isNavHidden = !0) : (e < lastScrollTop || e <= 50) && (isNavHidden && (nav.classList.remove("hidden"), isNavHidden = !1), 0 === e ? (nav.classList.add("scrolled-top"), nav.classList.remove("scrolled")) : (nav.classList.remove("scrolled-top"), nav.classList.add("scrolled"))), lastScrollTop = e
    }, {
        passive: !0
    }), window.addEventListener("load", function() {
        document.body.style.opacity = "0", document.body.style.transition = "opacity 0.5s ease", setTimeout(() => {
            document.body.style.opacity = "1"
        }, 100), showEmptyState("articlesTableBody", "暂无文章", 6), showEmptyState("usersTableBody", "暂无用户", 7), showEmptyState("commentsTableBody", "暂无评论", 6), fetchAdminData(), loadCategoriesAndTags(), document.querySelectorAll(".stat-card").forEach((e, t) => {
            e.style.opacity = "0", e.style.transform = "translateY(20px)", e.style.transition = "opacity 0.6s ease, transform 0.6s ease", setTimeout(() => {
                e.style.opacity = "1", e.style.transform = "translateY(0)"
            }, 200 + 50 * t)
        })
    }), document.getElementById("main-title")),
    uploadArea = (window.openModal = function(e) {
        e = document.getElementById(e);
        e && (e.classList.add("active"), document.body.style.overflow = "hidden")
    }, window.closeModal = function(e) {
        const t = document.getElementById(e);
        t && (t.classList.add("closing"), setTimeout(() => {
            t.classList.remove("active", "closing"), document.body.style.overflow = "auto"
        }, 300))
    }, mainTitle && (mainTitle.addEventListener("mouseenter", function() {
        this.style.animationPlayState = "paused"
    }), mainTitle.addEventListener("mouseleave", function() {
        this.style.animationPlayState = "running"
    })), document.querySelectorAll(".tab-btn").forEach(e => {
        e.addEventListener("click", async function() {
            document.querySelectorAll(".tab-btn").forEach(e => {
                e.classList.remove("active")
            }), this.classList.add("active"), document.querySelectorAll(".tab-pane").forEach(e => {
                e.classList.remove("active")
            });
            var e = this.getAttribute("data-tab"),
                t = document.getElementById(e);
            t && (t.classList.add("active"), "tags" === e && await loadCategoriesAndTags(), "analytics" === e && initAnalytics(), "attachments" === e) && await loadAttachments()
        })
    }), document.querySelectorAll("[data-modal]").forEach(e => {
        (e.classList.contains("modal-close") || e.hasAttribute("data-modal")) && e.addEventListener("click", function() {
            closeModal(this.getAttribute("data-modal"))
        })
    }), document.getElementById("uploadArticleBtn").addEventListener("click", async () => {
        await populateTagSelectorForUpload(), await populateCategorySelectorForUpload(), openModal("uploadModal")
    }), document.getElementById("newArticleBtn").addEventListener("click", async () => {
        await populateTagSelectorForNewArticle(), await populateCategorySelectorForNewArticle(), openModal("articleModal")
    }), document.getElementById("addUserBtn").addEventListener("click", () => {
        openModal("userModal")
    }), document.querySelectorAll(".tag-option").forEach(e => {
        e.addEventListener("click", function() {
            this.classList.toggle("selected")
        })
    }), document.getElementById("addUploadTagBtn").addEventListener("click", async function() {
        var e, t, a, n = document.getElementById("uploadTagInput"),
            o = n.value.trim();
        o ? (e = document.getElementById("uploadTagSelector"), Array.from(e.querySelectorAll(".tag-option")).map(e => (e.dataset.tagName || e.textContent.trim()).toLowerCase()).includes(o.toLowerCase()) ? (showToast("标签已存在", "warning"), n.value = "") : (t = (t = ["#e74c3c", "#e67e22", "#f1c40f", "#2ecc71", "#1abc9c", "#3498db", "#9b59b6", "#34495e"])[Math.floor(Math.random() * t.length)], (a = document.createElement("div")).className = "tag-option selected", a.dataset.tagName = o, a.dataset.isNew = "true", a.innerHTML = `
    <span style="display: inline-block; width: 12px; height: 12px; background-color: ${t}; border-radius: 2px; margin-right: 6px;"></span>
    ${o}
  `, a.addEventListener("click", function() {
            this.classList.toggle("selected")
        }), e.appendChild(a), n.value = "", showToast(`标签 "${o}" 已添加`, "success"))) : showToast("请输入标签名称", "warning")
    }), document.getElementById("addUploadCategoryBtn").addEventListener("click", async function() {
        var e, t = document.getElementById("uploadCategoryInput"),
            a = t.value.trim();
        if (a) {
            const n = document.getElementById("uploadCategorySelector");
            Array.from(n.querySelectorAll(".category-option")).map(e => (e.dataset.categoryName || e.textContent.trim()).toLowerCase()).includes(a.toLowerCase()) ? (showToast("分类已存在", "warning"), t.value = "") : ((e = document.createElement("div")).className = "category-option selected", e.dataset.categoryName = a, e.dataset.isNew = "true", e.innerHTML = `
    <span style="display: inline-block; width: 12px; height: 12px; background-color: #007bff; border-radius: 2px; margin-right: 6px;"></span>
    ${a}
  `, e.addEventListener("click", function() {
                this.classList.contains("selected") ? this.classList.remove("selected") : (n.querySelectorAll(".category-option").forEach(e => {
                    e.classList.remove("selected")
                }), this.classList.add("selected"))
            }), n.querySelectorAll(".category-option").forEach(e => {
                e.classList.remove("selected")
            }), n.appendChild(e), t.value = "", showToast(`分类 "${a}" 已添加`, "success"))
        } else showToast("请输入分类名称", "warning")
    }), document.getElementById("uploadArea")),
    fileInput = document.getElementById("fileInput"),
    uploadPreview = document.getElementById("uploadPreview");
let selectedFiles = [];

function handleFiles(t) {
    for (let e = 0; e < t.length; e++) {
        const n = t[e];
        var a;
        10485760 < n.size ? showToast(`文件 ${n.name} 超过10MB限制`, "error") : (selectedFiles.push(n), (a = new FileReader).onload = function(e) {
            const t = document.createElement("div");
            if (t.className = "upload-item", t.dataset.fileIndex = selectedFiles.length - 1, n.type.startsWith("image/")) t.innerHTML = `
          <img src="${e.target.result}" alt="${n.name}">
          <button class="upload-remove">×</button>
        `;
            else {
                const e = n.name.split(".").pop().toUpperCase();
                t.innerHTML = `
          <div style="background:#f0f0f0; width:100%; height:100%; display:flex; flex-direction:column; align-items:center; justify-content:center; color:#666;">
            <div style="font-size:2em;"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg></div>
            <div style="font-size:0.8em; margin-top:5px; text-align:center; padding:0 5px;">${e}</div>
          </div>
          <button class="upload-remove">×</button>
        `
            }
            t.querySelector(".upload-remove").addEventListener("click", function() {
                var e = parseInt(t.dataset.fileIndex);
                selectedFiles.splice(e, 1), t.remove()
            }), uploadPreview.appendChild(t)
        }, a.readAsDataURL(n))
    }
}

function readFileContent(n) {
    return new Promise((t, a) => {
        var e = new FileReader;
        e.onload = e => {
            t(e.target.result)
        }, e.onerror = e => {
            a(new Error("读取文件失败"))
        }, e.readAsText(n)
    })
}
uploadArea.addEventListener("click", () => {
    fileInput.click()
}), uploadArea.addEventListener("dragover", e => {
    e.preventDefault(), uploadArea.classList.add("dragover")
}), uploadArea.addEventListener("dragleave", () => {
    uploadArea.classList.remove("dragover")
}), uploadArea.addEventListener("drop", e => {
    e.preventDefault(), uploadArea.classList.remove("dragover"), handleFiles(e.dataTransfer.files)
}), fileInput.addEventListener("change", () => {
    handleFiles(fileInput.files)
}), document.getElementById("uploadForm").addEventListener("submit", async function(n) {
    n.preventDefault();
    const o = document.getElementById("uploadTitle").value.trim();
    if (o) {
        const h = document.getElementById("uploadAuthor").value.trim();
        if (h) {
            const y = document.querySelector("#uploadCategorySelector .category-option.selected"),
                p = y ? y.dataset.categoryName : "",
                v = [],
                f = (document.querySelectorAll("#uploadTagSelector .tag-option.selected").forEach(e => {
                    v.push(e.dataset.tagName)
                }), this.querySelector('button[type="submit"]')),
                b = f.textContent;
            f.textContent = "上传中...", f.disabled = !0;
            try {
                const n = selectedFiles,
                    y = document.getElementById("uploadContent").value.trim();
                if (0 !== n.length || y) {
                    let t = y;
                    if (0 < n.length && !y) {
                        const o = n[0];
                        t = await readFileContent(o)
                    }
                    if (t && 0 !== t.trim().length) {
                        var a = document.getElementById("uploadStatus").value,
                            c = document.getElementById("uploadYear").value,
                            d = document.getElementById("uploadMonth").value,
                            s = document.getElementById("uploadDay").value;
                        let e = null;
                        if (c && d && s) {
                            const n = parseInt(c),
                                o = parseInt(d),
                                h = parseInt(s);
                            2020 <= n && n <= 2030 && 1 <= o && o <= 12 && 1 <= h && h <= 31 && (e = new Date(n, o - 1, h).toISOString())
                        }
                        var r = localStorage.getItem("auth_token"),
                            i = {
                                "Content-Type": "application/json"
                            },
                            l = (r && (i.Authorization = "Bearer " + r), {
                                title: o,
                                content: t,
                                author: h,
                                category: p,
                                tags: v.join(","),
                                status: a
                            });
                        e && (l.created_at = e);
                        var m = await (await fetch("/api/admin/passages", {
                            method: "POST",
                            headers: i,
                            body: JSON.stringify(l)
                        })).json();
                        if (m.success) {
                            var u = m.data.id;
                            let e = 0,
                                t = 0;
                            var g = [];
                            for (const o of n) try {
                                const n = new FormData,
                                    h = (n.append("file", o), n.append("passage_id", u), await fetch("/api/admin/attachments", {
                                        method: "POST",
                                        headers: {
                                            Authorization: "Bearer " + r
                                        },
                                        body: n
                                    })),
                                    y = await h.json();
                                y.success ? (e++, console.log(`附件 ${o.name} 上传成功`)) : (t++, g.push(o.name + ": " + (y.message || "未知错误")), console.error(`附件 ${o.name} 上传失败:`, y))
                            } catch (n) {
                                t++, g.push(o.name + ": " + (n.message || "网络错误")), console.error(`上传附件 ${o.name} 失败:`, n)
                            }
                            0 < t && console.error("附件上传失败详情:", g), f.textContent = "创建成功!", f.style.background = "rgba(255, 183, 122, 0.8)";
                            let a = "文章创建成功！";
                            0 < e && (a += ` 成功上传 ${e} 个附件`), 0 < t && (a += ` 失败 ${t} 个附件`), setTimeout(() => {
                                closeModal("uploadModal"), f.textContent = b, f.disabled = !1, f.style.background = "rgba(255, 183, 122, 0.8)", this.reset(), uploadPreview.innerHTML = "", selectedFiles = [], fetchAdminData(), showToast(a, 0 < t ? "warning" : "success")
                            }, 2e3)
                        } else f.textContent = b, f.disabled = !1, showToast("创建失败：" + (m.message || "未知错误"), "error")
                    } else showToast("文章内容不能为空", "warning"), f.textContent = b, f.disabled = !1
                } else showToast("请选择要上传的文件或输入文章内容", "warning"), f.textContent = b, f.disabled = !1
            } catch (n) {
                console.error("创建文章失败:", n), f.textContent = b, f.disabled = !1, showToast("创建失败，无法连接到服务器，请检查网络连接后重试", "error")
            }
        } else showToast("请输入作者名称", "warning"), document.getElementById("uploadAuthor").focus()
    } else showToast("请输入文章标题", "warning"), document.getElementById("uploadTitle").focus()
}), document.getElementById("articleForm").addEventListener("submit", async function(e) {
    e.preventDefault();
    var t = document.getElementById("articleTitle").value;
    if (t) {
        const n = this.querySelector('button[type="submit"]'),
            o = n.textContent;
        n.textContent = "保存中...", n.disabled = !0;
        try {
            const e = localStorage.getItem("auth_token"),
                c = {
                    "Content-Type": "application/json"
                };
            e && (c.Authorization = "Bearer " + e);
            var a = await (await fetch("/api/admin/passages", {
                method: "POST",
                headers: c,
                body: JSON.stringify({
                    title: t,
                    content: document.getElementById("articleContent").value,
                    author: document.getElementById("articleAuthor").value,
                    category: document.getElementById("articleCategory").value,
                    tags: document.getElementById("articleTags").value,
                    cover_image: document.getElementById("articleCoverImage").value
                })
            })).json();
            a.success ? (n.textContent = "保存成功!", n.style.background = "rgba(255, 183, 122, 0.8)", setTimeout(() => {
                closeModal("articleModal"), n.textContent = o, n.disabled = !1, n.style.background = "rgba(255, 183, 122, 0.8)", this.reset(), fetchAdminData(), showToast("文章创建成功！", "success")
            }, 2e3)) : (n.textContent = o, n.disabled = !1, showToast("保存失败：" + (a.message || "未知错误"), "error"))
        } catch (e) {
            console.error("保存文章失败:", e), n.textContent = o, n.disabled = !1, showToast("保存失败，请稍后重试", "error")
        }
    } else showToast("请输入文章标题", "warning")
}), document.getElementById("editForm").addEventListener("submit", async function(t) {
    t.preventDefault();
    var a = this.dataset.articleId;
    if (a) {
        const c = this.querySelector('button[type="submit"]'),
            d = c.textContent;
        c.textContent = "保存中...", c.disabled = !0;
        try {
            const t = localStorage.getItem("auth_token"),
                s = {
                    "Content-Type": "application/json"
                };
            t && (s.Authorization = "Bearer " + t);
            var n = document.getElementById("editIsScheduled").checked;
            let e = null;
            if (n) {
                const t = document.getElementById("editPublishedAt").value;
                t && (e = new Date(t).toISOString())
            }
            var o = await (await fetch("/api/admin/passages?id=" + a, {
                method: "PUT",
                headers: s,
                body: JSON.stringify({
                    title: document.getElementById("editTitle").value,
                    content: document.getElementById("editContent").value,
                    author: document.getElementById("editAuthor").value,
                    category: document.getElementById("editCategory").value,
                    show_title: document.getElementById("editShowTitle").checked,
                    tags: document.getElementById("editTags").value,
                    status: document.getElementById("editStatus").value,
                    visibility: document.getElementById("editVisibility").value,
                    is_scheduled: n,
                    published_at: e,
                    cover_image: document.getElementById("editCoverImage").value || void 0
                })
            })).json();
            o.success ? (c.textContent = "保存成功!", c.style.background = "rgba(255, 183, 122, 0.8)", setTimeout(() => {
                closeModal("editModal"), c.textContent = d, c.disabled = !1, c.style.background = "rgba(255, 183, 122, 0.8)", fetchAdminData(), showToast("文章修改已保存！", "success")
            }, 2e3)) : (c.textContent = d, c.disabled = !1, showToast("保存失败：" + (o.message || "未知错误"), "error"))
        } catch (t) {
            console.error("保存文章失败:", t), c.textContent = d, c.disabled = !1, showToast("保存失败，请稍后重试", "error")
        }
    } else showToast("无法确定要编辑的文章", "error")
}), document.getElementById("editIsScheduled").addEventListener("change", function(e) {
    const t = document.getElementById("editPublishedAtGroup"),
        a = document.getElementById("editPublishedAt");
    if (e.target.checked) {
        if (t.style.display = "block", !a.value) {
            const e = new Date,
                t = (e.setDate(e.getDate() + 1), new Date(e.getTime() - 6e4 * e.getTimezoneOffset()).toISOString().slice(0, 16));
            a.value = t
        }
    } else t.style.display = "none", a.value = ""
}), document.getElementById("userForm").addEventListener("submit", async function(e) {
    e.preventDefault();
    const t = this.querySelector('button[type="submit"]'),
        a = t.textContent;
    t.textContent = "创建中...", t.disabled = !0;
    var n = document.getElementById("userName").value.trim(),
        o = document.getElementById("userEmail").value.trim(),
        c = document.getElementById("userPassword").value,
        d = document.getElementById("userRole").value;
    try {
        const e = await fetch("/api/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    ...getAuthHeaders()
                },
                body: JSON.stringify({
                    username: n,
                    email: o,
                    password: c,
                    role: d
                })
            }),
            t = await e.json();
        t.success ? (showToast("用户创建成功！", "success"), closeModal("userModal"), this.reset(), fetchAdminData(currentPage, 10, currentUserPage, 10)) : showToast(t.message || "用户创建失败", "error")
    } catch (e) {
        console.error("创建用户失败:", e), showToast("创建用户失败，请重试", "error")
    } finally {
        t.textContent = a, t.disabled = !1
    }
}), document.getElementById("editUserForm").addEventListener("submit", async function(e) {
    e.preventDefault();
    var t = this.dataset.userId;
    if (t) {
        const c = this.querySelector('button[type="submit"]'),
            d = c.textContent;
        c.textContent = "保存中...", c.disabled = !0;
        try {
            const e = localStorage.getItem("auth_token"),
                s = {
                    "Content-Type": "application/json"
                };
            e && (s.Authorization = "Bearer " + e);
            var a = {
                    username: document.getElementById("editUserName").value,
                    email: document.getElementById("editUserEmail").value,
                    role: document.getElementById("editUserRole").value,
                    status: document.getElementById("editUserStatus").value
                },
                n = document.getElementById("editUserPassword").value;
            n && (a.password = n);
            var o = await (await fetch("/api/admin/users/" + t, {
                method: "PATCH",
                headers: s,
                body: JSON.stringify(a)
            })).json();
            o.success ? (c.textContent = "保存成功!", c.style.background = "rgba(255, 183, 122, 0.8)", setTimeout(() => {
                closeModal("editUserModal"), c.textContent = d, c.disabled = !1, c.style.background = "rgba(255, 183, 122, 0.8)", fetchAdminData(), showToast("用户修改已保存！", "success")
            }, 2e3)) : (c.textContent = d, c.disabled = !1, showToast("保存失败：" + (o.message || "未知错误"), "error"))
        } catch (e) {
            console.error("保存用户失败:", e), c.textContent = d, c.disabled = !1, showToast("保存失败，请稍后重试", "error")
        }
    } else showToast("无法确定要编辑的用户", "error")
}), document.getElementById("categoryForm").addEventListener("submit", async function(a) {
    a.preventDefault();
    const n = this.dataset.categoryId,
        o = this.querySelector('button[type="submit"]'),
        c = o.textContent;
    o.textContent = "保存中...", o.disabled = !0;
    try {
        const a = localStorage.getItem("auth_token"),
            r = {
                "Content-Type": "application/json"
            };
        a && (r.Authorization = "Bearer " + a);
        var d = {
            name: document.getElementById("categoryName").value,
            description: document.getElementById("categoryDescription").value,
            icon: document.getElementById("categoryIcon").value,
            sort_order: parseInt(document.getElementById("categorySortOrder").value) || 0,
            is_enabled: document.getElementById("categoryEnabled").checked
        };
        let e = "/api/admin/categories",
            t = "POST";
        n && (e = "/api/admin/categories/" + n, t = "PUT");
        var s = await (await fetch(e, {
            method: t,
            headers: r,
            body: JSON.stringify(d)
        })).json();
        s.success ? (o.textContent = "保存成功!", o.style.background = "rgba(255, 183, 122, 0.8)", setTimeout(() => {
            closeModal("categoryModal"), o.textContent = c, o.disabled = !1, o.style.background = "rgba(255, 183, 122, 0.8)", this.reset(), delete this.dataset.categoryId, loadCategoriesAndTags(), showToast(n ? "分类修改已保存！" : "分类创建成功！", "success")
        }, 2e3)) : (o.textContent = c, o.disabled = !1, showToast("保存失败：" + (s.message || "未知错误"), "error"))
    } catch (a) {
        console.error("保存分类失败:", a), o.textContent = c, o.disabled = !1, showToast("保存失败，请稍后重试", "error")
    }
}), document.getElementById("tagForm").addEventListener("submit", async function(a) {
    a.preventDefault();
    const n = this.dataset.tagId,
        o = this.querySelector('button[type="submit"]'),
        c = o.textContent;
    o.textContent = "保存中...", o.disabled = !0;
    try {
        const a = localStorage.getItem("auth_token"),
            r = {
                "Content-Type": "application/json"
            };
        a && (r.Authorization = "Bearer " + a);
        var d = {
            name: document.getElementById("tagName").value,
            description: document.getElementById("tagDescription").value,
            color: document.getElementById("tagColor").value,
            category_id: parseInt(document.getElementById("tagCategory").value) || 0,
            sort_order: parseInt(document.getElementById("tagSortOrder").value) || 0,
            is_enabled: document.getElementById("tagEnabled").checked
        };
        let e = "/api/admin/tags",
            t = "POST";
        n && (e += "?id=" + n, t = "PUT");
        var s = await (await fetch(e, {
            method: t,
            headers: r,
            body: JSON.stringify(d)
        })).json();
        s.success ? (o.textContent = "保存成功!", o.style.background = "rgba(255, 183, 122, 0.8)", setTimeout(() => {
            closeModal("tagModal"), o.textContent = c, o.disabled = !1, o.style.background = "rgba(255, 183, 122, 0.8)", this.reset(), delete this.dataset.tagId, loadCategoriesAndTags(), showToast(n ? "标签修改已保存！" : "标签创建成功！", "success")
        }, 2e3)) : (o.textContent = c, o.disabled = !1, showToast("保存失败：" + (s.message || "未知错误"), "error"))
    } catch (a) {
        console.error("保存标签失败:", a), o.textContent = c, o.disabled = !1, showToast("保存失败，请稍后重试", "error")
    }
});
window.currentAction = window.currentAction || null;
window.currentItemId = window.currentItemId || null;
window.selectedAttachments = window.selectedAttachments || new Set();
async function handleBatchDelete(e, t) {
    var a = t.split(",").map(e => parseInt(e.trim()));
    let n = "",
        o = "",
        c = null;
    switch (e) {
        case "batch-delete-comments":
            n = "/api/admin/comments/batch-delete", o = "批量删除评论成功", c = () => {
                clearCommentSelection(), fetchAdminData(currentPage, currentLimit, currentUserPage, currentUserLimit, currentCommentsPage, currentCommentsLimit)
            };
            break;
        case "batch-delete-categories":
            n = "/api/admin/categories/batch-delete", o = "批量删除分类成功", c = () => {
                clearCategorySelection(), loadCategoriesAndTags()
            };
            break;
        case "batch-delete-tags":
            n = "/api/admin/tags/batch-delete", o = "批量删除标签成功", c = () => {
                clearTagSelection(), loadCategoriesAndTags()
            };
            break;
        case "batch-delete-users":
            n = "/api/admin/users/batch-delete", o = "批量删除用户成功", c = () => {
                clearUserSelection(), fetchAdminData(currentPage, 10, currentUserPage, 10)
            };
            break;
        case "batch-delete-articles":
            n = "/api/admin/passages/batch-delete", o = "批量删除文章成功", c = () => {
                clearSelection(), fetchAdminData(currentPage, 10)
            };
            break;
        default:
            return
    }
    try {
        const e = await fetch(n, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    ...getAuthHeaders()
                },
                body: JSON.stringify({
                    ids: a
                })
            }),
            t = await e.json();
        t.success ? (closeModal("confirmModal"), showToast(t.message || o, "success"), c && c()) : showToast(t.message || "批量删除失败", "error")
    } catch (e) {
        console.error("批量删除失败:", e), showToast("批量删除失败，请重试", "error")
    }
}
document.querySelectorAll(".modal").forEach(e => {
    e.addEventListener("click", function(e) {
        e.target === this && closeModal(this.id)
    })
}), document.addEventListener("keydown", function(e) {
    "Escape" === e.key && document.querySelectorAll(".modal.active").forEach(e => {
        closeModal(e.id)
    })
});
