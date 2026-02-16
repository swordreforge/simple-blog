function showToast(e, t = 'success') {
  const n = document.getElementById('toastContainer');
  if (!n) return void console.error('Toast container not found');
  const a = document.createElement('div');
  a.className = `toast ${t}`;
  let o = '';
  switch (t) {
    case 'success':
    default:
      o = '⭕';
      break;
    case 'error':
      o = '❌';
      break;
    case 'warning':
      o = '⚠️';
  }
  ((a.innerHTML = `\n    <div class="toast-icon">${o}</div>\n    <div class="toast-message">${e}</div>\n    <button class="toast-close" onclick="this.parentElement.remove()">×</button>\n  `),
    n.appendChild(a),
    setTimeout(() => {
      a.parentElement && a.remove();
    }, 3e3));
}
async function fetchCategories() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = { 'Content-Type': 'application/json' };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/admin/categories', { headers: t }),
      a = await n.json();
    return a.success && a.data ? a.data : (console.error('获取分类列表失败:', a.message), []);
  } catch (e) {
    return (console.error('获取分类列表失败:', e), []);
  }
}
async function fetchTags(e = null) {
  try {
    const t = localStorage.getItem('auth_token'),
      n = { 'Content-Type': 'application/json' };
    t && (n.Authorization = `Bearer ${t}`);
    let a = '/api/admin/tags';
    null !== e && '' !== e && (a += `?category_id=${e}`);
    const o = await fetch(a, { headers: n }),
      c = await o.json();
    return c.success && c.data ? c.data : (console.error('获取标签列表失败:', c.message), []);
  } catch (e) {
    return (console.error('获取标签列表失败:', e), []);
  }
}
function updateCategoriesTable(e) {
  const t = document.querySelector('#categories tbody');
  t &&
    ((t.innerHTML = ''),
    0 !== e.length
      ? (e.forEach(e => {
          const n = document.createElement('tr');
          ((n.innerHTML = `\n      <td>\n        <input type="checkbox" class="category-checkbox" data-id="${e.id}">\n      </td>\n      <td>${e.sort_order}</td>\n      <td>${e.icon || ''}</td>\n      <td>${e.name}</td>\n      <td>${e.description || '-'}</td>\n      <td><span style="color: ${e.is_enabled ? '#00b894' : '#e74c3c'};">${e.is_enabled ? '启用' : '禁用'}</span></td>\n      <td class="action-buttons">\n        <button class="btn btn-sm btn-edit" data-action="edit-category" data-id="${e.id}">编辑</button>\n        <button class="btn btn-sm btn-delete" data-action="delete-category" data-id="${e.id}">删除</button>\n      </td>\n    `),
            t.appendChild(n));
        }),
        bindActionButtons(),
        bindCategoryCheckboxes())
      : (t.innerHTML =
          '\n      <tr>\n        <td colspan="7" style="text-align: center; padding: 40px; color: #999;">\n          <div style="font-size: 48px; margin-bottom: 10px;">📭</div>\n          <div>暂无分类</div>\n        </td>\n      </tr>\n    '));
}
function updateTagsTable(e) {
  const t = document.querySelector('#tags tbody');
  t &&
    ((t.innerHTML = ''),
    0 !== e.length
      ? (e.forEach(e => {
          const n = document.createElement('tr');
          ((n.innerHTML = `\n      <td>\n        <input type="checkbox" class="tag-checkbox" data-id="${e.id}">\n      </td>\n      <td>${e.sort_order}</td>\n      <td><span style="display: inline-block; width: 20px; height: 20px; background-color: ${e.color}; border-radius: 4px;"></span></td>\n      <td>${e.name}</td>\n      <td>${e.description || '-'}</td>\n      <td>${0 === e.category_id ? '无分类' : e.category_id}</td>\n      <td><span style="color: ${e.is_enabled ? '#00b894' : '#e74c3c'};">${e.is_enabled ? '启用' : '禁用'}</span></td>\n      <td class="action-buttons">\n        <button class="btn btn-sm btn-edit" data-action="edit-tag" data-id="${e.id}">编辑</button>\n        <button class="btn btn-sm btn-delete" data-action="delete-tag" data-id="${e.id}">删除</button>\n      </td>\n    `),
            t.appendChild(n));
        }),
        bindActionButtons(),
        bindTagCheckboxes())
      : (t.innerHTML =
          '\n      <tr>\n        <td colspan="8" style="text-align: center; padding: 40px; color: #999;">\n          <div style="font-size: 48px; margin-bottom: 10px;">📭</div>\n          <div>暂无标签</div>\n        </td>\n      </tr>\n    '));
}
async function loadCategoriesAndTags() {
  const e = await fetchCategories();
  updateCategoriesTable(e);
  const t = document.getElementById('tagCategoryFilter');
  t &&
    ((t.innerHTML = '<option value="">全部标签</option>'),
    e.forEach(e => {
      const n = document.createElement('option');
      ((n.value = e.id), (n.textContent = e.name), t.appendChild(n));
    }));
  const n = document.getElementById('tagCategory');
  n &&
    ((n.innerHTML = '<option value="0">无分类</option>'),
    e.forEach(e => {
      const t = document.createElement('option');
      ((t.value = e.id), (t.textContent = e.name), n.appendChild(t));
    }));
  updateTagsTable(await fetchTags());
}
async function populateCategorySelect(e, t = '') {
  const n = document.getElementById(e);
  if (!n) return;
  const a = await fetchCategories();
  n.innerHTML = '';
  const o = document.createElement('option');
  ((o.value = ''),
    (o.textContent = '未分类'),
    ('' !== t && '未分类' !== t) || (o.selected = !0),
    n.appendChild(o),
    a.length > 0 &&
      a.forEach(e => {
        const a = document.createElement('option');
        ((a.value = e.name),
          (a.textContent = e.name),
          e.icon && (a.textContent = `${e.icon} ${e.name}`),
          e.name === t && (a.selected = !0),
          n.appendChild(a));
      }));
}
async function populateTagSelector(e = []) {
  const t = document.getElementById('editTagSelector');
  if (!t) return;
  const n = await fetchTags();
  if (((t.innerHTML = ''), 0 === n.length))
    return void (t.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无标签</div>');
  n.forEach(n => {
    if (!n.is_enabled) return;
    const a = document.createElement('div');
    ((a.className = 'tag-option'),
      (a.dataset.tagId = n.id),
      (a.dataset.tagName = n.name),
      (a.innerHTML = `\n      <span style="display: inline-block; width: 12px; height: 12px; background-color: ${n.color}; border-radius: 2px; margin-right: 6px;"></span>\n      ${n.name}\n    `),
      e.includes(n.name) && a.classList.add('selected'),
      a.addEventListener('click', function () {
        (this.classList.toggle('selected'), updateSelectedTags());
      }),
      t.appendChild(a));
  });
  const a = document.getElementById('editTags');
  a && (a.value = JSON.stringify(e));
}
function updateSelectedTags() {
  const e = [];
  document.querySelectorAll('#editTagSelector .tag-option.selected').forEach(t => {
    e.push(t.dataset.tagName);
  });
  const t = document.getElementById('editTags');
  t && (t.value = JSON.stringify(e));
}
async function populateTagSelectorForNewArticle() {
  const e = document.getElementById('articleTagSelector');
  if (!e) return;
  const t = await fetchTags();
  ((e.innerHTML = ''),
    0 !== t.length
      ? (t.forEach(t => {
          if (!t.is_enabled) return;
          const n = document.createElement('div');
          ((n.className = 'tag-option'),
            (n.dataset.tagId = t.id),
            (n.dataset.tagName = t.name),
            (n.innerHTML = `\n      <span style="display: inline-block; width: 12px; height: 12px; background-color: ${t.color}; border-radius: 2px; margin-right: 6px;"></span>\n      ${t.name}\n    `),
            n.addEventListener('click', function () {
              (this.classList.toggle('selected'), updateSelectedTagsForNewArticle());
            }),
            e.appendChild(n));
        }),
        updateSelectedTagsForNewArticle())
      : (e.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无标签</div>'));
}
function updateSelectedTagsForNewArticle() {
  const e = [];
  document.querySelectorAll('#articleTagSelector .tag-option.selected').forEach(t => {
    e.push(t.dataset.tagName);
  });
  const t = document.getElementById('articleTags');
  t && (t.value = JSON.stringify(e));
}
async function populateTagSelectorForUpload() {
  const e = document.getElementById('uploadTagSelector');
  if (!e) return;
  const t = await fetchTags();
  ((e.innerHTML = ''),
    0 !== t.length
      ? t.forEach(t => {
          if (!t.is_enabled) return;
          const n = document.createElement('div');
          ((n.className = 'tag-option'),
            (n.dataset.tagId = t.id),
            (n.dataset.tagName = t.name),
            (n.innerHTML = `\n      <span style="display: inline-block; width: 12px; height: 12px; background-color: ${t.color}; border-radius: 2px; margin-right: 6px;"></span>\n      ${t.name}\n    `),
            n.addEventListener('click', function () {
              this.classList.toggle('selected');
            }),
            e.appendChild(n));
        })
      : (e.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无标签</div>'));
}
async function populateCategorySelectorForUpload() {
  const e = document.getElementById('uploadCategorySelector');
  if (!e) return;
  const t = await fetchCategories();
  ((e.innerHTML = ''),
    0 !== t.length
      ? t.forEach(t => {
          if (!t.is_enabled) return;
          const n = document.createElement('div');
          ((n.className = 'category-option'),
            (n.dataset.categoryId = t.id),
            (n.dataset.categoryName = t.name),
            (n.innerHTML = `\n      <span style="display: inline-block; width: 12px; height: 12px; background-color: #007bff; border-radius: 2px; margin-right: 6px;"></span>\n      ${t.name}\n    `),
            n.addEventListener('click', function () {
              this.classList.contains('selected')
                ? this.classList.remove('selected')
                : (e.querySelectorAll('.category-option').forEach(e => {
                    e.classList.remove('selected');
                  }),
                  this.classList.add('selected'));
            }),
            e.appendChild(n));
        })
      : (e.innerHTML = '<div style="color: #999; font-size: 0.9em;">暂无分类</div>'));
}
async function populateCategorySelectorForNewArticle() {
  const e = document.getElementById('articleCategory');
  if (!e) return;
  const t = await fetchCategories();
  if (((e.innerHTML = ''), 0 === t.length)) {
    const t = document.createElement('option');
    return ((t.value = ''), (t.textContent = '暂无分类'), void e.appendChild(t));
  }
  t.forEach(t => {
    const n = document.createElement('option');
    ((n.value = t.name), (n.textContent = t.name), e.appendChild(n));
  });
}
async function fetchAdminData(e = 1, t = 10, n = 1, a = 10, o = 1, c = 10) {
  try {
    const s = localStorage.getItem('auth_token'),
      d = { 'Content-Type': 'application/json' };
    s && (d.Authorization = `Bearer ${s}`);
    const i = await fetch(`/api/admin/passages?page=${e}&limit=${t}`, { headers: d }),
      r = await i.json();
    if (r.success && r.data)
      if (r.data.length > 0) {
        updateArticlesTable(r.data);
        (updateStatCard('totalArticles', r.pagination?.total || r.data.length),
          updatePagination(r.pagination));
      } else
        (showEmptyState('articlesTableBody', '暂无文章'),
          updateStatCard('totalArticles', 0),
          hidePagination());
    else
      (showEmptyState('articlesTableBody', '暂无文章'),
        updateStatCard('totalArticles', 0),
        hidePagination());
    const l = await fetch(`/api/admin/users?page=${n}&limit=${a}`, { headers: d }),
      u = await l.json();
    if (u.success && u.data)
      if (u.data.length > 0) {
        updateUsersTable(u.data);
        (updateStatCard('totalUsers', u.pagination?.total || u.data.length),
          updateUserPagination(u.pagination));
      } else
        (showEmptyState('usersTableBody', '暂无用户', 7),
          updateStatCard('totalUsers', 0),
          hideUserPagination());
    else
      (showEmptyState('usersTableBody', '暂无用户', 7),
        updateStatCard('totalUsers', 0),
        hideUserPagination());
    const m = await fetch(`/api/admin/comments?page=${o}&limit=${c}`, { headers: d }),
      g = await m.json();
    g.success && g.data && g.data.length > 0
      ? (updateCommentsTable(g.data), updateCommentsPagination(g.pagination))
      : (showEmptyState('commentsTableBody', '暂无评论', 6), hideCommentsPagination());
    const h = await fetch('/api/admin/stats', { headers: d }),
      y = await h.json();
    if ((console.log('统计数据响应:', y), y.success && y.data)) {
      updateStatCard('todayVisits', y.data.today_visits || 0);
      const e = document
        .querySelector('#todayVisits')
        .closest('.stat-card')
        .querySelector('.stat-change');
      if (e && void 0 !== y.data.yesterday_visits) {
        y.data.yesterday_visits;
        const t = y.data.visits_change_percent || 0,
          n = y.data.visits_trend || 'stable';
        'up' === n
          ? ((e.className = 'stat-change positive'), (e.textContent = `较昨日 +${t.toFixed(1)}%`))
          : 'down' === n
            ? ((e.className = 'stat-change negative'), (e.textContent = `较昨日 ${t.toFixed(1)}%`))
            : ((e.className = 'stat-change neutral'), (e.textContent = '较昨日持平'));
      }
    } else console.error('获取统计数据失败:', y);
  } catch (e) {
    console.error('获取管理数据失败:', e);
  }
}
function showEmptyState(e, t, n = 6) {
  const a = document.querySelector(`#${e}`);
  if (!a) return;
  const o = localStorage.getItem('auth_token');
  let c = t;
  (o || (c = '请先登录以查看数据'),
    (a.innerHTML = `\n    <tr>\n      <td colspan="${n}" style="text-align: center; padding: 40px; color: #999;">\n        <div style="font-size: 48px; margin-bottom: 10px;">📭</div>\n        <div>${c}</div>\n        ${o ? '' : '<div style="margin-top: 10px; font-size: 0.9em;"><a href="#loginModal" onclick="document.getElementById(\'loginBtn\').click(); return false;" style="color: #007bff; text-decoration: none;">点击登录</a></div>'}\n      </td>\n    </tr>\n  `));
}
let currentPage = 1,
  currentLimit = 100,
  totalPages = 1;
function updatePagination(e) {
  if (!e) return void hidePagination();
  const t = parseInt(e.page) || 1,
    n = parseInt(e.limit) || 100,
    a = parseInt(e.total) || 0;
  ((currentPage = t), (currentLimit = n), (totalPages = Math.ceil(a / n)));
  const o = document.getElementById('paginationContainer');
  o && (o.style.display = 'flex');
  const c = (t - 1) * n + 1,
    s = Math.min(t * n, a),
    d = document.getElementById('paginationInfo');
  d && (d.textContent = `显示 ${c}-${s} 条，共 ${a} 条`);
  const i = document.getElementById('prevPageBtn'),
    r = document.getElementById('nextPageBtn');
  (i && (i.disabled = t <= 1),
    r && (r.disabled = t >= totalPages),
    generatePageButtons(t, totalPages));
}
function hidePagination() {
  const e = document.getElementById('paginationContainer');
  e && (e.style.display = 'none');
}
function generatePageButtons(e, t) {
  const n = document.getElementById('paginationPages');
  if (!n) return;
  n.innerHTML = '';
  let a = Math.max(1, e - 2),
    o = Math.min(t, e + 2);
  (o - a < 4 && (1 === a ? (o = Math.min(t, 5)) : o === t && (a = Math.max(1, t - 4))),
    a > 1 && (addPageButton(1, n), a > 2 && addEllipsis(n)));
  for (let t = a; t <= o; t++) addPageButton(t, n, t === e);
  o < t && (o < t - 1 && addEllipsis(n), addPageButton(t, n));
}
function addPageButton(e, t, n = !1) {
  const a = document.createElement('button');
  ((a.className = 'pagination-page ' + (n ? 'active' : '')),
    (a.textContent = e),
    a.addEventListener('click', () => {
      e !== currentPage && fetchAdminData(e, currentLimit);
    }),
    t.appendChild(a));
}
function addEllipsis(e) {
  const t = document.createElement('span');
  ((t.className = 'pagination-page ellipsis'), (t.textContent = '...'), e.appendChild(t));
}
function goToPrevPage() {
  currentPage > 1 && fetchAdminData(currentPage - 1, currentLimit);
}
function goToNextPage() {
  currentPage < totalPages && fetchAdminData(currentPage + 1, currentLimit);
}
function getUsernameFromToken() {
  const e = localStorage.getItem('auth_token');
  if (!e) return '管理员';
  try {
    const t = e.split('.');
    if (3 !== t.length) return '管理员';
    const n = t[1].replace(/-/g, '+').replace(/_/g, '/'),
      a = atob(n);
    return JSON.parse(a).username || '管理员';
  } catch (e) {
    return (console.error('解析token失败:', e), '管理员');
  }
}
async function updateWelcomeMessage() {
  const e = getUsernameFromToken(),
    t = document.querySelector('.welcome-text h2');
  t && (t.textContent = `欢迎回来，${e}`);
  let n = '/img/avatar.webp';
  try {
    const e = localStorage.getItem('auth_token'),
      t = { 'Content-Type': 'application/json' };
    e && (t.Authorization = `Bearer ${e}`);
    const a = await fetch('/api/settings/template', { method: 'GET', headers: t });
    if (a.ok) {
      const e = await a.json();
      e.global_avatar && (n = e.global_avatar);
    }
  } catch (e) {
    console.error('获取全局头像设置失败:', e);
  }
  const a = document.querySelector('.admin-avatar');
  a && (a.innerHTML = `<img src="${n}" alt="${e}" class="avatar-image">`);
}
document.addEventListener('DOMContentLoaded', function () {
  updateWelcomeMessage();
  const e = document.getElementById('prevPageBtn'),
    t = document.getElementById('nextPageBtn'),
    n = document.getElementById('refreshArticlesBtn');
  (e && e.addEventListener('click', goToPrevPage),
    t && t.addEventListener('click', goToNextPage),
    n &&
      n.addEventListener('click', () => {
        fetchAdminData(currentPage, currentLimit);
      }));
  const a = document.getElementById('prevUserPageBtn'),
    o = document.getElementById('nextUserPageBtn');
  (a && a.addEventListener('click', goToPrevUserPage),
    o && o.addEventListener('click', goToNextUserPage));
  const c = document.getElementById('prevCommentsPageBtn'),
    s = document.getElementById('nextCommentsPageBtn'),
    d = document.getElementById('refreshCommentsBtn');
  (c && c.addEventListener('click', goToPrevCommentsPage),
    s && s.addEventListener('click', goToNextCommentsPage),
    d &&
      d.addEventListener('click', () => {
        fetchAdminData(
          currentPage,
          currentLimit,
          currentUserPage,
          currentUserLimit,
          currentCommentsPage,
          currentCommentsLimit
        );
      }));
  const i = document.getElementById('batchDeleteCommentsBtn');
  i && i.addEventListener('click', batchDeleteComments);
  const r = document.getElementById('addCategoryBtn'),
    l = document.getElementById('refreshCategoriesBtn');
  (r &&
    r.addEventListener('click', () => {
      ((document.getElementById('categoryModalTitle').textContent = '添加分类'),
        document.getElementById('categoryForm').reset(),
        delete document.getElementById('categoryForm').dataset.categoryId,
        openModal('categoryModal'));
    }),
    l && l.addEventListener('click', loadCategoriesAndTags));
  const u = document.getElementById('batchDeleteCategoriesBtn');
  u && u.addEventListener('click', batchDeleteCategories);
  const m = document.getElementById('addTagBtn'),
    g = document.getElementById('refreshTagsBtn'),
    h = document.getElementById('tagCategoryFilter');
  (m &&
    m.addEventListener('click', async () => {
      (await loadCategoriesAndTags(),
        (document.getElementById('tagModalTitle').textContent = '添加标签'),
        document.getElementById('tagForm').reset(),
        delete document.getElementById('tagForm').dataset.tagId,
        openModal('tagModal'));
    }),
    g && g.addEventListener('click', loadCategoriesAndTags));
  const y = document.getElementById('batchDeleteTagsBtn');
  (y && y.addEventListener('click', batchDeleteTags),
    h &&
      h.addEventListener('change', async function () {
        const e = this.value;
        updateTagsTable(await fetchTags(e));
      }));
});
let currentUserPage = 1,
  currentUserLimit = 10,
  totalUserPages = 1;
function updateUserPagination(e) {
  if (!e) return void hideUserPagination();
  const t = parseInt(e.page) || 1,
    n = parseInt(e.limit) || 100,
    a = parseInt(e.total) || 0;
  ((currentUserPage = t), (currentUserLimit = n), (totalUserPages = Math.ceil(a / n)));
  const o = document.getElementById('userPaginationContainer');
  o && (o.style.display = 'flex');
  const c = (t - 1) * n + 1,
    s = Math.min(t * n, a),
    d = document.getElementById('userPaginationInfo');
  d && (d.textContent = `显示 ${c}-${s} 条，共 ${a} 条`);
  const i = document.getElementById('prevUserPageBtn'),
    r = document.getElementById('nextUserPageBtn');
  (i && (i.disabled = t <= 1),
    r && (r.disabled = t >= totalUserPages),
    generateUserPageButtons(t, totalUserPages));
}
function hideUserPagination() {
  const e = document.getElementById('userPaginationContainer');
  e && (e.style.display = 'none');
}
function generateUserPageButtons(e, t) {
  const n = document.getElementById('userPaginationPages');
  if (!n) return;
  n.innerHTML = '';
  let a = Math.max(1, e - 2),
    o = Math.min(t, e + 2);
  (o - a < 4 && (1 === a ? (o = Math.min(t, 5)) : o === t && (a = Math.max(1, t - 4))),
    a > 1 && (addUserPageButton(1, n), a > 2 && addUserEllipsis(n)));
  for (let t = a; t <= o; t++) addUserPageButton(t, n, t === e);
  o < t && (o < t - 1 && addUserEllipsis(n), addUserPageButton(t, n));
}
function addUserPageButton(e, t, n = !1) {
  const a = document.createElement('button');
  ((a.className = 'pagination-page ' + (n ? 'active' : '')),
    (a.textContent = e),
    a.addEventListener('click', () => {
      e !== currentUserPage && fetchAdminData(currentPage, currentLimit, e, currentUserLimit);
    }),
    t.appendChild(a));
}
function addUserEllipsis(e) {
  const t = document.createElement('span');
  ((t.className = 'pagination-page ellipsis'), (t.textContent = '...'), e.appendChild(t));
}
function goToPrevUserPage() {
  currentUserPage > 1 &&
    fetchAdminData(currentPage, currentLimit, currentUserPage - 1, currentUserLimit);
}
function goToNextUserPage() {
  currentUserPage < totalUserPages &&
    fetchAdminData(currentPage, currentLimit, currentUserPage + 1, currentUserLimit);
}
let currentCommentsPage = 1,
  currentCommentsLimit = 10,
  totalCommentsPages = 1;
function updateCommentsTable(e) {
  const t = document.querySelector('#comments tbody');
  t &&
    ((t.innerHTML = ''),
    e.forEach(e => {
      const n = document.createElement('tr'),
        a = generateAdminIdenticon(e.username || 'anonymous', 24);
      ((n.innerHTML = `\n      <td>\n        <input type="checkbox" class="comment-checkbox" data-id="${e.id}">\n      </td>\n      <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${e.content}</td>\n      <td>${e.passage_uuid}</td>\n      <td>\n        <div style="display: flex; align-items: center; gap: 8px;">\n          <img src="${a}" alt="${e.username || '匿名用户'}" style="width: 24px; height: 24px; border-radius: 50%;"/>\n          <span>${e.username}</span>\n        </div>\n      </td>\n      <td>${e.created_at}</td>\n      <td class="action-buttons">\n        <button class="btn btn-sm btn-delete" data-action="delete-comment" data-id="${e.id}">删除</button>\n      </td>\n    `),
        t.appendChild(n));
    }),
    bindActionButtons(),
    bindCommentCheckboxes());
}
function generateAdminIdenticon(e, t = 24) {
  const n = simpleAdminHash(e),
    a = n % 360,
    o = 65 + (n % 15),
    c = 55 + (n % 10),
    s = `hsl(${a}, ${o}%, ${c}%)`,
    d = `hsl(${a}, ${o}%, ${c - 25}%)`;
  let i = '';
  for (let e = 0; e < 5; e++)
    for (let t = 0; t < Math.ceil(2.5); t++) {
      (n >> (5 * e + t)) & 1 &&
        ((i += `<rect x="${t}" y="${e}" width="1" height="1" fill="${s}"/>`),
        (i += `<rect x="${4 - t}" y="${e}" width="1" height="1" fill="${s}"/>`));
    }
  const r = e ? e.charAt(0).toUpperCase() : '?';
  return (
    'data:image/svg+xml;base64,' +
    btoa(
      unescape(
        encodeURIComponent(
          `\n    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5 5" width="${t}" height="${t}">\n      <rect width="5" height="5" fill="#f0f0f0"/>\n      ${i}\n      <text x="2.5" y="2.85" \n            text-anchor="middle" \n            font-size="2.5" \n            font-weight="bold" \n            fill="${d}"\n            font-family="system-ui, -apple-system, sans-serif">${r}</text>\n    </svg>\n  `
        )
      )
    )
  );
}
function simpleAdminHash(e) {
  let t = 0;
  for (let n = 0; n < e.length; n++) {
    ((t = (t << 5) - t + e.charCodeAt(n)), (t &= t));
  }
  return Math.abs(t);
}
function bindCommentCheckboxes() {
  const e = document.getElementById('selectAllComments'),
    t = document.querySelectorAll('.comment-checkbox');
  (e &&
    e.addEventListener('change', function () {
      (t.forEach(e => {
        e.checked = this.checked;
        const t = e.closest('tr');
        t && (this.checked ? t.classList.add('selected') : t.classList.remove('selected'));
      }),
        updateBatchDeleteButton());
    }),
    t.forEach(e => {
      e.addEventListener('change', function () {
        const e = this.closest('tr');
        (e && (this.checked ? e.classList.add('selected') : e.classList.remove('selected')),
          updateBatchDeleteButton(),
          updateSelectAllCheckbox());
      });
    }));
}
function updateBatchDeleteButton() {
  const e = document.querySelectorAll('.comment-checkbox:checked').length,
    t = document.getElementById('batchDeleteCommentsBtn');
  t &&
    (e > 0
      ? ((t.style.display = 'inline-block'), (t.textContent = `批量删除 (${e})`))
      : (t.style.display = 'none'));
}
function updateSelectAllCheckbox() {
  const e = document.getElementById('selectAllComments'),
    t = document.querySelectorAll('.comment-checkbox');
  if (e && t.length > 0) {
    const n = Array.from(t).every(e => e.checked),
      a = Array.from(t).some(e => e.checked);
    ((e.checked = n), (e.indeterminate = a && !n));
  }
}
function clearCommentSelection() {
  const e = document.getElementById('selectAllComments'),
    t = document.querySelectorAll('.comment-checkbox');
  (e && ((e.checked = !1), (e.indeterminate = !1)),
    t.forEach(e => {
      e.checked = !1;
      const t = e.closest('tr');
      t && t.classList.remove('selected');
    }),
    updateBatchDeleteButton());
}
async function batchDeleteComments() {
  const e = document.querySelectorAll('.comment-checkbox:checked'),
    t = Array.from(e).map(e => parseInt(e.dataset.id));
  0 !== t.length
    ? ((currentAction = 'batch-delete-comments'),
      (currentItemId = t.join(',')),
      (document.getElementById('confirmMessage').textContent =
        `确定要删除选中的 ${t.length} 条评论吗？此操作不可恢复！`),
      openModal('confirmModal'))
    : showToast('请选择要删除的评论', 'warning');
}
function bindCategoryCheckboxes() {
  const e = document.getElementById('selectAllCategories'),
    t = document.querySelectorAll('.category-checkbox');
  (e &&
    e.addEventListener('change', function () {
      (t.forEach(e => {
        e.checked = this.checked;
        const t = e.closest('tr');
        t && (this.checked ? t.classList.add('selected') : t.classList.remove('selected'));
      }),
        updateBatchDeleteCategoriesButton());
    }),
    t.forEach(e => {
      e.addEventListener('change', function () {
        const e = this.closest('tr');
        (e && (this.checked ? e.classList.add('selected') : e.classList.remove('selected')),
          updateBatchDeleteCategoriesButton(),
          updateSelectAllCategoriesCheckbox());
      });
    }));
}
function updateBatchDeleteCategoriesButton() {
  const e = document.querySelectorAll('.category-checkbox:checked').length,
    t = document.getElementById('batchDeleteCategoriesBtn');
  t &&
    (e > 0
      ? ((t.style.display = 'inline-block'), (t.textContent = `批量删除 (${e})`))
      : (t.style.display = 'none'));
}
function updateSelectAllCategoriesCheckbox() {
  const e = document.getElementById('selectAllCategories'),
    t = document.querySelectorAll('.category-checkbox');
  if (e && t.length > 0) {
    const n = Array.from(t).every(e => e.checked),
      a = Array.from(t).some(e => e.checked);
    ((e.checked = n), (e.indeterminate = a && !n));
  }
}
function clearCategorySelection() {
  const e = document.getElementById('selectAllCategories'),
    t = document.querySelectorAll('.category-checkbox');
  (e && ((e.checked = !1), (e.indeterminate = !1)),
    t.forEach(e => {
      e.checked = !1;
      const t = e.closest('tr');
      t && t.classList.remove('selected');
    }),
    updateBatchDeleteCategoriesButton());
}
async function batchDeleteCategories() {
  const e = document.querySelectorAll('.category-checkbox:checked'),
    t = Array.from(e).map(e => parseInt(e.dataset.id));
  0 !== t.length
    ? ((currentAction = 'batch-delete-categories'),
      (currentItemId = t.join(',')),
      (document.getElementById('confirmMessage').textContent =
        `确定要删除选中的 ${t.length} 个分类吗？此操作不可恢复！`),
      openModal('confirmModal'))
    : showToast('请选择要删除的分类', 'warning');
}
function bindTagCheckboxes() {
  const e = document.getElementById('selectAllTags'),
    t = document.querySelectorAll('.tag-checkbox');
  (e &&
    e.addEventListener('change', function () {
      (t.forEach(e => {
        e.checked = this.checked;
        const t = e.closest('tr');
        t && (this.checked ? t.classList.add('selected') : t.classList.remove('selected'));
      }),
        updateBatchDeleteTagsButton());
    }),
    t.forEach(e => {
      e.addEventListener('change', function () {
        const e = this.closest('tr');
        (e && (this.checked ? e.classList.add('selected') : e.classList.remove('selected')),
          updateBatchDeleteTagsButton(),
          updateSelectAllTagsCheckbox());
      });
    }));
}
function updateBatchDeleteTagsButton() {
  const e = document.querySelectorAll('.tag-checkbox:checked').length,
    t = document.getElementById('batchDeleteTagsBtn');
  t &&
    (e > 0
      ? ((t.style.display = 'inline-block'), (t.textContent = `批量删除 (${e})`))
      : (t.style.display = 'none'));
}
function updateSelectAllTagsCheckbox() {
  const e = document.getElementById('selectAllTags'),
    t = document.querySelectorAll('.tag-checkbox');
  if (e && t.length > 0) {
    const n = Array.from(t).every(e => e.checked),
      a = Array.from(t).some(e => e.checked);
    ((e.checked = n), (e.indeterminate = a && !n));
  }
}
function clearTagSelection() {
  const e = document.getElementById('selectAllTags'),
    t = document.querySelectorAll('.tag-checkbox');
  (e && ((e.checked = !1), (e.indeterminate = !1)),
    t.forEach(e => {
      e.checked = !1;
      const t = e.closest('tr');
      t && t.classList.remove('selected');
    }),
    updateBatchDeleteTagsButton());
}
async function batchDeleteTags() {
  const e = document.querySelectorAll('.tag-checkbox:checked'),
    t = Array.from(e).map(e => parseInt(e.dataset.id));
  0 !== t.length
    ? ((currentAction = 'batch-delete-tags'),
      (currentItemId = t.join(',')),
      (document.getElementById('confirmMessage').textContent =
        `确定要删除选中的 ${t.length} 个标签吗？此操作不可恢复！`),
      openModal('confirmModal'))
    : showToast('请选择要删除的标签', 'warning');
}
function updateCommentsPagination(e) {
  if (!e) return void hideCommentsPagination();
  const t = parseInt(e.page) || 1,
    n = parseInt(e.limit) || 100,
    a = parseInt(e.total) || 0;
  ((currentCommentsPage = t), (currentCommentsLimit = n), (totalCommentsPages = Math.ceil(a / n)));
  const o = document.getElementById('commentsPaginationContainer');
  o && (o.style.display = 'flex');
  const c = (t - 1) * n + 1,
    s = Math.min(t * n, a),
    d = document.getElementById('commentsPaginationInfo');
  d && (d.textContent = `显示 ${c}-${s} 条，共 ${a} 条`);
  const i = document.getElementById('prevCommentsPageBtn'),
    r = document.getElementById('nextCommentsPageBtn');
  (i && (i.disabled = t <= 1),
    r && (r.disabled = t >= totalCommentsPages),
    generateCommentsPageButtons(t, totalCommentsPages));
}
function hideCommentsPagination() {
  const e = document.getElementById('commentsPaginationContainer');
  e && (e.style.display = 'none');
}
function generateCommentsPageButtons(e, t) {
  const n = document.getElementById('commentsPaginationPages');
  if (!n) return;
  n.innerHTML = '';
  let a = Math.max(1, e - 2),
    o = Math.min(t, e + 2);
  (o - a < 4 && (1 === a ? (o = Math.min(t, 5)) : o === t && (a = Math.max(1, t - 4))),
    a > 1 && (addCommentsPageButton(1, n), a > 2 && addCommentsEllipsis(n)));
  for (let t = a; t <= o; t++) addCommentsPageButton(t, n, t === e);
  o < t && (o < t - 1 && addCommentsEllipsis(n), addCommentsPageButton(t, n));
}
function addCommentsPageButton(e, t, n = !1) {
  const a = document.createElement('button');
  ((a.className = 'pagination-page ' + (n ? 'active' : '')),
    (a.textContent = e),
    a.addEventListener('click', () => {
      e !== currentCommentsPage &&
        fetchAdminData(
          currentPage,
          currentLimit,
          currentUserPage,
          currentUserLimit,
          e,
          currentCommentsLimit
        );
    }),
    t.appendChild(a));
}
function addCommentsEllipsis(e) {
  const t = document.createElement('span');
  ((t.className = 'pagination-page ellipsis'), (t.textContent = '...'), e.appendChild(t));
}
function goToPrevCommentsPage() {
  currentCommentsPage > 1 &&
    fetchAdminData(
      currentPage,
      currentLimit,
      currentUserPage,
      currentUserLimit,
      currentCommentsPage - 1,
      currentCommentsLimit
    );
}
function goToNextCommentsPage() {
  currentCommentsPage < totalCommentsPages &&
    fetchAdminData(
      currentPage,
      currentLimit,
      currentUserPage,
      currentUserLimit,
      currentCommentsPage + 1,
      currentCommentsLimit
    );
}
function updateArticlesTable(e) {
  const t = document.querySelector('#articles tbody');
  t &&
    ((t.innerHTML = ''),
    e.forEach(e => {
      const n = document.createElement('tr');
      n.dataset.articleId = e.id;
      let a = `<span class="status ${e.status || 'published'}">${getStatusText(e.status || 'published')}</span>`;
      e.is_scheduled &&
        (a += ` <span class="status scheduled" title="定时发布">⏰ ${e.published_at ? formatDate(e.published_at) : '未设置'}</span>`);
      const o = e.visibility || 'public',
        c = 'public' === o ? '公开' : '私密',
        s = 'public' === o ? 'visibility-public' : 'visibility-private';
      ((n.innerHTML = `\n      <td>\n        <input type="checkbox" class="article-checkbox" data-id="${e.id}">\n      </td>\n      <td>${e.title}</td>\n      <td>管理员</td>\n      <td>${formatDate(e.created_at) || '2024-01-01'}</td>\n      <td>${a}</td>\n      <td><span class="visibility ${s}">${c}</span></td>\n      <td class="action-buttons">\n        <button class="btn btn-sm btn-view" data-action="view" data-id="${e.id}">查看</button>\n        <button class="btn btn-sm btn-edit" data-action="edit" data-id="${e.id}">编辑</button>\n        <button class="btn btn-sm btn-upload" data-action="upload" data-id="${e.id}">上传附件</button>\n        <button class="btn btn-sm btn-delete" data-action="delete" data-id="${e.id}">删除</button>\n      </td>\n    `),
        n.addEventListener('click', e => {
          if (
            e.target.closest('.action-buttons') ||
            e.target.classList.contains('article-checkbox')
          )
            return;
          const t = n.querySelector('.article-checkbox');
          ((t.checked = !t.checked),
            n.classList.toggle('selected', t.checked),
            updateBatchActionsBar());
        }));
      (n.querySelector('.article-checkbox').addEventListener('change', e => {
        (n.classList.toggle('selected', e.target.checked), updateBatchActionsBar());
      }),
        t.appendChild(n));
    }),
    bindActionButtons(),
    bindBatchActionEvents());
}
function bindBatchActionEvents() {
  const e = document.getElementById('selectAllCheckbox'),
    t = document.getElementById('batchDeleteBtn'),
    n = document.getElementById('clearSelectionBtn');
  (e &&
    e.addEventListener('change', e => {
      (document.querySelectorAll('.article-checkbox').forEach(t => {
        t.checked = e.target.checked;
        const n = t.closest('tr');
        n && n.classList.toggle('selected', e.target.checked);
      }),
        updateBatchActionsBar());
    }),
    t && t.addEventListener('click', batchDeleteArticles),
    n && n.addEventListener('click', clearSelection));
}
function updateBatchActionsBar() {
  const e = document.querySelectorAll('.article-checkbox:checked'),
    t = document.getElementById('batchActionsBar'),
    n = document.getElementById('selectedCount'),
    a = document.getElementById('selectAllCheckbox'),
    o = e.length;
  if (
    (o > 0
      ? ((t.style.display = 'flex'), (n.textContent = `已选择 ${o} 篇文章`))
      : (t.style.display = 'none'),
    a)
  ) {
    const e = document.querySelectorAll('.article-checkbox');
    e.length > 0 && o === e.length
      ? ((a.checked = !0), (a.indeterminate = !1))
      : o > 0
        ? ((a.checked = !1), (a.indeterminate = !0))
        : ((a.checked = !1), (a.indeterminate = !1));
  }
}
async function batchDeleteArticles() {
  const e = document.querySelectorAll('.article-checkbox:checked'),
    t = Array.from(e).map(e => parseInt(e.dataset.id));
  0 !== t.length
    ? ((currentAction = 'batch-delete-articles'),
      (currentItemId = t.join(',')),
      (document.getElementById('confirmMessage').textContent =
        `确定要删除选中的 ${t.length} 篇文章吗？此操作不可恢复！`),
      openModal('confirmModal'))
    : showToast('请选择要删除的文章', 'warning');
}
function clearSelection() {
  document.querySelectorAll('.article-checkbox').forEach(e => {
    e.checked = !1;
    const t = e.closest('tr');
    t && t.classList.remove('selected');
  });
  const e = document.getElementById('selectAllCheckbox');
  (e && ((e.checked = !1), (e.indeterminate = !1)), updateBatchActionsBar());
}
function bindUserBatchActionEvents() {
  const e = document.getElementById('selectAllUsersCheckbox'),
    t = document.getElementById('batchDeleteUsersBtn'),
    n = document.getElementById('clearUserSelectionBtn');
  (e &&
    e.addEventListener('change', e => {
      (document.querySelectorAll('.user-checkbox:not(:disabled)').forEach(t => {
        t.checked = e.target.checked;
        const n = t.closest('tr');
        n && n.classList.toggle('selected', e.target.checked);
      }),
        updateUserBatchActionsBar());
    }),
    t && t.addEventListener('click', batchDeleteUsers),
    n && n.addEventListener('click', clearUserSelection));
}
function updateUserBatchActionsBar() {
  const e = document.querySelectorAll('.user-checkbox:checked'),
    t = document.getElementById('userBatchActionsBar'),
    n = document.getElementById('userSelectedCount'),
    a = document.getElementById('selectAllUsersCheckbox'),
    o = e.length;
  if (
    (o > 0
      ? ((t.style.display = 'flex'), (n.textContent = `已选择 ${o} 个用户`))
      : (t.style.display = 'none'),
    a)
  ) {
    const e = document.querySelectorAll('.user-checkbox:not(:disabled)');
    e.length > 0 && o === e.length
      ? ((a.checked = !0), (a.indeterminate = !1))
      : o > 0
        ? ((a.checked = !1), (a.indeterminate = !0))
        : ((a.checked = !1), (a.indeterminate = !1));
  }
}
async function batchDeleteUsers() {
  const e = document.querySelectorAll('.user-checkbox:checked'),
    t = Array.from(e).map(e => parseInt(e.dataset.id));
  0 !== t.length
    ? ((currentAction = 'batch-delete-users'),
      (currentItemId = t.join(',')),
      (document.getElementById('confirmMessage').textContent =
        `确定要删除选中的 ${t.length} 个用户吗？此操作不可恢复！`),
      openModal('confirmModal'))
    : showToast('请选择要删除的用户', 'warning');
}
function clearUserSelection() {
  document.querySelectorAll('.user-checkbox').forEach(e => {
    e.checked = !1;
    const t = e.closest('tr');
    t && t.classList.remove('selected');
  });
  const e = document.getElementById('selectAllUsersCheckbox');
  (e && ((e.checked = !1), (e.indeterminate = !1)), updateUserBatchActionsBar());
}
function formatDate(e) {
  if (!e) return '';
  return new Date(e).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}
function updateUsersTable(e) {
  const t = document.querySelector('#users tbody');
  t &&
    ((t.innerHTML = ''),
    e.forEach(e => {
      const n = document.createElement('tr');
      n.dataset.userId = e.id;
      const a = 'admin' === e.role ? '管理员' : 'editor' === e.role ? '编辑' : '普通用户',
        o = 'active' === e.status ? '正常' : 'restricted' === e.status ? '受限' : '禁用',
        c = 'active' === e.status ? '#00b894' : 'restricted' === e.status ? '#fdcb6e' : '#e74c3c';
      ((n.innerHTML = `\n      <td>\n        <input type="checkbox" class="user-checkbox" data-id="${e.id}" ${'admin' === e.role ? 'disabled' : ''}>\n      </td>\n      <td>${e.username}</td>\n      <td>${e.email}</td>\n      <td>${e.created_at || '2024-01-01'}</td>\n      <td>${a}</td>\n      <td><span style="color:${c};">${o}</span></td>\n      <td class="action-buttons">\n        <button class="btn btn-sm btn-view" data-action="view-user" data-id="${e.id}">详情</button>\n        <button class="btn btn-sm btn-edit" data-action="edit-user" data-id="${e.id}">编辑</button>\n        ${'admin' !== e.role ? `<button class="btn btn-sm btn-delete" data-action="delete-user" data-id="${e.id}">删除</button>` : ''}\n      </td>\n    `),
        n.addEventListener('click', e => {
          if (e.target.closest('.action-buttons') || e.target.classList.contains('user-checkbox'))
            return;
          const t = n.querySelector('.user-checkbox');
          t.disabled ||
            ((t.checked = !t.checked),
            n.classList.toggle('selected', t.checked),
            updateUserBatchActionsBar());
        }));
      (n.querySelector('.user-checkbox').addEventListener('change', e => {
        (n.classList.toggle('selected', e.target.checked), updateUserBatchActionsBar());
      }),
        t.appendChild(n));
    }),
    bindActionButtons(),
    bindUserBatchActionEvents());
}
function updateStatCard(e, t) {
  const n = document.getElementById(e);
  n && (n.textContent = t);
}
function getAuthHeaders() {
  const e = localStorage.getItem('auth_token');
  return e ? { Authorization: `Bearer ${e}` } : {};
}
let viewTrendChart = null;
async function loadMostViewedArticles() {
  const e = document.getElementById('mostViewedLimit')?.value || 10,
    t = document.getElementById('mostViewedTableBody');
  if (t) {
    t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载中...</td></tr>';
    try {
      const n = await fetch(`/api/admin/analytics?action=most-viewed&limit=${e}`, {
          headers: getAuthHeaders(),
        }),
        a = await n.json();
      if (a.success && a.data) {
        if (0 === a.data.length)
          return void (t.innerHTML =
            '<tr><td colspan="4" style="text-align: center;">暂无数据</td></tr>');
        t.innerHTML = a.data
          .map(
            (e, t) =>
              `\n        <tr>\n          <td>${t + 1}</td>\n          <td>${e.title}</td>\n          <td>${e.author}</td>\n          <td>${e.view_count}</td>\n        </tr>\n      `
          )
          .join('');
      } else t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>';
    } catch (e) {
      (console.error('加载热门文章失败:', e),
        (t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>'));
    }
  }
}
async function loadViewSources() {
  const e = document.getElementById('viewSourcesDays')?.value || 30,
    t = document.getElementById('viewSourcesTableBody');
  if (t) {
    t.innerHTML = '<tr><td colspan="3" style="text-align: center;">加载中...</td></tr>';
    try {
      const n = await fetch(`/api/admin/analytics?action=view-sources&days=${e}`, {
          headers: getAuthHeaders(),
        }),
        a = await n.json();
      if (a.success && a.data) {
        if (0 === a.data.length)
          return void (t.innerHTML =
            '<tr><td colspan="3" style="text-align: center;">暂无数据</td></tr>');
        t.innerHTML = a.data
          .map(
            (e, t) =>
              `\n        <tr>\n          <td>${t + 1}</td>\n          <td>${'unknown' === e.country ? '未知' : e.country}</td>\n          <td>${e.count}</td>\n        </tr>\n      `
          )
          .join('');
      } else t.innerHTML = '<tr><td colspan="3" style="text-align: center;">加载失败</td></tr>';
    } catch (e) {
      (console.error('加载访问来源失败:', e),
        (t.innerHTML = '<tr><td colspan="3" style="text-align: center;">加载失败</td></tr>'));
    }
  }
}
async function loadViewByCity() {
  const e = document.getElementById('viewByCityDays')?.value || 30,
    t = document.getElementById('viewByCityTableBody');
  if (t) {
    t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载中...</td></tr>';
    try {
      const n = await fetch(`/api/admin/analytics?action=view-by-city&days=${e}`, {
          headers: getAuthHeaders(),
        }),
        a = await n.json();
      if (a.success && a.data) {
        if (0 === a.data.length)
          return void (t.innerHTML =
            '<tr><td colspan="4" style="text-align: center;">暂无数据</td></tr>');
        t.innerHTML = a.data
          .map(
            (e, t) =>
              `\n        <tr>\n          <td>${t + 1}</td>\n          <td>${'unknown' === e.city ? '未知' : e.city}</td>\n          <td>${'unknown' === e.country ? '未知' : e.country}</td>\n          <td>${e.count}</td>\n        </tr>\n      `
          )
          .join('');
      } else t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>';
    } catch (e) {
      (console.error('加载城市统计失败:', e),
        (t.innerHTML = '<tr><td colspan="4" style="text-align: center;">加载失败</td></tr>'));
    }
  }
}
async function loadViewByIP() {
  const e = document.getElementById('viewByIPDays')?.value || 30,
    t = document.getElementById('viewByIPTableBody');
  if (t) {
    t.innerHTML = '<tr><td colspan="8" style="text-align: center;">加载中...</td></tr>';
    try {
      const n = await fetch(`/api/admin/analytics?action=view-by-ip&days=${e}`, {
          headers: getAuthHeaders(),
        }),
        a = await n.json();
      if (a.success && a.data) {
        if (0 === a.data.length)
          return void (t.innerHTML =
            '<tr><td colspan="8" style="text-align: center;">暂无数据</td></tr>');
        t.innerHTML = a.data
          .map(
            (e, t) =>
              `\n        <tr>\n          <td>${t + 1}</td>\n          <td>${e.ip}</td>\n          <td>${'unknown' === e.country ? '未知' : e.country || '-'}</td>\n          <td>${'unknown' === e.city ? '未知' : e.city || '-'}</td>\n          <td>${'unknown' === e.region ? '未知' : e.region || '-'}</td>\n          <td>${e.count}</td>\n          <td>${e.firstVisit}</td>\n          <td>${e.lastVisit}</td>\n        </tr>\n      `
          )
          .join('');
      } else t.innerHTML = '<tr><td colspan="8" style="text-align: center;">加载失败</td></tr>';
    } catch (e) {
      (console.error('加载IP统计失败:', e),
        (t.innerHTML = '<tr><td colspan="8" style="text-align: center;">加载失败</td></tr>'));
    }
  }
}
async function loadViewTrend() {
  const e = document.getElementById('viewTrendDays')?.value || 30;
  if (document.getElementById('viewTrendChart'))
    try {
      const t = await fetch(`/api/admin/analytics?action=view-trend&days=${e}`, {
          headers: getAuthHeaders(),
        }),
        n = await t.json();
      n.success && n.data && drawViewTrendChart(n.data);
    } catch (e) {
      console.error('加载阅读趋势失败:', e);
    }
}
function drawViewTrendChart(e) {
  const t = document.getElementById('viewTrendChart');
  if (!t) return;
  const n = t.getContext('2d');
  if ((n.clearRect(0, 0, t.width, t.height), !e || 0 === e.length))
    return (
      (n.font = '16px Arial'),
      (n.fillStyle = 'rgba(255, 255, 255, 0.7)'),
      (n.textAlign = 'center'),
      void n.fillText('暂无数据', t.width / 2, t.height / 2)
    );
  const a = 50,
    o = t.width - 100,
    c = t.height - 100,
    s = Math.max(...e.map(e => e.count));
  ((n.strokeStyle = 'rgba(255, 255, 255, 0.3)'),
    (n.lineWidth = 1),
    n.beginPath(),
    n.moveTo(a, a),
    n.lineTo(a, t.height - a),
    n.lineTo(t.width - a, t.height - a),
    n.stroke());
  const d = o / (e.length - 1 || 1);
  (n.beginPath(),
    (n.strokeStyle = 'rgba(255, 255, 255, 0.8)'),
    (n.lineWidth = 2),
    e.forEach((e, o) => {
      const i = a + o * d,
        r = t.height - a - (e.count / s) * c;
      0 === o ? n.moveTo(i, r) : n.lineTo(i, r);
    }),
    n.stroke(),
    e.forEach((e, o) => {
      const i = a + o * d,
        r = t.height - a - (e.count / s) * c;
      (n.beginPath(),
        (n.fillStyle = 'rgba(255, 255, 255, 0.9)'),
        n.arc(i, r, 4, 0, 2 * Math.PI),
        n.fill(),
        (n.fillStyle = 'rgba(255, 255, 255, 0.8)'),
        (n.font = '12px Arial'),
        (n.textAlign = 'center'),
        n.fillText(e.count, i, r - 10),
        (n.fillStyle = 'rgba(255, 255, 255, 0.6)'),
        (n.font = '10px Arial'),
        e.date &&
          'string' == typeof e.date &&
          n.fillText(e.date.substring(5), i, t.height - a + 15));
    }));
}
function initAnalytics() {
  const e = document.getElementById('mostViewedLimit');
  e && e.addEventListener('change', loadMostViewedArticles);
  const t = document.getElementById('viewSourcesDays');
  t && t.addEventListener('change', loadViewSources);
  const n = document.getElementById('viewByCityDays');
  n && n.addEventListener('change', loadViewByCity);
  const a = document.getElementById('viewByIPDays');
  a && a.addEventListener('change', loadViewByIP);
  const o = document.getElementById('viewTrendDays');
  (o && o.addEventListener('change', loadViewTrend),
    loadMostViewedArticles(),
    loadViewSources(),
    loadViewByCity(),
    loadViewByIP(),
    loadViewTrend());
}
function getStatusText(e) {
  return { published: '已发布', draft: '草稿', pending: '待审核' }[e] || e;
}
function bindActionButtons() {
  document.querySelectorAll('.action-buttons button').forEach(e => {
    const t = e.cloneNode(!0);
    (e.parentNode.replaceChild(t, e),
      t.addEventListener('click', async function () {
        const e = this.getAttribute('data-action'),
          t = this.getAttribute('data-id');
        if (
          'delete' === e ||
          'delete-comment' === e ||
          'delete-user' === e ||
          'delete-category' === e
        ) {
          ((currentAction = e), (currentItemId = t));
          let n = '';
          ('delete' === e
            ? (n = `确定要删除文章 #${t} 吗？此操作不可撤销。`)
            : 'delete-comment' === e
              ? (n = `确定要删除评论 #${t} 吗？此操作不可撤销。`)
              : 'delete-user' === e
                ? (n = `确定要删除用户 #${t} 吗？此操作不可撤销。`)
                : 'delete-category' === e && (n = `确定要删除分类 #${t} 吗？此操作不可撤销。`),
            (document.getElementById('confirmMessage').textContent = n),
            openModal('confirmModal'));
        } else if ('edit' === e)
          try {
            const e = localStorage.getItem('auth_token'),
              n = { 'Content-Type': 'application/json' };
            e && (n.Authorization = `Bearer ${e}`);
            const a = await fetch(`/api/admin/passages?id=${t}`, { headers: n }),
              o = await a.json();
            if (o.success && o.data) {
              (await populateCategorySelect('editCategory', o.data.category || ''),
                (document.getElementById('editTitle').value = o.data.title || ''),
                (document.getElementById('editAuthor').value = o.data.author || '管理员'),
                (document.getElementById('editContent').value =
                  o.data.original_content || o.data.content || ''),
                (document.getElementById('editShowTitle').checked = !1 !== o.data.show_title),
                (document.getElementById('editCoverImage').value = o.data.cover_image || ''),
                (document.getElementById('editStatus').value = o.data.status || 'published'),
                (document.getElementById('editVisibility').value = o.data.visibility || 'public'));
              const e = o.data.is_scheduled || !1;
              document.getElementById('editIsScheduled').checked = e;
              const n = document.getElementById('editPublishedAtGroup');
              if (e) {
                if (((n.style.display = 'block'), o.data.published_at)) {
                  const e = new Date(o.data.published_at),
                    t = new Date(e.getTime() - 6e4 * e.getTimezoneOffset())
                      .toISOString()
                      .slice(0, 16);
                  document.getElementById('editPublishedAt').value = t;
                }
              } else
                ((n.style.display = 'none'),
                  (document.getElementById('editPublishedAt').value = ''));
              let a = [];
              if (o.data.tags) {
                const e = o.data.tags;
                if (Array.isArray(e)) a = e;
                else if ('string' == typeof e)
                  try {
                    const t = JSON.parse(e);
                    a = Array.isArray(t)
                      ? t
                      : e
                          .split(',')
                          .map(e => e.trim())
                          .filter(e => e);
                  } catch (t) {
                    a = e
                      .split(',')
                      .map(e => e.trim())
                      .filter(e => e);
                  }
              }
              (await populateTagSelector(a),
                (document.getElementById('editForm').dataset.articleId = t),
                openModal('editModal'));
            } else showToast('获取文章详情失败：' + (o.message || '未知错误'), 'error');
          } catch (e) {
            (console.error('获取文章详情失败:', e),
              showToast('获取文章详情失败，请稍后重试', 'error'));
          }
        else if ('upload' === e)
          ((document.getElementById('uploadAttachmentArticleId').textContent = '#' + t),
            (document.getElementById('uploadAttachmentArticleId').dataset.articleId = t),
            (document.getElementById('attachmentFile').value = ''),
            (document.getElementById('uploadAttachmentProgress').style.display = 'none'),
            (document.getElementById('uploadAttachmentResult').style.display = 'none'),
            openModal('uploadAttachmentModal'));
        else if ('view' === e)
          try {
            const e = localStorage.getItem('auth_token'),
              n = { 'Content-Type': 'application/json' };
            e && (n.Authorization = `Bearer ${e}`);
            const a = await fetch(`/api/admin/passages?id=${t}`, { headers: n }),
              o = await a.json();
            o.success && o.data
              ? ((document.getElementById('viewArticleId').textContent = '#' + o.data.id),
                (document.getElementById('viewArticleTitle').textContent = o.data.title || ''),
                (document.getElementById('viewArticleAuthor').textContent =
                  o.data.author || '管理员'),
                (document.getElementById('viewArticleStatus').textContent = getStatusText(
                  o.data.status || 'published'
                )),
                (document.getElementById('viewArticleDate').textContent = o.data.created_at || ''),
                (document.getElementById('viewArticleContent').textContent = o.data.content || ''),
                openModal('viewArticleModal'))
              : showToast('获取文章详情失败：' + (o.message || '未知错误'), 'error');
          } catch (e) {
            (console.error('获取文章详情失败:', e),
              showToast('获取文章详情失败，请稍后重试', 'error'));
          }
        else if ('edit-user' === e)
          try {
            const e = localStorage.getItem('auth_token'),
              n = { 'Content-Type': 'application/json' };
            e && (n.Authorization = `Bearer ${e}`);
            const a = await fetch(`/api/admin/users/${t}`, { headers: n }),
              o = await a.json();
            if (o.success && o.data) {
              const e = o.data;
              ((document.getElementById('editUserName').value = e.username || ''),
                (document.getElementById('editUserEmail').value = e.email || ''),
                (document.getElementById('editUserRole').value = e.role || 'user'),
                (document.getElementById('editUserStatus').value = e.status || 'active'),
                (document.getElementById('editUserPassword').value = ''),
                (document.getElementById('editUserForm').dataset.userId = t),
                openModal('editUserModal'));
            } else showToast('获取用户详情失败：' + (o.message || '未知错误'), 'error');
          } catch (e) {
            (console.error('获取用户详情失败:', e),
              showToast('获取用户详情失败，请稍后重试', 'error'));
          }
        else if ('view-user' === e)
          try {
            const e = localStorage.getItem('auth_token'),
              n = { 'Content-Type': 'application/json' };
            e && (n.Authorization = `Bearer ${e}`);
            const a = await fetch(`/api/admin/users/${t}`, { headers: n }),
              o = await a.json();
            if (o.success && o.data) {
              const e = o.data;
              ((document.getElementById('viewUserId').textContent = '#' + e.id),
                (document.getElementById('viewUserName').textContent = e.username || ''),
                (document.getElementById('viewUserEmail').textContent = e.email || ''),
                (document.getElementById('viewUserRole').textContent = e.role || '普通用户'),
                (document.getElementById('viewUserStatus').textContent = e.status || '正常'),
                (document.getElementById('viewUserDate').textContent = e.created_at || ''),
                openModal('viewUserModal'));
            } else showToast('获取用户详情失败：' + (o.message || '未知错误'), 'error');
          } catch (e) {
            (console.error('获取用户详情失败:', e),
              showToast('获取用户详情失败，请稍后重试', 'error'));
          }
        else if ('edit-comment' === e) alert(`编辑评论 #${t}`);
        else if ('view-comment' === e) {
          const e = this.closest('tr').querySelectorAll('td');
          e.length >= 6
            ? ((document.getElementById('viewCommentId').textContent = e[0].textContent),
              (document.getElementById('viewCommentContent').textContent = e[1].textContent),
              (document.getElementById('viewCommentArticle').textContent = e[2].textContent),
              (document.getElementById('viewCommentUser').textContent = e[3].textContent),
              (document.getElementById('viewCommentDate').textContent = e[4].textContent),
              (document.getElementById('viewCommentStatus').textContent = e[5].textContent.trim()),
              openModal('viewCommentModal'))
            : alert('查看评论 #${itemId} 的详细信息');
        } else if ('delete-tag' === e)
          ((currentAction = e),
            (currentItemId = t),
            (document.getElementById('confirmMessage').textContent =
              `确定要删除标签 #${t} 吗？此操作不可撤销。`),
            openModal('confirmModal'));
        else if ('edit-tag' === e)
          try {
            const e = localStorage.getItem('auth_token'),
              n = { 'Content-Type': 'application/json' };
            e && (n.Authorization = `Bearer ${e}`);
            const a = await fetch(`/api/admin/tags?id=${t}`, { headers: n }),
              o = await a.json();
            o.success && o.data
              ? ((document.getElementById('tagModalTitle').textContent = '编辑标签'),
                (document.getElementById('tagName').value = o.data.name || ''),
                (document.getElementById('tagDescription').value = o.data.description || ''),
                (document.getElementById('tagColor').value = o.data.color || '#007bff'),
                (document.getElementById('tagCategory').value = o.data.category_id || 0),
                (document.getElementById('tagSortOrder').value = o.data.sort_order || 0),
                (document.getElementById('tagEnabled').checked = o.data.is_enabled),
                (document.getElementById('tagForm').dataset.tagId = t),
                openModal('tagModal'))
              : showToast('获取标签详情失败：' + (o.message || '未知错误'), 'error');
          } catch (e) {
            (console.error('获取标签详情失败:', e),
              showToast('获取标签详情失败，请稍后重试', 'error'));
          }
        else if ('edit-category' === e)
          try {
            const e = localStorage.getItem('auth_token'),
              n = { 'Content-Type': 'application/json' };
            e && (n.Authorization = `Bearer ${e}`);
            const a = await fetch(`/api/admin/categories?id=${t}`, { headers: n }),
              o = await a.json();
            o.success && o.data
              ? ((document.getElementById('categoryModalTitle').textContent = '编辑分类'),
                (document.getElementById('categoryName').value = o.data.name || ''),
                (document.getElementById('categoryDescription').value = o.data.description || ''),
                (document.getElementById('categoryIcon').value = o.data.icon || ''),
                (document.getElementById('categorySortOrder').value = o.data.sort_order || 0),
                (document.getElementById('categoryEnabled').checked = o.data.is_enabled),
                (document.getElementById('categoryForm').dataset.categoryId = t),
                openModal('categoryModal'))
              : showToast('获取分类详情失败：' + (o.message || '未知错误'), 'error');
          } catch (e) {
            (console.error('获取分类详情失败:', e),
              showToast('获取分类详情失败，请稍后重试', 'error'));
          }
      }));
  });
}
let lastScrollTop = 0,
  isNavHidden = !1;
const nav = document.getElementById('mainNav'),
  scrollIndicator = document.getElementById('scrollIndicator'),
  scrollProgress = document.getElementById('scrollProgress');
(nav.classList.add('scrolled-top'),
  window.addEventListener(
    'scroll',
    function () {
      const e = window.pageYOffset || document.documentElement.scrollTop,
        t = (e / (document.documentElement.scrollHeight - window.innerHeight)) * 100;
      (e > 100
        ? (scrollIndicator.classList.add('active'), (scrollProgress.style.height = `${t}%`))
        : scrollIndicator.classList.remove('active'),
        e > lastScrollTop && e > 50
          ? isNavHidden || (nav.classList.add('hidden'), (isNavHidden = !0))
          : (e < lastScrollTop || e <= 50) &&
            (isNavHidden && (nav.classList.remove('hidden'), (isNavHidden = !1)),
            0 === e
              ? (nav.classList.add('scrolled-top'), nav.classList.remove('scrolled'))
              : (nav.classList.remove('scrolled-top'), nav.classList.add('scrolled'))),
        (lastScrollTop = e));
    },
    { passive: !0 }
  ),
  window.addEventListener('load', function () {
    ((document.body.style.opacity = '0'),
      (document.body.style.transition = 'opacity 0.5s ease'),
      setTimeout(() => {
        document.body.style.opacity = '1';
      }, 100),
      showEmptyState('articlesTableBody', '暂无文章', 6),
      showEmptyState('usersTableBody', '暂无用户', 7),
      showEmptyState('commentsTableBody', '暂无评论', 6),
      fetchAdminData(),
      loadCategoriesAndTags());
    document.querySelectorAll('.stat-card').forEach((e, t) => {
      ((e.style.opacity = '0'),
        (e.style.transform = 'translateY(20px)'),
        (e.style.transition = 'opacity 0.6s ease, transform 0.6s ease'),
        setTimeout(
          () => {
            ((e.style.opacity = '1'), (e.style.transform = 'translateY(0)'));
          },
          200 + 50 * t
        ));
    });
  }));
const mainTitle = document.getElementById('main-title');
function openModal(e) {
  const t = document.getElementById(e);
  t && (t.classList.add('active'), (document.body.style.overflow = 'hidden'));
}
function closeModal(e) {
  const t = document.getElementById(e);
  t &&
    (t.classList.add('closing'),
    setTimeout(() => {
      (t.classList.remove('active', 'closing'), (document.body.style.overflow = 'auto'));
    }, 300));
}
(mainTitle &&
  (mainTitle.addEventListener('mouseenter', function () {
    this.style.animationPlayState = 'paused';
  }),
  mainTitle.addEventListener('mouseleave', function () {
    this.style.animationPlayState = 'running';
  })),
  document.querySelectorAll('.tab-btn').forEach(e => {
    e.addEventListener('click', async function () {
      (document.querySelectorAll('.tab-btn').forEach(e => {
        e.classList.remove('active');
      }),
        this.classList.add('active'),
        document.querySelectorAll('.tab-pane').forEach(e => {
          e.classList.remove('active');
        }));
      const e = this.getAttribute('data-tab'),
        t = document.getElementById(e);
      t &&
        (t.classList.add('active'),
        'tags' === e && (await loadCategoriesAndTags()),
        'analytics' === e && initAnalytics(),
        'attachments' === e && (await loadAttachments()));
    });
  }),
  document.querySelectorAll('[data-modal]').forEach(e => {
    (e.classList.contains('modal-close') || e.hasAttribute('data-modal')) &&
      e.addEventListener('click', function () {
        closeModal(this.getAttribute('data-modal'));
      });
  }),
  document.getElementById('uploadArticleBtn').addEventListener('click', async () => {
    (await populateTagSelectorForUpload(),
      await populateCategorySelectorForUpload(),
      openModal('uploadModal'));
  }),
  document.getElementById('newArticleBtn').addEventListener('click', async () => {
    (await populateTagSelectorForNewArticle(),
      await populateCategorySelectorForNewArticle(),
      openModal('articleModal'));
  }),
  document.getElementById('addUserBtn').addEventListener('click', () => {
    openModal('userModal');
  }),
  document.querySelectorAll('.tag-option').forEach(e => {
    e.addEventListener('click', function () {
      this.classList.toggle('selected');
    });
  }),
  document.getElementById('addUploadTagBtn').addEventListener('click', async function () {
    const e = document.getElementById('uploadTagInput'),
      t = e.value.trim();
    if (!t) return void showToast('请输入标签名称', 'warning');
    const n = document.getElementById('uploadTagSelector');
    if (
      Array.from(n.querySelectorAll('.tag-option'))
        .map(e =>
          e.dataset.tagName ? e.dataset.tagName.toLowerCase() : e.textContent.trim().toLowerCase()
        )
        .includes(t.toLowerCase())
    )
      return (showToast('标签已存在', 'warning'), void (e.value = ''));
    const a = [
        '#e74c3c',
        '#e67e22',
        '#f1c40f',
        '#2ecc71',
        '#1abc9c',
        '#3498db',
        '#9b59b6',
        '#34495e',
      ],
      o = a[Math.floor(Math.random() * a.length)],
      c = document.createElement('div');
    ((c.className = 'tag-option selected'),
      (c.dataset.tagName = t),
      (c.dataset.isNew = 'true'),
      (c.innerHTML = `\n    <span style="display: inline-block; width: 12px; height: 12px; background-color: ${o}; border-radius: 2px; margin-right: 6px;"></span>\n    ${t}\n  `),
      c.addEventListener('click', function () {
        this.classList.toggle('selected');
      }),
      n.appendChild(c),
      (e.value = ''),
      showToast(`标签 "${t}" 已添加`, 'success'));
  }),
  document.getElementById('addUploadCategoryBtn').addEventListener('click', async function () {
    const e = document.getElementById('uploadCategoryInput'),
      t = e.value.trim();
    if (!t) return void showToast('请输入分类名称', 'warning');
    const n = document.getElementById('uploadCategorySelector');
    if (
      Array.from(n.querySelectorAll('.category-option'))
        .map(e =>
          e.dataset.categoryName
            ? e.dataset.categoryName.toLowerCase()
            : e.textContent.trim().toLowerCase()
        )
        .includes(t.toLowerCase())
    )
      return (showToast('分类已存在', 'warning'), void (e.value = ''));
    const a = document.createElement('div');
    ((a.className = 'category-option selected'),
      (a.dataset.categoryName = t),
      (a.dataset.isNew = 'true'),
      (a.innerHTML = `\n    <span style="display: inline-block; width: 12px; height: 12px; background-color: #007bff; border-radius: 2px; margin-right: 6px;"></span>\n    ${t}\n  `),
      a.addEventListener('click', function () {
        this.classList.contains('selected')
          ? this.classList.remove('selected')
          : (n.querySelectorAll('.category-option').forEach(e => {
              e.classList.remove('selected');
            }),
            this.classList.add('selected'));
      }),
      n.querySelectorAll('.category-option').forEach(e => {
        e.classList.remove('selected');
      }),
      n.appendChild(a),
      (e.value = ''),
      showToast(`分类 "${t}" 已添加`, 'success'));
  }));
const uploadArea = document.getElementById('uploadArea'),
  fileInput = document.getElementById('fileInput'),
  uploadPreview = document.getElementById('uploadPreview');
let selectedFiles = [];
function handleFiles(e) {
  for (let t = 0; t < e.length; t++) {
    const n = e[t];
    if (n.size > 10485760) {
      showToast(`文件 ${n.name} 超过10MB限制`, 'error');
      continue;
    }
    selectedFiles.push(n);
    const a = new FileReader();
    ((a.onload = function (e) {
      const t = document.createElement('div');
      if (
        ((t.className = 'upload-item'),
        (t.dataset.fileIndex = selectedFiles.length - 1),
        n.type.startsWith('image/'))
      )
        t.innerHTML = `\n          <img src="${e.target.result}" alt="${n.name}">\n          <button class="upload-remove">×</button>\n        `;
      else {
        const e = n.name.split('.').pop().toUpperCase();
        t.innerHTML = `\n          <div style="background:#f0f0f0; width:100%; height:100%; display:flex; flex-direction:column; align-items:center; justify-content:center; color:#666;">\n            <div style="font-size:2em;">📄</div>\n            <div style="font-size:0.8em; margin-top:5px; text-align:center; padding:0 5px;">${e}</div>\n          </div>\n          <button class="upload-remove">×</button>\n        `;
      }
      (t.querySelector('.upload-remove').addEventListener('click', function () {
        const e = parseInt(t.dataset.fileIndex);
        (selectedFiles.splice(e, 1), t.remove());
      }),
        uploadPreview.appendChild(t));
    }),
      a.readAsDataURL(n));
  }
}
function readFileContent(e) {
  return new Promise((t, n) => {
    const a = new FileReader();
    ((a.onload = e => {
      t(e.target.result);
    }),
      (a.onerror = e => {
        n(new Error('读取文件失败'));
      }),
      a.readAsText(e));
  });
}
(uploadArea.addEventListener('click', () => {
  fileInput.click();
}),
  uploadArea.addEventListener('dragover', e => {
    (e.preventDefault(), uploadArea.classList.add('dragover'));
  }),
  uploadArea.addEventListener('dragleave', () => {
    uploadArea.classList.remove('dragover');
  }),
  uploadArea.addEventListener('drop', e => {
    (e.preventDefault(), uploadArea.classList.remove('dragover'));
    handleFiles(e.dataTransfer.files);
  }),
  fileInput.addEventListener('change', () => {
    handleFiles(fileInput.files);
  }),
  document.getElementById('uploadForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = document.getElementById('uploadTitle').value.trim();
    if (!t)
      return (
        showToast('请输入文章标题', 'warning'),
        void document.getElementById('uploadTitle').focus()
      );
    const n = document.getElementById('uploadAuthor').value.trim();
    if (!n)
      return (
        showToast('请输入作者名称', 'warning'),
        void document.getElementById('uploadAuthor').focus()
      );
    const a = document.querySelector('#uploadCategorySelector .category-option.selected'),
      o = a ? a.dataset.categoryName : '',
      c = [];
    document.querySelectorAll('#uploadTagSelector .tag-option.selected').forEach(e => {
      c.push(e.dataset.tagName);
    });
    const s = this.querySelector('button[type="submit"]'),
      d = s.textContent;
    ((s.textContent = '上传中...'), (s.disabled = !0));
    try {
      const e = selectedFiles,
        a = document.getElementById('uploadContent').value.trim();
      if (0 === e.length && !a)
        return (
          showToast('请选择要上传的文件或输入文章内容', 'warning'),
          (s.textContent = d),
          void (s.disabled = !1)
        );
      let i = a;
      if (e.length > 0 && !a) {
        const t = e[0];
        i = await readFileContent(t);
      }
      if (!i || 0 === i.trim().length)
        return (
          showToast('文章内容不能为空', 'warning'),
          (s.textContent = d),
          void (s.disabled = !1)
        );
      const r = document.getElementById('uploadStatus').value,
        l = document.getElementById('uploadYear').value,
        u = document.getElementById('uploadMonth').value,
        m = document.getElementById('uploadDay').value;
      let g = null;
      if (l && u && m) {
        const e = parseInt(l),
          t = parseInt(u),
          n = parseInt(m);
        e >= 2020 &&
          e <= 2030 &&
          t >= 1 &&
          t <= 12 &&
          n >= 1 &&
          n <= 31 &&
          (g = new Date(e, t - 1, n).toISOString());
      }
      const h = localStorage.getItem('auth_token'),
        y = { 'Content-Type': 'application/json' };
      h && (y.Authorization = `Bearer ${h}`);
      const p = { title: t, content: i, author: n, category: o, tags: c.join(','), status: r };
      g && (p.created_at = g);
      const f = await fetch('/api/admin/passages', {
          method: 'POST',
          headers: y,
          body: JSON.stringify(p),
        }),
        b = await f.json();
      if (!b.success)
        return (
          (s.textContent = d),
          (s.disabled = !1),
          void showToast('创建失败：' + (b.message || '未知错误'), 'error')
        );
      const E = b.data.id;
      let v = 0,
        B = 0;
      const I = [];
      for (const t of e)
        try {
          const e = new FormData();
          (e.append('file', t), e.append('passage_id', E));
          const n = await fetch('/api/admin/attachments', {
              method: 'POST',
              headers: { Authorization: `Bearer ${h}` },
              body: e,
            }),
            a = await n.json();
          a.success
            ? (v++, console.log(`附件 ${t.name} 上传成功`))
            : (B++,
              I.push(`${t.name}: ${a.message || '未知错误'}`),
              console.error(`附件 ${t.name} 上传失败:`, a));
        } catch (e) {
          (B++,
            I.push(`${t.name}: ${e.message || '网络错误'}`),
            console.error(`上传附件 ${t.name} 失败:`, e));
        }
      (B > 0 && console.error('附件上传失败详情:', I),
        (s.textContent = '创建成功!'),
        (s.style.background = 'rgba(255, 183, 122, 0.8)'));
      let C = '文章创建成功！';
      (v > 0 && (C += ` 成功上传 ${v} 个附件`),
        B > 0 && (C += ` 失败 ${B} 个附件`),
        setTimeout(() => {
          (closeModal('uploadModal'),
            (s.textContent = d),
            (s.disabled = !1),
            (s.style.background = 'rgba(255, 183, 122, 0.8)'),
            this.reset(),
            (uploadPreview.innerHTML = ''),
            (selectedFiles = []),
            fetchAdminData(),
            showToast(C, B > 0 ? 'warning' : 'success'));
        }, 2e3));
    } catch (e) {
      (console.error('创建文章失败:', e),
        (s.textContent = d),
        (s.disabled = !1),
        showToast('创建失败，无法连接到服务器，请检查网络连接后重试', 'error'));
    }
  }),
  document.getElementById('articleForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = document.getElementById('articleTitle').value;
    if (!t) return void showToast('请输入文章标题', 'warning');
    const n = this.querySelector('button[type="submit"]'),
      a = n.textContent;
    ((n.textContent = '保存中...'), (n.disabled = !0));
    try {
      const e = localStorage.getItem('auth_token'),
        o = { 'Content-Type': 'application/json' };
      e && (o.Authorization = `Bearer ${e}`);
      const c = await fetch('/api/admin/passages', {
          method: 'POST',
          headers: o,
          body: JSON.stringify({
            title: t,
            content: document.getElementById('articleContent').value,
            author: document.getElementById('articleAuthor').value,
            category: document.getElementById('articleCategory').value,
            tags: document.getElementById('articleTags').value,
            cover_image: document.getElementById('articleCoverImage').value,
          }),
        }),
        s = await c.json();
      s.success
        ? ((n.textContent = '保存成功!'),
          (n.style.background = 'rgba(255, 183, 122, 0.8)'),
          setTimeout(() => {
            (closeModal('articleModal'),
              (n.textContent = a),
              (n.disabled = !1),
              (n.style.background = 'rgba(255, 183, 122, 0.8)'),
              this.reset(),
              fetchAdminData(),
              showToast('文章创建成功！', 'success'));
          }, 2e3))
        : ((n.textContent = a),
          (n.disabled = !1),
          showToast('保存失败：' + (s.message || '未知错误'), 'error'));
    } catch (e) {
      (console.error('保存文章失败:', e),
        (n.textContent = a),
        (n.disabled = !1),
        showToast('保存失败，请稍后重试', 'error'));
    }
  }),
  document.getElementById('editForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = this.dataset.articleId;
    if (!t) return void showToast('无法确定要编辑的文章', 'error');
    const n = this.querySelector('button[type="submit"]'),
      a = n.textContent;
    ((n.textContent = '保存中...'), (n.disabled = !0));
    try {
      const e = localStorage.getItem('auth_token'),
        o = { 'Content-Type': 'application/json' };
      e && (o.Authorization = `Bearer ${e}`);
      const c = document.getElementById('editIsScheduled').checked;
      let s = null;
      if (c) {
        const e = document.getElementById('editPublishedAt').value;
        e && (s = new Date(e).toISOString());
      }
      const d = await fetch(`/api/admin/passages?id=${t}`, {
          method: 'PUT',
          headers: o,
          body: JSON.stringify({
            title: document.getElementById('editTitle').value,
            content: document.getElementById('editContent').value,
            author: document.getElementById('editAuthor').value,
            category: document.getElementById('editCategory').value,
            show_title: document.getElementById('editShowTitle').checked,
            tags: document.getElementById('editTags').value,
            status: document.getElementById('editStatus').value,
            visibility: document.getElementById('editVisibility').value,
            is_scheduled: c,
            published_at: s,
            cover_image: document.getElementById('editCoverImage').value || void 0,
          }),
        }),
        i = await d.json();
      i.success
        ? ((n.textContent = '保存成功!'),
          (n.style.background = 'rgba(255, 183, 122, 0.8)'),
          setTimeout(() => {
            (closeModal('editModal'),
              (n.textContent = a),
              (n.disabled = !1),
              (n.style.background = 'rgba(255, 183, 122, 0.8)'),
              fetchAdminData(),
              showToast('文章修改已保存！', 'success'));
          }, 2e3))
        : ((n.textContent = a),
          (n.disabled = !1),
          showToast('保存失败：' + (i.message || '未知错误'), 'error'));
    } catch (e) {
      (console.error('保存文章失败:', e),
        (n.textContent = a),
        (n.disabled = !1),
        showToast('保存失败，请稍后重试', 'error'));
    }
  }),
  document.getElementById('editIsScheduled').addEventListener('change', function (e) {
    const t = document.getElementById('editPublishedAtGroup'),
      n = document.getElementById('editPublishedAt');
    if (e.target.checked) {
      if (((t.style.display = 'block'), !n.value)) {
        const e = new Date();
        e.setDate(e.getDate() + 1);
        const t = new Date(e.getTime() - 6e4 * e.getTimezoneOffset()).toISOString().slice(0, 16);
        n.value = t;
      }
    } else ((t.style.display = 'none'), (n.value = ''));
  }),
  document.getElementById('userForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = this.querySelector('button[type="submit"]'),
      n = t.textContent;
    ((t.textContent = '创建中...'), (t.disabled = !0));
    const a = document.getElementById('userName').value.trim(),
      o = document.getElementById('userEmail').value.trim(),
      c = document.getElementById('userPassword').value,
      s = document.getElementById('userRole').value;
    try {
      const e = await fetch('/api/register', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', ...getAuthHeaders() },
          body: JSON.stringify({ username: a, email: o, password: c, role: s }),
        }),
        t = await e.json();
      t.success
        ? (showToast('用户创建成功！', 'success'),
          closeModal('userModal'),
          this.reset(),
          fetchAdminData(currentPage, 10, currentUserPage, 10))
        : showToast(t.message || '用户创建失败', 'error');
    } catch (e) {
      (console.error('创建用户失败:', e), showToast('创建用户失败，请重试', 'error'));
    } finally {
      ((t.textContent = n), (t.disabled = !1));
    }
  }),
  document.getElementById('editUserForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = this.dataset.userId;
    if (!t) return void showToast('无法确定要编辑的用户', 'error');
    const n = this.querySelector('button[type="submit"]'),
      a = n.textContent;
    ((n.textContent = '保存中...'), (n.disabled = !0));
    try {
      const e = localStorage.getItem('auth_token'),
        o = { 'Content-Type': 'application/json' };
      e && (o.Authorization = `Bearer ${e}`);
      const c = {
          username: document.getElementById('editUserName').value,
          email: document.getElementById('editUserEmail').value,
          role: document.getElementById('editUserRole').value,
          status: document.getElementById('editUserStatus').value,
        },
        s = document.getElementById('editUserPassword').value;
      s && (c.password = s);
      const d = await fetch(`/api/admin/users/${t}`, {
          method: 'PATCH',
          headers: o,
          body: JSON.stringify(c),
        }),
        i = await d.json();
      i.success
        ? ((n.textContent = '保存成功!'),
          (n.style.background = 'rgba(255, 183, 122, 0.8)'),
          setTimeout(() => {
            (closeModal('editUserModal'),
              (n.textContent = a),
              (n.disabled = !1),
              (n.style.background = 'rgba(255, 183, 122, 0.8)'),
              fetchAdminData(),
              showToast('用户修改已保存！', 'success'));
          }, 2e3))
        : ((n.textContent = a),
          (n.disabled = !1),
          showToast('保存失败：' + (i.message || '未知错误'), 'error'));
    } catch (e) {
      (console.error('保存用户失败:', e),
        (n.textContent = a),
        (n.disabled = !1),
        showToast('保存失败，请稍后重试', 'error'));
    }
  }),
  document.getElementById('categoryForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = this.dataset.categoryId,
      n = this.querySelector('button[type="submit"]'),
      a = n.textContent;
    ((n.textContent = '保存中...'), (n.disabled = !0));
    try {
      const e = localStorage.getItem('auth_token'),
        o = { 'Content-Type': 'application/json' };
      e && (o.Authorization = `Bearer ${e}`);
      const c = {
        name: document.getElementById('categoryName').value,
        description: document.getElementById('categoryDescription').value,
        icon: document.getElementById('categoryIcon').value,
        sort_order: parseInt(document.getElementById('categorySortOrder').value) || 0,
        is_enabled: document.getElementById('categoryEnabled').checked,
      };
      let s = '/api/admin/categories',
        d = 'POST';
      t && ((s += `?id=${t}`), (d = 'PUT'));
      const i = await fetch(s, { method: d, headers: o, body: JSON.stringify(c) }),
        r = await i.json();
      r.success
        ? ((n.textContent = '保存成功!'),
          (n.style.background = 'rgba(255, 183, 122, 0.8)'),
          setTimeout(() => {
            (closeModal('categoryModal'),
              (n.textContent = a),
              (n.disabled = !1),
              (n.style.background = 'rgba(255, 183, 122, 0.8)'),
              this.reset(),
              delete this.dataset.categoryId,
              loadCategoriesAndTags(),
              showToast(t ? '分类修改已保存！' : '分类创建成功！', 'success'));
          }, 2e3))
        : ((n.textContent = a),
          (n.disabled = !1),
          showToast('保存失败：' + (r.message || '未知错误'), 'error'));
    } catch (e) {
      (console.error('保存分类失败:', e),
        (n.textContent = a),
        (n.disabled = !1),
        showToast('保存失败，请稍后重试', 'error'));
    }
  }),
  document.getElementById('tagForm').addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = this.dataset.tagId,
      n = this.querySelector('button[type="submit"]'),
      a = n.textContent;
    ((n.textContent = '保存中...'), (n.disabled = !0));
    try {
      const e = localStorage.getItem('auth_token'),
        o = { 'Content-Type': 'application/json' };
      e && (o.Authorization = `Bearer ${e}`);
      const c = {
        name: document.getElementById('tagName').value,
        description: document.getElementById('tagDescription').value,
        color: document.getElementById('tagColor').value,
        category_id: parseInt(document.getElementById('tagCategory').value) || 0,
        sort_order: parseInt(document.getElementById('tagSortOrder').value) || 0,
        is_enabled: document.getElementById('tagEnabled').checked,
      };
      let s = '/api/admin/tags',
        d = 'POST';
      t && ((s += `?id=${t}`), (d = 'PUT'));
      const i = await fetch(s, { method: d, headers: o, body: JSON.stringify(c) }),
        r = await i.json();
      r.success
        ? ((n.textContent = '保存成功!'),
          (n.style.background = 'rgba(255, 183, 122, 0.8)'),
          setTimeout(() => {
            (closeModal('tagModal'),
              (n.textContent = a),
              (n.disabled = !1),
              (n.style.background = 'rgba(255, 183, 122, 0.8)'),
              this.reset(),
              delete this.dataset.tagId,
              loadCategoriesAndTags(),
              showToast(t ? '标签修改已保存！' : '标签创建成功！', 'success'));
          }, 2e3))
        : ((n.textContent = a),
          (n.disabled = !1),
          showToast('保存失败：' + (r.message || '未知错误'), 'error'));
    } catch (e) {
      (console.error('保存标签失败:', e),
        (n.textContent = a),
        (n.disabled = !1),
        showToast('保存失败，请稍后重试', 'error'));
    }
  }));
let currentAction = null,
  currentItemId = null,
  selectedAttachments = new Set();
async function handleBatchDelete(e, t) {
  const n = t.split(',').map(e => parseInt(e.trim()));
  let a = '',
    o = '',
    c = null;
  switch (e) {
    case 'batch-delete-comments':
      ((a = '/api/admin/comments/batch-delete'),
        (o = '批量删除评论成功'),
        (c = () => {
          (clearCommentSelection(),
            fetchAdminData(
              currentPage,
              currentLimit,
              currentUserPage,
              currentUserLimit,
              currentCommentsPage,
              currentCommentsLimit
            ));
        }));
      break;
    case 'batch-delete-categories':
      ((a = '/api/admin/categories/batch-delete'),
        (o = '批量删除分类成功'),
        (c = () => {
          (clearCategorySelection(), loadCategoriesAndTags());
        }));
      break;
    case 'batch-delete-tags':
      ((a = '/api/admin/tags/batch-delete'),
        (o = '批量删除标签成功'),
        (c = () => {
          (clearTagSelection(), loadCategoriesAndTags());
        }));
      break;
    case 'batch-delete-users':
      ((a = '/api/admin/users/batch-delete'),
        (o = '批量删除用户成功'),
        (c = () => {
          (clearUserSelection(), fetchAdminData(currentPage, 10, currentUserPage, 10));
        }));
      break;
    case 'batch-delete-articles':
      ((a = '/api/admin/passages/batch-delete'),
        (o = '批量删除文章成功'),
        (c = () => {
          (clearSelection(), fetchAdminData(currentPage, 10));
        }));
      break;
    default:
      return;
  }
  try {
    const e = await fetch(a, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...getAuthHeaders() },
        body: JSON.stringify({ ids: n }),
      }),
      t = await e.json();
    t.success
      ? (closeModal('confirmModal'), showToast(t.message || o, 'success'), c && c())
      : showToast(t.message || '批量删除失败', 'error');
  } catch (e) {
    (console.error('批量删除失败:', e), showToast('批量删除失败，请重试', 'error'));
  }
}
(document.getElementById('confirmAction').addEventListener('click', async function () {
  if (currentAction && currentItemId) {
    if (currentAction.startsWith('batch-delete-'))
      return void (await handleBatchDelete(currentAction, currentItemId));
    const e = localStorage.getItem('auth_token'),
      t = { 'Content-Type': 'application/json' };
    e && (t.Authorization = `Bearer ${e}`);
    try {
      let e = '';
      if ('delete' === currentAction) e = `/api/admin/passages?id=${currentItemId}`;
      else if ('delete-comment' === currentAction) e = `/api/admin/comments/${currentItemId}`;
      else if ('delete-user' === currentAction) e = `/api/admin/users/${currentItemId}`;
      else if ('delete-category' === currentAction) e = `/api/admin/categories?id=${currentItemId}`;
      else if ('delete-tag' === currentAction) e = `/api/admin/tags/${currentItemId}`;
      else if ('delete-main-card' === currentAction)
        e = `/api/about/main-cards/delete?id=${currentItemId}`;
      else if ('delete-sub-card' === currentAction)
        e = `/api/about/sub-cards/delete?id=${currentItemId}`;
      else if ('delete-attachment' === currentAction) e = `/api/admin/attachments/${currentItemId}`;
      else if ('batch-delete-attachment' === currentAction) {
        const e = currentItemId.split(',');
        let t = 0,
          n = 0;
        for (const a of e)
          try {
            const e = await fetch(`/api/admin/attachments/${a}`, { method: 'DELETE' });
            (await e.json()).success ? t++ : n++;
          } catch (e) {
            (console.error('删除失败:', e), n++);
          }
        return (
          closeModal('confirmModal'),
          t > 0 && showToast(`成功删除 ${t} 个附件`, 'success'),
          n > 0 && showToast(`${n} 个附件删除失败`, 'error'),
          selectedAttachments.clear(),
          updateBatchActions(),
          void loadAttachments()
        );
      }
      const n = await fetch(e, { method: 'DELETE', headers: t }),
        a = await n.json();
      a.success
        ? (closeModal('confirmModal'),
          showToast('删除成功！', 'success'),
          'delete-category' === currentAction || 'delete-tag' === currentAction
            ? loadCategoriesAndTags()
            : 'delete-main-card' === currentAction
              ? (loadMainCards(), loadSubCards())
              : 'delete-sub-card' === currentAction
                ? loadSubCards()
                : 'delete-attachment' === currentAction
                  ? (selectedAttachments.delete(currentItemId),
                    updateBatchActions(),
                    loadAttachments())
                  : fetchAdminData())
        : showToast('删除失败：' + (a.message || '未知错误'), 'error');
    } catch (e) {
      (console.error('删除失败:', e), alert('删除失败，请稍后重试'));
    }
  }
}),
  document.querySelectorAll('.modal').forEach(e => {
    e.addEventListener('click', function (e) {
      if (e.target === this) {
        closeModal(this.id);
      }
    });
  }),
  document.addEventListener('keydown', function (e) {
    'Escape' === e.key &&
      document.querySelectorAll('.modal.active').forEach(e => {
        closeModal(e.id);
      });
  }));
