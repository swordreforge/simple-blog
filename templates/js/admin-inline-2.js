let originalAppearanceSettings = {},
    currentAction = null,
    currentItemId = null;
async function loadSettings() {
    try {
        var e, t = localStorage.getItem("auth_token"),
            n = {
                "Content-Type": "application/json"
            },
            o = (t && (n.Authorization = "Bearer " + t), await fetch("/api/settings/appearance", {
                headers: n
            }));
        o.ok ? (e = await o.json(), originalAppearanceSettings = {
            ...e
        }, document.getElementById("backgroundImage").value = e.background_image, document.getElementById("mobileBackgroundImage").value = e.mobile_background_image, document.getElementById("globalOpacity").value = e.global_opacity || "0.15", document.getElementById("opacityValue").textContent = e.global_opacity || "0.15", document.getElementById("backgroundSize").value = e.background_size || "cover", document.getElementById("backgroundPosition").value = e.background_position || "center", document.getElementById("backgroundRepeat").value = e.background_repeat || "no-repeat", document.getElementById("backgroundAttachment").value = e.background_attachment || "fixed", document.getElementById("blurAmount").value = e.blur_amount || "20px", document.getElementById("saturateAmount").value = e.saturate_amount || "180%", document.getElementById("darkModeEnabled").checked = e.dark_mode_enabled || !1, document.getElementById("navbarGlassColor").value = e.navbar_glass_color || "rgba(255, 255, 255, 0.85)", document.getElementById("navbarTextColor").value = e.navbar_text_color || "#333333", document.getElementById("cardGlassColor").value = e.card_glass_color || "rgba(255, 255, 255, 0.75)", document.getElementById("footerGlassColor").value = e.footer_glass_color || "rgba(255, 255, 255, 0.9)", document.getElementById("floatingTextEnabled").checked = e.floating_text_enabled || !1, document.getElementById("floatingTexts").value = e.floating_texts && Array.isArray(e.floating_texts) ? e.floating_texts.join(", ") : "perfect, good, excellent, extraordinary, legend", updateColorPickers(), applyDarkMode(e.dark_mode_enabled || !1), applyGlassColors(e.navbar_glass_color, e.card_glass_color, e.footer_glass_color), updatePreview()) : console.error("加载设置失败:", o.statusText)
    } catch (e) {
        console.error("加载设置失败:", e)
    }
}
async function saveSettings() {
    try {
        var e, t, n = localStorage.getItem("auth_token"),
            o = {
                "Content-Type": "application/json"
            },
            a = (n && (o.Authorization = "Bearer " + n), {
                background_image: document.getElementById("backgroundImage").value,
                mobile_background_image: document.getElementById("mobileBackgroundImage").value,
                global_opacity: document.getElementById("globalOpacity").value,
                background_size: document.getElementById("backgroundSize").value,
                background_position: document.getElementById("backgroundPosition").value,
                background_repeat: document.getElementById("backgroundRepeat").value,
                background_attachment: document.getElementById("backgroundAttachment").value,
                blur_amount: document.getElementById("blurAmount").value,
                saturate_amount: document.getElementById("saturateAmount").value,
                dark_mode_enabled: document.getElementById("darkModeEnabled").checked,
                navbar_glass_color: document.getElementById("navbarGlassColor").value,
                navbar_text_color: document.getElementById("navbarTextColor").value,
                card_glass_color: document.getElementById("cardGlassColor").value,
                footer_glass_color: document.getElementById("footerGlassColor").value,
                floating_text_enabled: document.getElementById("floatingTextEnabled").checked,
                floating_texts: document.getElementById("floatingTexts").value.split(",").map(e => e.trim()).filter(e => 0 < e.length)
            }),
            i = {};
        for (const l in a) {
            var d = originalAppearanceSettings[l],
                r = a[l];
            r !== d && (i[l] = r)
        }
        0 === Object.keys(i).length ? showToast("没有检测到任何变化", "warning") : (e = await fetch("/api/settings/appearance", {
            method: "PATCH",
            headers: o,
            body: JSON.stringify(i)
        })).ok ? (showToast("设置保存成功！", "success"), Object.assign(originalAppearanceSettings, i), "dark_mode_enabled" in i && applyDarkMode(i.dark_mode_enabled), updatePreview()) : (t = await e.json(), showToast("保存失败：" + (t.error || "未知错误"), "error"))
    } catch (e) {
        console.error("保存设置失败:", e), showToast("保存失败，请稍后重试", "error")
    }
}

function resetSettings() {
    confirm("确定要重置为默认设置吗？") && (document.getElementById("backgroundImage").value = "/img/test.webp", document.getElementById("globalOpacity").value = "0.15", document.getElementById("opacityValue").textContent = "0.15", document.getElementById("backgroundSize").value = "cover", document.getElementById("backgroundPosition").value = "center", document.getElementById("backgroundRepeat").value = "no-repeat", document.getElementById("backgroundAttachment").value = "fixed", document.getElementById("blurAmount").value = "20px", document.getElementById("saturateAmount").value = "180%", document.getElementById("darkModeEnabled").checked = !1, document.getElementById("navbarGlassColor").value = "rgba(255, 255, 255, 0.85)", document.getElementById("navbarTextColor").value = "#333333", document.getElementById("cardGlassColor").value = "rgba(255, 255, 255, 0.75)", document.getElementById("footerGlassColor").value = "rgba(255, 255, 255, 0.9)", document.getElementById("floatingTextEnabled").checked = !1, document.getElementById("floatingTexts").value = "perfect, good, excellent, extraordinary, legend", updateColorPickers(), applyDarkMode(!1), applyGlassColors("rgba(255, 255, 255, 0.85)", "rgba(255, 255, 255, 0.75)", "rgba(255, 255, 255, 0.9)"), updatePreview())
}

function updatePreview() {
    var e, t, n, o, a, i, d, r, l, s = document.getElementById("previewBox");
    s && (l = document.getElementById("backgroundImage").value, r = document.getElementById("mobileBackgroundImage").value, e = document.getElementById("globalOpacity").value, t = document.getElementById("backgroundSize").value, n = document.getElementById("backgroundPosition").value, o = document.getElementById("backgroundRepeat").value, a = document.getElementById("backgroundAttachment").value, i = document.getElementById("blurAmount").value, d = document.getElementById("saturateAmount").value, r = window.innerWidth <= 768 && r ? r : l, s.style.backgroundImage = `url('${r}')`, s.style.backgroundSize = t, s.style.backgroundPosition = n, s.style.backgroundRepeat = o, s.style.backgroundAttachment = a, s.style.setProperty("--blur-amount", i), s.style.setProperty("--saturate-amount", d), s.style.setProperty("--global-opacity", e), (l = document.createElement("style")).textContent = `
    #previewBox::before {
      background-image: url('${r}') !important;
      background-size: ${t} !important;
      background-position: ${n} !important;
      background-repeat: ${o} !important;
      background-attachment: ${a} !important;
      filter: blur(${i}) saturate(${d}) !important;
    }
    #previewBox {
      background: rgba(255, 255, 255, ${e}) !important;
      backdrop-filter: blur(${i}) saturate(${d}) !important;
      -webkit-backdrop-filter: blur(${i}) saturate(${d}) !important;
    }
  `, (s = document.getElementById("preview-style")) && s.remove(), l.id = "preview-style", document.head.appendChild(l))
}

function applyDarkMode(e) {
    e ? document.documentElement.classList.add("dark-mode") : document.documentElement.classList.remove("dark-mode")
}

function updateColorPickers() {
    const e = document.getElementById("navbarGlassColor").value,
        t = document.getElementById("navbarTextColor").value,
        n = document.getElementById("cardGlassColor").value,
        o = document.getElementById("footerGlassColor").value;
    if (e.startsWith("rgba")) {
        const t = parseRgba(e);
        t && (document.getElementById("navbarGlassColorPicker").value = rgbaToHex(t.r, t.g, t.b))
    }
    if (t.startsWith("#")) document.getElementById("navbarTextColorPicker").value = t;
    else if (t.startsWith("rgb")) {
        const e = parseRgba(t);
        e && (document.getElementById("navbarTextColorPicker").value = rgbaToHex(e.r, e.g, e.b))
    }
    if (n.startsWith("rgba")) {
        const e = parseRgba(n);
        e && (document.getElementById("cardGlassColorPicker").value = rgbaToHex(e.r, e.g, e.b))
    }
    if (o.startsWith("rgba")) {
        const e = parseRgba(o);
        e && (document.getElementById("footerGlassColorPicker").value = rgbaToHex(e.r, e.g, e.b))
    }
}

function applyGlassColors(e, t, n) {
    document.documentElement.style.setProperty("--navbar-glass-color", e), document.documentElement.style.setProperty("--card-glass-color", t), document.documentElement.style.setProperty("--footer-glass-color", n)
}

function parseRgba(e) {
    return (e = e.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/)) ? {
        r: parseInt(e[1]),
        g: parseInt(e[2]),
        b: parseInt(e[3]),
        a: e[4] ? parseFloat(e[4]) : 1
    } : null
}

function rgbaToHex(e, t, n) {
    var o = e => 1 === (e = e.toString(16)).length ? "0" + e : e;
    return "#" + o(e) + o(t) + o(n)
}
document.addEventListener("DOMContentLoaded", async function() {
    await loadSettings(), await loadTemplateSettings(), await loadMusicSettings(), await loadMusicPlaylist();
    const e = document.getElementById("globalOpacity"),
        t = document.getElementById("opacityValue");
    e && t && e.addEventListener("input", function() {
        t.textContent = this.value, updatePreview()
    });
    var n = document.getElementById("saveSettingsBtn");
    n && n.addEventListener("click", saveSettings), (n = document.getElementById("saveTemplateSettingsBtn")) && n.addEventListener("click", saveTemplateSettings), (n = document.getElementById("saveMusicSettingsBtn")) && n.addEventListener("click", saveMusicSettings), initMusicDragDrop();
    const o = document.getElementById("musicPlayerColorPicker"),
        a = document.getElementById("musicPlayerColor"),
        i = (o && a && o.addEventListener("input", function() {
            var e = parseRgba(a.value);
            e && (e = `rgba(${this.value.match(/\w\w/g).map(e=>parseInt(e,16)).join(", ")}, ${e.a})`, a.value = e)
        }), document.getElementById("live2dEnabled")),
        d = document.getElementById("live2dConfig");
    i && d && i.addEventListener("change", function() {
        d.style.display = this.checked ? "block" : "none"
    }), (n = document.getElementById("resetSettingsBtn")) && n.addEventListener("click", resetSettings), [{
        picker: "navbarGlassColorPicker",
        input: "navbarGlassColor"
    }, {
        picker: "navbarTextColorPicker",
        input: "navbarTextColor"
    }, {
        picker: "cardGlassColorPicker",
        input: "cardGlassColor"
    }, {
        picker: "footerGlassColorPicker",
        input: "footerGlassColor"
    }].forEach(({
        picker: e,
        input: t
    }) => {
        const n = document.getElementById(e),
            o = document.getElementById(t);
        n && o && (n.addEventListener("input", function() {
            const e = o.value;
            if (e.startsWith("rgba")) {
                var t = parseRgba(e);
                if (t) {
                    const e = `rgba(${this.value.match(/\w\w/g).map(e=>parseInt(e,16)).join(", ")}, ${t.a})`;
                    o.value = e
                }
            } else o.value = this.value
        }), o.addEventListener("input", function() {
            updateColorPickers()
        }))
    }), ["backgroundImage", "mobileBackgroundImage", "backgroundSize", "backgroundPosition", "backgroundRepeat", "backgroundAttachment", "blurAmount", "saturateAmount"].forEach(e => {
        (e = document.getElementById(e)) && (e.addEventListener("input", updatePreview), e.addEventListener("change", updatePreview))
    }), window.addEventListener("resize", updatePreview), (n = document.querySelector('[data-tab="settings"]')) && n.addEventListener("click", function() {
        loadSettings(), loadTemplateSettings()
    })
});
let originalTemplateSettings = {};
async function loadTemplateSettings() {
    try {
        var e, t = localStorage.getItem("auth_token"),
            n = {
                "Content-Type": "application/json"
            },
            o = (t && (n.Authorization = "Bearer " + t), await fetch("/api/settings/template", {
                method: "GET",
                headers: n
            }));
        o.ok ? (e = await o.json(), originalTemplateSettings = {
            ...e
        }, document.getElementById("templateName").value = e.name || "", document.getElementById("templateGreting").value = e.greting || "", document.getElementById("templateYear").value = e.year || "", document.getElementById("templateFoodes").value = e.foodes || "", document.getElementById("globalAvatar").value = e.global_avatar || "/img/avatar.webp", document.getElementById("templateArticleTitle").checked = e.article_title || !1, document.getElementById("templateArticleTitlePrefix").value = e.article_title_prefix || "", document.getElementById("templateSwitchNotice").checked = e.switch_notice || !1, document.getElementById("templateSwitchNoticeText").value = e.switch_notice_text || "", document.getElementById("externalLinkWarning").checked = e.external_link_warning || !1, document.getElementById("externalLinkWhitelist").value = e.external_link_whitelist || "", document.getElementById("externalLinkWarningText").value = e.external_link_warning_text || "", document.getElementById("passageSummarizeEnabled").checked = e.passage_summarize_enabled || !1, document.getElementById("live2dEnabled").checked = e.live2d_enabled || !1, document.getElementById("live2dShowOnIndex").checked = !1 !== e.live2d_show_on_index, document.getElementById("live2dShowOnPassage").checked = !1 !== e.live2d_show_on_passage, document.getElementById("live2dShowOnCollect").checked = !1 !== e.live2d_show_on_collect, document.getElementById("live2dShowOnAbout").checked = !1 !== e.live2d_show_on_about, document.getElementById("live2dShowOnAdmin").checked = e.live2d_show_on_admin || !1, document.getElementById("live2dModelId").value = e.live2d_model_id || "1", document.getElementById("live2dModelPath").value = e.live2d_model_path || "", document.getElementById("live2dCDNPath").value = e.live2d_cdn_path || "https://unpkg.com/live2d-widget-model@1.0.5/", document.getElementById("live2dPosition").value = e.live2d_position || "right", document.getElementById("live2dWidth").value = e.live2d_width || "280px", document.getElementById("live2dHeight").value = e.live2d_height || "250px", document.getElementById("live2dConfig").style.display = e.live2d_enabled ? "block" : "none", document.getElementById("sponsorEnabled").checked = e.sponsor_enabled || !1, document.getElementById("sponsorTitle").value = e.sponsor_title || "感谢您的支持", document.getElementById("sponsorImage").value = e.sponsor_image || "/img/avatar.webp", document.getElementById("sponsorDescription").value = e.sponsor_description || "如果您觉得这个博客对您有帮助，欢迎赞助支持！", document.getElementById("sponsorButtonText").value = e.sponsor_button_text || "❤️ 赞助支持", document.getElementById("beianEnabled").checked = e.beian_enabled || !1, document.getElementById("icpNumber").value = e.icp_number || "", document.getElementById("policeRecordCode").value = e.police_record_code || "", document.getElementById("policeRecordContent").value = e.police_record_content || "") : console.error("加载模板设置失败")
    } catch (e) {
        console.error("加载模板设置失败:", e)
    }
}
async function saveTemplateSettings() {
    try {
        var e, t, n = localStorage.getItem("auth_token"),
            o = {
                "Content-Type": "application/json"
            },
            a = (n && (o.Authorization = "Bearer " + n), {
                name: document.getElementById("templateName").value,
                greting: document.getElementById("templateGreting").value,
                year: document.getElementById("templateYear").value,
                foodes: document.getElementById("templateFoodes").value,
                global_avatar: document.getElementById("globalAvatar").value,
                article_title: document.getElementById("templateArticleTitle").checked,
                article_title_prefix: document.getElementById("templateArticleTitlePrefix").value,
                switch_notice: document.getElementById("templateSwitchNotice").checked,
                switch_notice_text: document.getElementById("templateSwitchNoticeText").value,
                external_link_warning: document.getElementById("externalLinkWarning").checked,
                external_link_whitelist: document.getElementById("externalLinkWhitelist").value,
                external_link_warning_text: document.getElementById("externalLinkWarningText").value,
                passage_summarize_enabled: document.getElementById("passageSummarizeEnabled").checked,
                live2d_enabled: document.getElementById("live2dEnabled").checked,
                live2d_show_on_index: document.getElementById("live2dShowOnIndex").checked,
                live2d_show_on_passage: document.getElementById("live2dShowOnPassage").checked,
                live2d_show_on_collect: document.getElementById("live2dShowOnCollect").checked,
                live2d_show_on_about: document.getElementById("live2dShowOnAbout").checked,
                live2d_show_on_admin: document.getElementById("live2dShowOnAdmin").checked,
                live2d_model_id: document.getElementById("live2dModelId").value,
                live2d_model_path: document.getElementById("live2dModelPath").value,
                live2d_cdn_path: document.getElementById("live2dCDNPath").value,
                live2d_position: document.getElementById("live2dPosition").value,
                live2d_width: document.getElementById("live2dWidth").value,
                live2d_height: document.getElementById("live2dHeight").value,
                sponsor_enabled: document.getElementById("sponsorEnabled").checked,
                sponsor_title: document.getElementById("sponsorTitle").value,
                sponsor_image: document.getElementById("sponsorImage").value,
                sponsor_description: document.getElementById("sponsorDescription").value,
                sponsor_button_text: document.getElementById("sponsorButtonText").value,
                beian_enabled: document.getElementById("beianEnabled").checked,
                icp_number: document.getElementById("icpNumber").value,
                police_record_code: document.getElementById("policeRecordCode").value,
                police_record_content: document.getElementById("policeRecordContent").value
            }),
            i = {};
        console.log("原始设置:", originalTemplateSettings);
        console.log("当前设置:", a);
        console.log("passage_summarize_enabled - 原始:", originalTemplateSettings.passage_summarize_enabled, "当前:", a.passage_summarize_enabled);
        for (const d in a) a[d] !== originalTemplateSettings[d] && (i[d] = a[d]);
        0 === Object.keys(i).length ? showToast("没有检测到任何变化", "warning") : (e = await fetch("/api/settings/template", {
            method: "PATCH",
            headers: o,
            body: JSON.stringify(i)
        })).ok ? (showToast("模板设置保存成功！", "success"), Object.assign(originalTemplateSettings, i)) : (t = await e.json(), showToast("保存失败：" + (t.error || "未知错误"), "error"))
    } catch (e) {
        console.error("保存模板设置失败:", e), showToast("保存失败，请稍后重试", "error")
    }
}
let originalMusicSettings = {};
async function loadMusicSettings() {
    try {
        var e, t, n = localStorage.getItem("auth_token"),
            o = {
                "Content-Type": "application/json"
            },
            a = (n && (o.Authorization = "Bearer " + n), await fetch("/api/settings/music", {
                method: "GET",
                headers: o
            }));
        a.ok ? (e = await a.json(), originalMusicSettings = {
            ...e
        }, document.getElementById("musicEnabled").checked = e.enabled || !1, document.getElementById("musicAutoPlay").checked = e.auto_play || !1, document.getElementById("musicControlSize").value = e.control_size || "medium", document.getElementById("musicPlayerColor").value = e.player_color || "rgba(66, 133, 244, 0.9)", document.getElementById("musicPosition").value = e.position || "bottom-right", document.getElementById("musicCustomCSS").value = e.custom_css || "", (t = parseRgba(e.player_color || "rgba(66, 133, 244, 0.9)")) && (document.getElementById("musicPlayerColorPicker").value = rgbaToHex(t.r, t.g, t.b))) : console.error("加载音乐设置失败")
    } catch (e) {
        console.error("加载音乐设置失败:", e)
    }
}
async function saveMusicSettings() {
    try {
        var e, t, n = localStorage.getItem("auth_token"),
            o = {
                "Content-Type": "application/json"
            },
            a = (n && (o.Authorization = "Bearer " + n), {
                enabled: document.getElementById("musicEnabled").checked,
                auto_play: document.getElementById("musicAutoPlay").checked,
                control_size: document.getElementById("musicControlSize").value,
                player_color: document.getElementById("musicPlayerColor").value,
                position: document.getElementById("musicPosition").value,
                custom_css: document.getElementById("musicCustomCSS").value
            }),
            i = {};
        for (const d in a) a[d] !== originalMusicSettings[d] && (i[d] = a[d]);
        0 === Object.keys(i).length ? showToast("没有检测到任何变化", "warning") : (e = await fetch("/api/settings/music", {
            method: "PATCH",
            headers: o,
            body: JSON.stringify(i)
        })).ok ? (showToast("音乐设置保存成功！", "success"), Object.assign(originalMusicSettings, i)) : (t = await e.json(), showToast("保存失败：" + (t.error || "未知错误"), "error"))
    } catch (e) {
        console.error("保存音乐设置失败:", e), showToast("保存失败，请稍后重试", "error")
    }
}
let musicUploadQueue = [],
    isUploading = !1,
    uploadAbortController = null;

function initMusicDragDrop() {
    const t = document.getElementById("musicDropZone"),
        n = document.getElementById("musicFileUpload"),
        e = document.querySelector(".browse-link");
    var o;

    function a(e) {
        e.preventDefault(), e.stopPropagation()
    }
    t && n && (t.addEventListener("click", () => {
        n.click()
    }), e && e.addEventListener("click", e => {
        e.stopPropagation(), n.click()
    }), n.addEventListener("change", e => {
        handleFileSelect(e.target.files), n.value = ""
    }), ["dragenter", "dragover", "dragleave", "drop"].forEach(e => {
        t.addEventListener(e, a, !1)
    }), ["dragenter", "dragover"].forEach(e => {
        t.addEventListener(e, () => {
            t.classList.add("dragover")
        }, !1)
    }), t.addEventListener("dragleave", e => {
        t.contains(e.relatedTarget) || t.classList.remove("dragover")
    }, !1), t.addEventListener("drop", e => {
        t.classList.remove("dragover"), handleFileSelect(e.dataTransfer.files)
    }, !1), (o = document.getElementById("clearUploadBtn")) && o.addEventListener("click", () => {
        isUploading ? showToast("正在上传中，请先取消上传", "warning") : (musicUploadQueue = [], updateUploadListUI())
    }), (o = document.getElementById("uploadAllMusicBtn")) && o.addEventListener("click", startBatchUpload), o = document.getElementById("cancelUploadBtn")) && o.addEventListener("click", cancelBatchUpload)
}

function handleFileSelect(e) {
    e && 0 !== e.length && (0 !== (e = Array.from(e).filter(e => {
        var t = "." + e.name.split(".").pop().toLowerCase();
        return (t = ["audio/mpeg", "audio/mp3", "audio/wav", "audio/wave", "audio/ogg", "audio/x-m4a", "audio/mp4"].includes(e.type) || [".mp3", ".wav", ".ogg", ".m4a"].includes(t)) || showToast("跳过不支持的文件: " + e.name, "warning"), t
    })).length ? (e.forEach(t => {
        musicUploadQueue.some(e => e.file.name === t.name && e.file.size === t.size) || musicUploadQueue.push({
            file: t,
            id: Date.now() + Math.random(),
            status: "pending",
            progress: 0,
            error: null
        })
    }), updateUploadListUI()) : showToast("没有有效的音频文件", "warning"))
}

function updateUploadListUI() {
    var e = document.getElementById("musicUploadList"),
        t = document.getElementById("musicUploadItems");
    e && t && (0 !== musicUploadQueue.length ? (e.style.display = "block", t.innerHTML = musicUploadQueue.map((e, t) => `
    <div class="upload-item" data-id="${e.id}">
      <div class="upload-item-cover">
        <div class="cover-preview ${e.coverFile?"has-cover":""}" onclick="triggerCoverSelect('${e.id}')">
          ${e.coverFile?`<img src="${e.coverPreview}" alt="封面">
             <div class="cover-remove-btn" onclick="event.stopPropagation(); removeCover('${e.id}')">×</div>`:'<span class="cover-placeholder"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>️</span>'}
          <div class="cover-upload-hint">点击上传封面</div>
        </div>
        <input type="file" class="cover-input" id="coverInput-${e.id}" accept="image/jpeg,image/jpg,image/png,image/gif,image/webp" onchange="handleCoverSelect('${e.id}', this)">
      </div>
      <div class="upload-item-icon" style="display: none;"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg></div>
      <div class="upload-item-info">
        <div class="upload-item-name">${e.file.name}</div>
        <div class="upload-item-meta">
          <span>${formatFileSize(e.file.size)}</span>
          <span>${e.file.type||"audio/*"}</span>
        </div>
      </div>
      <div class="upload-item-progress">
        <div class="progress-bar">
          <div class="progress-fill" style="width: ${e.progress}%"></div>
        </div>
        <div class="progress-text">${e.progress}%</div>
      </div>
      <div class="upload-item-status ${e.status}">
        ${getStatusText(e.status)}
      </div>
      <div class="upload-item-action">
        <button class="remove-upload-btn" onclick="removeUploadItem('${e.id}')" ${"uploading"===e.status?"disabled":""}>×</button>
      </div>
    </div>
  `).join("")) : e.style.display = "none")
}

function formatFileSize(e) {
    var t;
    return 0 === e ? "0 B" : (t = Math.floor(Math.log(e) / Math.log(1024)), Math.round(e / Math.pow(1024, t) * 100) / 100 + " " + ["B", "KB", "MB", "GB"][t])
}

function getStatusText(e) {
    return {
        pending: "等待上传",
        uploading: "上传中",
        success: "上传成功",
        error: "上传失败"
    } [e] || e
}

function removeUploadItem(t) {
    isUploading ? showToast("正在上传中，请先取消上传", "warning") : (musicUploadQueue = musicUploadQueue.filter(e => e.id != t), updateUploadListUI())
}

function triggerCoverSelect(e) {
    (e = document.getElementById("coverInput-" + e)) && e.click()
}

function handleCoverSelect(n, e) {
    const o = e.files[0];
    var t;
    o && (["image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp"].includes(o.type) ? 5242880 < o.size ? (showToast("封面图片大小不能超过 5MB", "error"), e.value = "") : ((t = new FileReader).onload = function(e) {
        var t = musicUploadQueue.find(e => e.id == n);
        t && (t.coverFile = o, t.coverPreview = e.target.result, updateUploadListUI())
    }, t.readAsDataURL(o)) : (showToast("请选择有效的图片文件（JPG、PNG、GIF、WebP）", "error"), e.value = ""))
}

function removeCover(t) {
    var e = musicUploadQueue.find(e => e.id == t);
    e && (e.coverFile = null, e.coverPreview = null, (e = document.getElementById("coverInput-" + t)) && (e.value = ""), updateUploadListUI())
}
async function startBatchUpload() {
    if (isUploading) showToast("正在上传中...", "info");
    else {
        var n = musicUploadQueue.filter(e => "pending" === e.status);
        if (0 === n.length) showToast("没有等待上传的文件", "warning");
        else {
            isUploading = !0, uploadAbortController = new AbortController;
            var o = document.getElementById("uploadAllMusicBtn"),
                a = document.getElementById("cancelUploadBtn");
            o && (o.disabled = !0), a && (a.disabled = !1);
            let e = 0,
                t = 0;
            for (const i of n) {
                if (!isUploading) break;
                i.status = "uploading", updateUploadListUI();
                try {
                    await uploadSingleMusicFile(i, uploadAbortController.signal), i.status = "success", i.progress = 100, e++
                } catch (n) {
                    i.status = "error", i.error = n.message, t++, console.error(`上传文件 ${i.file.name} 失败:`, n)
                }
                updateUploadListUI()
            }
            isUploading = !1, uploadAbortController = null, o && (o.disabled = !1), a && (a.disabled = !0), 0 < e && (showToast(`成功上传 ${e} 个文件`, "success"), loadMusicPlaylist()), 0 < t && showToast(t + " 个文件上传失败", "error"), setTimeout(() => {
                musicUploadQueue = musicUploadQueue.filter(e => "success" !== e.status), updateUploadListUI()
            }, 2e3)
        }
    }
}

function uploadSingleMusicFile(i, d) {
    return new Promise((t, n) => {
        var e = new FormData,
            o = (e.append("file", i.file), i.file.name.replace(/\.[^/.]+$/, "")),
            o = (e.append("title", o), e.append("artist", "未知艺术家"), i.coverFile && e.append("cover", i.coverFile), localStorage.getItem("auth_token"));
        const a = new XMLHttpRequest;
        a.upload.addEventListener("progress", e => {
            e.lengthComputable && (e = Math.round(e.loaded / e.total * 100), i.progress = e, updateUploadListUI())
        }), a.addEventListener("load", () => {
            if (200 <= a.status && a.status < 300) t();
            else try {
                var e = JSON.parse(a.responseText);
                n(new Error(e.error || "上传失败"))
            } catch (e) {
                n(new Error("上传失败: " + a.status))
            }
        }), a.addEventListener("error", () => {
            n(new Error("网络错误"))
        }), a.addEventListener("abort", () => {
            n(new Error("上传已取消"))
        }), d.addEventListener("abort", () => {
            a.abort()
        }), a.open("POST", "/api/music/upload"), o && a.setRequestHeader("Authorization", "Bearer " + o), a.send(e)
    })
}

function cancelBatchUpload() {
    var e, t;
    isUploading && (uploadAbortController && uploadAbortController.abort(), isUploading = !1, uploadAbortController = null, musicUploadQueue.forEach(e => {
        "uploading" === e.status && (e.status = "pending", e.progress = 0)
    }), updateUploadListUI(), showToast("已取消上传", "info"), e = document.getElementById("uploadAllMusicBtn"), t = document.getElementById("cancelUploadBtn"), e && (e.disabled = !1), t) && (t.disabled = !0)
}
async function uploadMusicFile() {
    var e = document.getElementById("musicFileUpload");
    e.files && 0 !== e.files.length ? (handleFileSelect(e.files), e.value = "") : showToast("请选择要上传的音乐文件", "warning")
}
async function loadMusicPlaylist() {
    try {
        var e = localStorage.getItem("auth_token"),
            t = {
                "Content-Type": "application/json"
            },
            n = (e && (t.Authorization = "Bearer " + e), await fetch("/api/music/playlist", {
                method: "GET",
                headers: t
            }));
        n.ok ? updateMusicPlaylistUI(await n.json()) : console.error("加载播放列表失败")
    } catch (e) {
        console.error("加载播放列表失败:", e)
    }
}

function updateMusicPlaylistUI(e) {
    var t = document.getElementById("musicPlaylistContainer");
    t && (e && 0 !== e.length ? t.innerHTML = e.map((e, t) => {
        let n = e.title,
            o = n.match(/^\d+_/);
        return o && (n = n.substring(o[0].length)), `
    <div style="display: flex; align-items: center; gap: 10px; padding: 10px; border-bottom: 1px solid rgba(0, 0, 0, 0.05);">
      <div style="width: 50px; height: 50px; border-radius: 8px; overflow: hidden; background: rgba(0, 0, 0, 0.05); display: flex; align-items: center; justify-content: center;">
        ${e.cover_image?`<img src="${e.cover_image}" alt="${e.title}" style="width: 100%; height: 100%; object-fit: cover;">`:'<span style="font-size: 24px;"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg></span>'}
      </div>
      <div style="flex: 1;">
        <div style="font-weight: 500;">${n}</div>
        <div style="font-size: 0.85em; color: #666;">${e.artist}</div>
      </div>
      <div style="font-size: 0.85em; color: #999;">${e.duration}</div>
      <div style="display: flex; gap: 5px;">
        <button onclick="editMusicTitle(${e.id}, '${n.replace(/'/g,"\\'")}')" style="background: #6c757d; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;">编辑标题</button>
        <button onclick="changeMusicCover(${e.id})" style="background: #007bff; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;">更换封面</button>
        <button onclick="deleteMusicTrack(${e.id})" style="background: #e74c3c; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;">删除</button>
      </div>
    </div>
    `
    }).join("") : t.innerHTML = '<div style="text-align: center; color: #999; padding: 20px;">暂无音乐文件</div>')
}
window.deleteMusicTrack = async function(e) {
    if (confirm("确定要删除这首音乐吗？")) try {
        var t = localStorage.getItem("auth_token"),
            n = {
                "Content-Type": "application/json"
            },
            o = (t && (n.Authorization = "Bearer " + t), await fetch("/api/music/" + e, {
                method: "DELETE",
                headers: n
            }));
        if (o.ok) showToast("删除成功！", "success"), loadMusicPlaylist();
        else {
            const e = await o.json();
            showToast("删除失败：" + (e.error || "未知错误"), "error")
        }
    } catch (e) {
        console.error("删除音乐失败:", e), showToast("删除失败，请稍后重试", "error")
    }
}, window.changeMusicCover = async function(a) {
    var e = document.createElement("input");
    e.type = "file", e.accept = "image/jpeg,image/jpg,image/png,image/gif,image/webp", e.onchange = async e => {
        const t = e.target.files[0];
        if (t)
            if (["image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp"].includes(t.type))
                if (5242880 < t.size) showToast("图片大小不能超过 5MB", "error");
                else {
                    var n = new FormData;
                    n.append("cover", t);
                    try {
                        const e = localStorage.getItem("auth_token"),
                            t = {};
                        e && (t.Authorization = "Bearer " + e);
                        var o = await fetch(`/api/music/${a}/cover`, {
                            method: "POST",
                            body: n,
                            headers: t
                        });
                        if (o.ok) await o.json(), showToast("封面更新成功！", "success"), loadMusicPlaylist();
                        else {
                            const a = await o.json();
                            showToast("封面更新失败：" + (a.message || "未知错误"), "error")
                        }
                    } catch (e) {
                        console.error("更新封面失败:", e), showToast("更新封面失败，请稍后重试", "error")
                    }
                }
        else showToast("请选择有效的图片文件（JPEG, PNG, GIF, WebP）", "error")
    }, e.click()
}, window.editMusicTitle = function(o, e) {
    const a = document.createElement("div"),
        i = (a.style.cssText = "\n    position: fixed;\n    top: 0;\n    left: 0;\n    width: 100%;\n    height: 100%;\n    background: rgba(0, 0, 0, 0.5);\n    display: flex;\n    justify-content: center;\n    align-items: center;\n    z-index: 10000;\n  ", a.innerHTML = `
    <div style="background: white; padding: 30px; border-radius: 10px; width: 400px; max-width: 90%;">
      <h3 style="margin: 0 0 20px 0;">编辑标题</h3>
      <input type="text" id="musicTitleInput" value="${e}" 
             style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 5px; margin-bottom: 20px; box-sizing: border-box;">
      <div style="display: flex; gap: 10px; justify-content: flex-end;">
        <button id="cancelBtn" style="padding: 8px 20px; background: #6c757d; color: white; border: none; border-radius: 5px; cursor: pointer;">取消</button>
        <button id="saveBtn" style="padding: 8px 20px; background: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer;">保存</button>
      </div>
    </div>
  `, document.body.appendChild(a), a.querySelector("#musicTitleInput")),
        t = a.querySelector("#cancelBtn"),
        n = a.querySelector("#saveBtn"),
        d = (t.onclick = () => {
            document.body.removeChild(a)
        }, n.onclick = async () => {
            var e = i.value.trim();
            if (e) try {
                const i = localStorage.getItem("auth_token"),
                    n = {
                        "Content-Type": "application/json"
                    };
                i && (n.Authorization = "Bearer " + i);
                var t = await fetch(`/api/music/${o}?action=title`, {
                    method: "PUT",
                    body: JSON.stringify({
                        title: e
                    }),
                    headers: n
                });
                if (t.ok) await t.json(), showToast("标题更新成功！", "success"), document.body.removeChild(a), loadMusicPlaylist();
                else {
                    const o = await t.json();
                    showToast("标题更新失败：" + (o.message || "未知错误"), "error")
                }
            } catch (e) {
                console.error("更新标题失败:", e), showToast("更新标题失败，请稍后重试", "error")
            } else showToast("标题不能为空", "error")
        }, a.onclick = e => {
            e.target === a && document.body.removeChild(a)
        }, e => {
            "Escape" === e.key && (document.body.removeChild(a), document.removeEventListener("keydown", d))
        });
    document.addEventListener("keydown", d)
};
let mainCards = [],
    subCards = [];
async function loadMainCards() {
    try {
        var e = localStorage.getItem("auth_token"),
            t = {
                "Content-Type": "application/json"
            },
            n = (e && (t.Authorization = "Bearer " + e), await fetch("/api/about/main-cards/admin", {
                method: "GET",
                headers: t
            }));
        n.ok ? (mainCards = await n.json(), updateMainCardsTable(), updateMainCardFilter()) : showToast("加载主卡片失败", "error")
    } catch (e) {
        console.error("加载主卡片失败:", e), showToast("加载主卡片失败", "error")
    }
}

function updateMainCardsTable() {
    var e = document.getElementById("mainCardsTableBody");
    e && (0 !== mainCards.length ? e.innerHTML = mainCards.map(e => `
    <tr>
      <td>${e.sort_order}</td>
      <td>${e.icon||""}</td>
      <td>${e.title}</td>
      <td>${e.layout_type}</td>
      <td>${getSubCardCount(e.id)}</td>
      <td>
        <span style="color: ${e.is_enabled?"#28a745":"#dc3545"}; font-weight: bold;">
          ${e.is_enabled?"✓ 启用":"✕ 禁用"}
        </span>
      </td>
      <td>
        <button class="btn-secondary" onclick="editMainCard(${e.id})">编辑</button>
        <button class="btn-secondary" onclick="toggleMainCardEnabled(${e.id}, ${e.is_enabled})">
          ${e.is_enabled?"禁用":"启用"}
        </button>
        <button class="btn-danger" onclick="deleteMainCard(${e.id})">删除</button>
      </td>
    </tr>
  `).join("") : e.innerHTML = '<tr><td colspan="7" style="text-align: center; color: #999;">暂无主卡片</td></tr>')
}

function getSubCardCount(t) {
    return subCards.filter(e => e.main_card_id === t).length
}

function updateMainCardFilter() {
    var e, t = document.getElementById("subCardMainCardFilter");
    t && (e = t.value, t.innerHTML = '<option value="">全部主卡片</option>' + mainCards.map(e => `<option value="${e.id}">${e.title}</option>`).join(""), t.value = e)
}
async function loadSubCards() {
    try {
        var e = localStorage.getItem("auth_token"),
            t = {
                "Content-Type": "application/json"
            },
            n = (e && (t.Authorization = "Bearer " + e), await fetch("/api/about/sub-cards/admin", {
                method: "GET",
                headers: t
            }));
        n.ok ? (subCards = await n.json(), updateSubCardsTable(), updateMainCardsTable()) : showToast("加载次卡片失败", "error")
    } catch (e) {
        console.error("加载次卡片失败:", e), showToast("加载次卡片失败", "error")
    }
}

function updateSubCardsTable() {
    var e = document.getElementById("subCardsTableBody");
    if (e) {
        const n = document.getElementById("subCardMainCardFilter").value;
        var t = subCards;
        0 !== (t = n ? subCards.filter(e => e.main_card_id === parseInt(n)) : t).length ? e.innerHTML = t.map(t => (mainCards.find(e => e.id === t.main_card_id), `
      <tr>
        <td>${t.sort_order}</td>
        <td>${t.icon||""}</td>
        <td>${t.title}</td>
        <td>${t.description.substring(0,30)}${30<t.description.length?"...":""}</td>
        <td>${t.link_url||"-"}</td>
        <td>
          <span style="color: ${t.is_enabled?"#28a745":"#dc3545"}; font-weight: bold;">
            ${t.is_enabled?"✓ 启用":"✕ 禁用"}
          </span>
        </td>
        <td>
          <button class="btn-secondary" onclick="editSubCard(${t.id})">编辑</button>
          <button class="btn-secondary" onclick="toggleSubCardEnabled(${t.id}, ${t.is_enabled})">
            ${t.is_enabled?"禁用":"启用"}
          </button>
          <button class="btn-danger" onclick="deleteSubCard(${t.id})">删除</button>
        </td>
      </tr>
    `)).join("") : e.innerHTML = '<tr><td colspan="7" style="text-align: center; color: #999;">暂无次卡片</td></tr>'
    }
}
async function addSubCard() {
    const t = document.getElementById("subCardMainCardFilter").value;
    var e;
    t ? (document.getElementById("subCardModalTitle").textContent = "添加次卡片", document.getElementById("subCardForm").reset(), document.getElementById("subCardId").value = "", (e = document.getElementById("subCardMainCardId")).innerHTML = '<option value="">请选择主卡片</option>' + mainCards.map(e => `<option value="${e.id}">${e.title}</option>`).join(""), e.value = t, document.getElementById("subCardSortOrder").value = subCards.filter(e => e.main_card_id === parseInt(t)).length + 1, document.getElementById("subCardEnabled").checked = !0, openModal("subCardModal")) : showToast("请先选择一个主卡片", "warning")
}
window.addMainCard = async function() {
    document.getElementById("mainCardModalTitle").textContent = "添加主卡片", document.getElementById("mainCardForm").reset(), document.getElementById("mainCardId").value = "", document.getElementById("mainCardSortOrder").value = mainCards.length + 1, document.getElementById("mainCardEnabled").checked = !0, openModal("mainCardModal")
}, window.editMainCard = async function(t) {
    var e = mainCards.find(e => e.id === t);
    e && (document.getElementById("mainCardModalTitle").textContent = "编辑主卡片", document.getElementById("mainCardId").value = e.id, document.getElementById("mainCardTitle").value = e.title || "", document.getElementById("mainCardIcon").value = e.icon || "", document.getElementById("mainCardLayoutType").value = e.layout_type || "grid", document.getElementById("mainCardCustomCss").value = e.custom_css || "", document.getElementById("mainCardSortOrder").value = e.sort_order || 0, document.getElementById("mainCardEnabled").checked = e.is_enabled, openModal("mainCardModal"))
}, window.deleteMainCard = async function(e) {
    currentAction = "delete-main-card", currentItemId = e, document.getElementById("confirmMessage").textContent = "确定要删除这个主卡片吗？所有关联的次卡片也会被删除。此操作不可撤销。", openModal("confirmModal")
}, window.toggleMainCardEnabled = async function(e, t) {
    try {
        var n = localStorage.getItem("auth_token");
        (await fetch("/api/about/main-cards/enabled?id=" + e, {
            method: "PUT",
            headers: {
                "Content-Type": "application/json",
                Authorization: "Bearer " + n
            },
            body: JSON.stringify({
                enabled: !t
            })
        })).ok ? loadMainCards() : showToast("操作失败", "error")
    } catch (e) {
        console.error("切换状态失败:", e), showToast("操作失败", "error")
    }
}, window.editSubCard = async function(t) {
    var e, n = subCards.find(e => e.id === t);
    n && (document.getElementById("subCardModalTitle").textContent = "编辑次卡片", document.getElementById("subCardId").value = n.id, (e = document.getElementById("subCardMainCardId")).innerHTML = '<option value="">请选择主卡片</option>' + mainCards.map(e => `<option value="${e.id}">${e.title}</option>`).join(""), e.value = n.main_card_id, document.getElementById("subCardTitle").value = n.title || "", document.getElementById("subCardDescription").value = n.description || "", document.getElementById("subCardIcon").value = n.icon || "", document.getElementById("subCardLinkUrl").value = n.link_url || "", document.getElementById("subCardCustomCss").value = n.custom_css || "", document.getElementById("subCardSortOrder").value = n.sort_order || 0, document.getElementById("subCardEnabled").checked = n.is_enabled, openModal("subCardModal"))
}, window.deleteSubCard = async function(e) {
    currentAction = "delete-sub-card", currentItemId = e, document.getElementById("confirmMessage").textContent = "确定要删除这个次卡片吗？此操作不可撤销。", openModal("confirmModal")
}, window.toggleSubCardEnabled = async function(e, t) {
    try {
        var n = localStorage.getItem("auth_token");
        (await fetch("/api/about/sub-cards/enabled?id=" + e, {
            method: "PUT",
            headers: {
                "Content-Type": "application/json",
                Authorization: "Bearer " + n
            },
            body: JSON.stringify({
                enabled: !t
            })
        })).ok ? loadSubCards() : showToast("操作失败", "error")
    } catch (e) {
        console.error("切换状态失败:", e), showToast("操作失败", "error")
    }
}, document.addEventListener("DOMContentLoaded", function() {
    var e = document.getElementById("addMainCardBtn");
    e && e.addEventListener("click", addMainCard), (e = document.getElementById("addSubCardBtn")) && e.addEventListener("click", addSubCard), (e = document.getElementById("refreshAboutCardsBtn")) && e.addEventListener("click", function() {
        loadMainCards(), loadSubCards()
    }), (e = document.getElementById("subCardMainCardFilter")) && e.addEventListener("change", updateSubCardsTable), (e = document.querySelector('[data-tab="about"]')) && e.addEventListener("click", function() {
        loadMainCards(), loadSubCards(), loadFriendLinks()
    }), initializeFriendLinksManagement()
});
const mainCardForm = document.getElementById("mainCardForm"),
    subCardForm = (mainCardForm && mainCardForm.addEventListener("submit", async function(n) {
        n.preventDefault();
        var o = document.getElementById("mainCardId").value,
            a = document.getElementById("mainCardTitle").value,
            i = document.getElementById("mainCardIcon").value,
            d = document.getElementById("mainCardLayoutType").value,
            r = document.getElementById("mainCardCustomCss").value,
            l = parseInt(document.getElementById("mainCardSortOrder").value),
            s = document.getElementById("mainCardEnabled").checked;
        try {
            const n = {
                "Content-Type": "application/json",
                Authorization: "Bearer " + localStorage.getItem("auth_token")
            };
            let e = "/api/about/main-cards",
                t = "POST";
            o && (e = "/api/about/main-cards/update?id=" + o, t = "PUT"), (await fetch(e, {
                method: t,
                headers: n,
                body: JSON.stringify({
                    title: a,
                    icon: i,
                    layout_type: d,
                    custom_css: r,
                    sort_order: l,
                    is_enabled: s
                })
            })).ok ? (showToast(o ? "更新成功" : "添加成功", "success"), closeModal("mainCardModal"), loadMainCards()) : showToast("操作失败", "error")
        } catch (n) {
            console.error("操作失败:", n), showToast("操作失败", "error")
        }
    }), document.getElementById("subCardForm"));

function escapeHtml(e) {
    var t;
    return e ? ((t = document.createElement("div")).textContent = e, t.innerHTML) : ""
}
subCardForm && subCardForm.addEventListener("submit", async function(n) {
    n.preventDefault();
    var o = document.getElementById("subCardId").value,
        a = parseInt(document.getElementById("subCardMainCardId").value),
        i = document.getElementById("subCardTitle").value,
        d = document.getElementById("subCardDescription").value,
        r = document.getElementById("subCardIcon").value,
        l = document.getElementById("subCardLinkUrl").value,
        s = document.getElementById("subCardCustomCss").value,
        c = parseInt(document.getElementById("subCardSortOrder").value),
        u = document.getElementById("subCardEnabled").checked;
    try {
        const n = {
            "Content-Type": "application/json",
            Authorization: "Bearer " + localStorage.getItem("auth_token")
        };
        let e = "/api/about/sub-cards",
            t = "POST";
        o && (e = "/api/about/sub-cards/update?id=" + o, t = "PUT"), (await fetch(e, {
            method: t,
            headers: n,
            body: JSON.stringify({
                main_card_id: a,
                title: i,
                description: d,
                icon: r,
                link_url: l,
                custom_css: s,
                sort_order: c,
                is_enabled: u
            })
        })).ok ? (showToast(o ? "更新成功" : "添加成功", "success"), closeModal("subCardModal"), loadSubCards()) : showToast("操作失败", "error")
    } catch (n) {
        console.error("操作失败:", n), showToast("操作失败", "error")
    }
});
let friendLinks = [],
    selectedFriendLinkIds = [];
async function loadFriendLinks() {
    try {
        var e, t = {
                Authorization: "Bearer " + localStorage.getItem("auth_token")
            },
            n = await fetch("/api/admin/friend-links?include_disabled=true", {
                headers: t
            });
        n.ok ? (e = await n.json()).success ? (friendLinks = e.data, updateFriendLinksTable()) : showToast(e.message || "加载友链列表失败", "error") : showToast("加载友链列表失败", "error")
    } catch (e) {
        console.error("加载友链列表失败:", e), showToast("加载友链列表失败", "error")
    }
}

function updateFriendLinksTable() {
    var e = document.getElementById("friendLinksTableBody");
    (0 === friendLinks.length ? (e.innerHTML = '\n      <tr>\n        <td colspan="7" style="text-align: center; padding: 40px;">\n          <div style="opacity: 0.5;">暂无友情链接</div>\n          <div style="margin-top: 10px;">\n            <button class="btn-primary" onclick="openAddFriendLinkModal()">添加第一个友链</button>\n          </div>\n        </td>\n      </tr>\n    ', updateBatchDeleteButton) : (e.innerHTML = friendLinks.map(e => `
    <tr style="${e.is_enabled?"":"opacity: 0.6;"}">
      <td>
        <input type="checkbox" class="friend-link-checkbox" data-id="${e.id}"
               style="width: auto;"
               ${selectedFriendLinkIds.includes(e.id)?"checked":""}>
      </td>
      <td>
        <img src="${e.avatar_url||"/img/avatar.webp"}" alt="${e.nickname}"
             style="width: 40px; height: 40px; border-radius: 50%; object-fit: cover;"
             onerror="this.src='/img/avatar.webp'">
      </td>
      <td><strong>${escapeHtml(e.nickname)}</strong></td>
      <td>
        <a href="${escapeHtml(e.link_url)}" target="_blank" rel="noopener noreferrer"
           style="color: #007bff; max-width: 200px; display: inline-block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
          ${escapeHtml(e.link_url)}
        </a>
      </td>
      <td style="max-width: 250px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
        ${escapeHtml(e.motto||"-")}
      </td>
      <td>
        <span style="color: ${e.is_enabled?"#28a745":"#dc3545"}; font-weight: bold;">
          ${e.is_enabled?"✓ 启用":"✕ 禁用"}
        </span>
      </td>
      <td>
        <button class="btn-secondary" onclick="toggleFriendLinkStatus(${e.id})" title="${e.is_enabled?"禁用":"启用"}">
          ${e.is_enabled?"禁用":"启用"}
        </button>
        <button class="btn-primary" onclick="openEditFriendLinkModal(${e.id})" title="编辑">编辑</button>
        <button class="btn-danger" onclick="deleteFriendLink(${e.id})" title="删除">删除</button>
      </td>
    </tr>
  `).join(""), document.querySelectorAll(".friend-link-checkbox").forEach(e => {
        e.addEventListener("change", function() {
            toggleFriendLinkSelection(parseInt(this.getAttribute("data-id")))
        })
    }), updateBatchDeleteButton(), updateSelectAllCheckbox))()
}

function toggleFriendLinkSelection(e) {
    var t = selectedFriendLinkIds.indexOf(e); - 1 < t ? selectedFriendLinkIds.splice(t, 1) : selectedFriendLinkIds.push(e), updateBatchDeleteButton(), updateSelectAllCheckbox()
}

function toggleSelectAllFriendLinks() {
    const t = document.getElementById("selectAllFriendLinks");
    selectedFriendLinkIds = t.checked ? friendLinks.map(e => e.id) : [], document.querySelectorAll(".friend-link-checkbox").forEach(e => {
        e.checked = t.checked
    }), updateBatchDeleteButton()
}

function updateSelectAllCheckbox() {
    var e = document.getElementById("selectAllFriendLinks"),
        t = document.querySelectorAll(".friend-link-checkbox");
    0 === t.length ? (e.checked = !1, e.disabled = !0) : (e.disabled = !1, e.checked = 0 < t.length && selectedFriendLinkIds.length === friendLinks.length)
}

function updateBatchDeleteButton() {
    var e = document.getElementById("batchDeleteFriendLinksBtn"),
        t = document.getElementById("friendLinkSelectedCount");
    0 < selectedFriendLinkIds.length ? (e.style.display = "inline-flex", t.textContent = selectedFriendLinkIds.length) : e.style.display = "none"
}
async function batchDeleteFriendLinks() {
    if (0 !== selectedFriendLinkIds.length) {
        if (confirm(`确定要删除选中的 ${selectedFriendLinkIds.length} 个友链吗？
此操作不可恢复！`)) try {
            var e = localStorage.getItem("auth_token"),
                t = await (await fetch("/api/admin/friend-links/batch-delete", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: "Bearer " + e
                    },
                    body: JSON.stringify({
                        ids: selectedFriendLinkIds
                    })
                })).json();
            t.success ? (showToast(t.message || "批量删除成功", "success"), selectedFriendLinkIds = [], updateBatchDeleteButton(), await loadFriendLinks()) : showToast(t.message || "批量删除失败", "error")
        } catch (e) {
            console.error("批量删除失败:", e), showToast("批量删除失败，请重试", "error")
        }
    } else showToast("请先选择要删除的友链", "warning")
}

function openAddFriendLinkModal() {
    document.getElementById("friendLinkModalTitle").textContent = "添加友情链接", document.getElementById("friendLinkId").value = "", document.getElementById("friendLinkNickname").value = "", document.getElementById("friendLinkUrl").value = "", document.getElementById("friendLinkAvatar").value = "", document.getElementById("friendLinkMotto").value = "", document.getElementById("friendLinkSortOrder").value = "0", document.getElementById("friendLinkEnabled").checked = !0, openModal("friendLinkModal")
}
async function saveFriendLink(e) {
    e.preventDefault();
    var t = document.getElementById("friendLinkId").value,
        n = !!t,
        o = {
            nickname: document.getElementById("friendLinkNickname").value.trim(),
            link_url: document.getElementById("friendLinkUrl").value.trim(),
            avatar_url: document.getElementById("friendLinkAvatar").value.trim(),
            motto: document.getElementById("friendLinkMotto").value.trim(),
            sort_order: parseInt(document.getElementById("friendLinkSortOrder").value) || 0,
            is_enabled: document.getElementById("friendLinkEnabled").checked
        };
    if (o.nickname && o.link_url) try {
        const e = {
                "Content-Type": "application/json",
                Authorization: "Bearer " + localStorage.getItem("auth_token")
            },
            a = n ? "/api/admin/friend-links/" + t : "/api/admin/friend-links",
            i = n ? "PUT" : "POST",
            d = await fetch(a, {
                method: i,
                headers: e,
                body: JSON.stringify(o)
            }),
            r = await d.json();
        r.success ? (showToast(n ? "友链更新成功" : "友链添加成功", "success"), closeModal("friendLinkModal"), await loadFriendLinks()) : showToast(r.message || "操作失败", "error")
    } catch (e) {
        console.error("保存友链失败:", e), showToast("操作失败，请重试", "error")
    } else showToast("昵称和链接地址不能为空", "error")
}

function initializeFriendLinksManagement() {
    var e = document.getElementById("addFriendLinkBtn");
    e && e.addEventListener("click", openAddFriendLinkModal), (e = document.getElementById("refreshFriendLinksBtn")) && e.addEventListener("click", loadFriendLinks), (e = document.getElementById("selectAllFriendLinks")) && e.addEventListener("change", toggleSelectAllFriendLinks), (e = document.getElementById("batchDeleteFriendLinksBtn")) && e.addEventListener("click", batchDeleteFriendLinks), (e = document.getElementById("friendLinkForm")) && e.addEventListener("submit", saveFriendLink)
}
window.openEditFriendLinkModal = function(t) {
    var e = friendLinks.find(e => e.id === t);
    e ? (document.getElementById("friendLinkModalTitle").textContent = "编辑友情链接", document.getElementById("friendLinkId").value = e.id, document.getElementById("friendLinkNickname").value = e.nickname, document.getElementById("friendLinkUrl").value = e.link_url, document.getElementById("friendLinkAvatar").value = e.avatar_url, document.getElementById("friendLinkMotto").value = e.motto, document.getElementById("friendLinkSortOrder").value = e.sort_order, document.getElementById("friendLinkEnabled").checked = e.is_enabled, openModal("friendLinkModal")) : showToast("友链不存在", "error")
}, window.toggleFriendLinkStatus = async function(t) {
    var e = friendLinks.find(e => e.id === t);
    if (e) try {
        var n = {
                "Content-Type": "application/json",
                Authorization: "Bearer " + localStorage.getItem("auth_token")
            },
            o = await (await fetch("/api/admin/friend-links/batch-update-status", {
                method: "POST",
                headers: n,
                body: JSON.stringify({
                    ids: [t],
                    is_enabled: !e.is_enabled
                })
            })).json();
        o.success ? (showToast(o.message || "状态更新成功", "success"), await loadFriendLinks()) : showToast(o.message || "状态更新失败", "error")
    } catch (t) {
        console.error("切换状态失败:", t), showToast("操作失败，请重试", "error")
    }
}, window.deleteFriendLink = function(t) {
    var e = friendLinks.find(e => e.id === t);
    e && confirm(`确定要删除友链 "${e.nickname}" 吗？
此操作不可恢复！`) && (e = localStorage.getItem("auth_token"), fetch("/api/admin/friend-links/" + t, {
        method: "DELETE",
        headers: {
            Authorization: "Bearer " + e
        }
    }).then(e => e.json()).then(e => {
        e.success ? (showToast("友链删除成功", "success"), loadFriendLinks()) : showToast(e.message || "删除失败", "error")
    }).catch(e => {
        console.error("删除友链失败:", e), showToast("删除失败，请重试", "error")
    }))
};
const FileManager = {
        currentPath: "img",
        currentRoot: "img",
        selectedFile: null,
        filesToUpload: [],
        getAuthHeader() {
            var e = localStorage.getItem("auth_token");
            return e ? "Bearer " + e : ""
        },
        init() {
            this.bindEvents(), this.loadTree(), this.loadFiles()
        },
        bindEvents() {
            document.getElementById("fmUploadBtn").addEventListener("click", () => this.openUploadModal()), document.getElementById("fmCreateDirBtn").addEventListener("click", () => this.openCreateDirModal()), document.getElementById("fmBackBtn").addEventListener("click", () => this.goBack()), document.getElementById("confirmCreateDirBtn").addEventListener("click", () => {
                var e = document.getElementById("dirNameInput");
                (e = e && e.value.trim()) && (this.createDirectory(e), document.getElementById("createDirModal").classList.remove("active"))
            }), document.getElementById("confirmRenameBtn").addEventListener("click", () => {
                var e = document.getElementById("renameInput");
                (e = e && e.value.trim()) && (this.renameFile(e), document.getElementById("renameModal").classList.remove("active"))
            }), document.addEventListener("click", e => {
                e.target.closest(".fm-context-menu") || e.target.closest(".fm-file-item") || this.hideContextMenu()
            }), document.addEventListener("keydown", e => {
                "Escape" === e.key && this.hideContextMenu()
            })
        },
        async loadTree() {
            try {
                var e = document.getElementById("fmTree");
                e.innerHTML = "";
                for (const n of ["img", "markdown", "attachments", "music"]) {
                    var t = this.createTreeItem(n, !0);
                    e.appendChild(t), await this.loadSubDirectories(n, t)
                }
            } catch (e) {
                console.error("加载目录树失败:", e)
            }
        },
        async loadSubDirectories(e, t) {
            try {
                var n = await (await fetch("/api/files?path=" + encodeURIComponent(e), {
                    headers: {
                        Authorization: this.getAuthHeader()
                    }
                })).json();
                if (n.success && n.data.files) {
                    var o = n.data.files.filter(e => e.is_dir);
                    if (0 < o.length) {
                        var a = document.createElement("div");
                        a.className = "fm-tree-children";
                        for (const t of o) {
                            var i = e + "/" + t.name,
                                d = this.createTreeItem(t.name, !1, i);
                            a.appendChild(d), await this.loadSubDirectories(i, d)
                        }
                        t.appendChild(a)
                    }
                }
            } catch (e) {
                console.error("加载子目录失败:", e)
            }
        },
        createTreeItem(e, t, n = null) {
            const o = document.createElement("div");
            o.className = "fm-tree-item", t && e === this.currentRoot && o.classList.add("active");
            var a = document.createElement("span");
            a.className = "fm-tree-icon", a.innerHTML = t ? '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>' : '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path><path d="M12 11v6"></path><path d="M9 14l3 3 3-3"></path></svg>', o.appendChild(a), (t = document.createElement("span")).textContent = e, o.appendChild(t);
            const i = n || e;
            return o.addEventListener("click", e => {
                e.stopPropagation(), this.navigateTo(i), document.querySelectorAll(".fm-tree-item").forEach(e => e.classList.remove("active")), o.classList.add("active")
            }), o
        },
        async loadFiles() {
            try {
                var e = await (await fetch("/api/files?path=" + encodeURIComponent(this.currentPath), {
                    headers: {
                        Authorization: this.getAuthHeader()
                    }
                })).json();
                e.success ? (this.renderFiles(e.data.files), this.updateBreadcrumb(e.data.current_path), this.updateBackButton(e.data.parent_path), this.updateFileCount(e.data.files.length)) : showToast(e.message, "error")
            } catch (e) {
                console.error("加载文件失败:", e), showToast("加载文件失败", "error")
            }
        },
        renderFiles(e) {
            var t = document.getElementById("fmFileList"),
                n = document.getElementById("fmEmptyState");
            0 === e.length ? (t.innerHTML = "", n.style.display = "flex") : (n.style.display = "none", n = [...e].sort((e, t) => e.is_dir && !t.is_dir ? -1 : !e.is_dir && t.is_dir ? 1 : e.name.localeCompare(t.name)), t.innerHTML = n.map(e => this.createFileItem(e)).join(""), t.querySelectorAll(".fm-file-item").forEach(o => {
                o.addEventListener("click", e => {
                    e.stopPropagation(), e = o.dataset.path, "true" === o.dataset.isDir ? this.navigateTo(e) : this.openFile(e)
                }), o.addEventListener("contextmenu", e => {
                    e.preventDefault();
                    var t = o.dataset.path,
                        n = "true" === o.dataset.isDir;
                    this.showContextMenu(e, t, n)
                })
            }))
        },
        createFileItem(e) {
            let t = this.getFileIcon("default"),
                n;
            n = e.is_dir ? (t = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>', "directory") : [".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp", ".svg", ".ico", ".tiff", ".tif", ".avif", ".jxl"].includes(e.extension) ? (t = `<img src="/${e.path}" alt="${e.name}" onerror="this.parentElement.innerHTML='<svg width=\\'16\\' height=\\'16\\' viewBox=\\'0 0 24 24\\' fill=\\'none\\' stroke=\\'currentColor\\' stroke-width=\\'2\\'><rect x=\\'3\\' y=\\'3\\' width=\\'18\\' height=\\'18\\' rx=\\'3\\' ry=\\'3\\'></rect><circle cx=\\'8.5\\' cy=\\'8.5\\' r=\\'1.5\\'></circle><path d=\\'21 15l-5-5L5 21\\'></path></svg>'">`, "image") : [".mp3", ".flac", ".wav", ".ogg", ".m4a", ".aac", ".wma", ".opus", ".ape", ".wv", ".tta"].includes(e.extension) ? (t = this.getFileIcon("audio"), "audio") : [".mp4", ".webm", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".m4v", ".3gp", ".ts", ".m2ts"].includes(e.extension) ? (t = this.getFileIcon("video"), "video") : ".md" === e.extension ? (t = this.getFileIcon("markdown"), "markdown") : ".pdf" === e.extension ? (t = this.getFileIcon("pdf"), "pdf") : [".doc", ".docx", ".odt", ".rtf"].includes(e.extension) ? (t = this.getFileIcon("word"), "word") : [".xls", ".xlsx", ".ods", ".csv"].includes(e.extension) ? (t = this.getFileIcon("excel"), "excel") : [".ppt", ".pptx", ".odp"].includes(e.extension) ? (t = this.getFileIcon("ppt"), "ppt") : [".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz"].includes(e.extension) ? (t = this.getFileIcon("archive"), "archive") : [".html", ".htm", ".css", ".js", ".ts", ".jsx", ".tsx", ".vue", ".svelte", ".json", ".xml", ".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf"].includes(e.extension) ? (t = this.getFileIcon("code"), "code") : [".txt", ".log"].includes(e.extension) ? (t = this.getFileIcon("text"), "text") : [".ttf", ".otf", ".woff", ".woff2", ".eot"].includes(e.extension) ? (t = this.getFileIcon("font"), "font") : [".db", ".sqlite", ".sqlite3", ".mdb", ".sql"].includes(e.extension) ? (t = this.getFileIcon("database"), "database") : [".exe", ".app", ".dmg", ".msi", ".deb", ".rpm", ".sh", ".bat", ".cmd", ".ps1"].includes(e.extension) ? (t = this.getFileIcon("executable"), "executable") : (t = this.getFileIcon("default"), "file");
            var o = this.formatFileSize(e.size);
            return `
      <div class="fm-file-item ${n}" data-path="${e.path}" data-is-dir="${e.is_dir}">
        <div class="fm-file-icon">${t}</div>
        <div class="fm-file-name">${e.name}</div>
        <div class="fm-file-meta">${e.is_dir?"文件夹":o}</div>
      </div>
    `
        },
        getFileIcon(e) {
            return {
                image: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="3" ry="3"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>',
                video: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3" fill="currentColor"/><rect x="2" y="2" width="20" height="20" rx="3" ry="3"/></svg>',
                audio: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3" fill="currentColor"/><circle cx="18" cy="16" r="3" fill="currentColor"/></svg>',
                document: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>',
                archive: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>',
                markdown: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M10 13l-2 2 2 2"/><path d="M14 13l2 2-2 2"/></svg>',
                code: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 18l6-6-6-6"/><path d="M8 6l-6 6 6 6"/></svg>',
                pdf: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><text x="7" y="17" font-size="6" font-weight="bold" fill="currentColor">PDF</text></svg>',
                word: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><text x="4" y="17" font-size="5" font-weight="bold" fill="currentColor">DOC</text></svg>',
                excel: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><rect x="7" y="10" width="10" height="2"/><rect x="7" y="14" width="10" height="2"/><rect x="7" y="18" width="10" height="2"/></svg>',
                ppt: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><rect x="8" y="11" width="8" height="6" rx="1"/></svg>',
                text: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><line x1="10" y1="9" x2="8" y2="9"/></svg>',
                font: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><text x="6" y="18" font-size="10" font-weight="bold" fill="currentColor">Aa</text></svg>',
                database: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>',
                executable: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><circle cx="12" cy="14" r="3"/><line x1="12" y1="11" x2="12" y2="17"/><line x1="9" y1="14" x2="15" y2="14"/></svg>'
            } [e] || '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/></svg>'
        },
        formatFileSize(e) {
            var t;
            return 0 === e ? "0 B" : (t = Math.floor(Math.log(e) / Math.log(1024)), parseFloat((e / Math.pow(1024, t)).toFixed(2)) + " " + ["B", "KB", "MB", "GB"][t])
        },
        updateBreadcrumb(e) {
            document.getElementById("fmBreadcrumb").textContent = e || "/"
        },
        updateBackButton(e) {
            document.getElementById("fmBackBtn").disabled = !e
        },
        updateFileCount(e) {
            document.getElementById("fmInfo").textContent = e + " 个项目"
        },
        navigateTo(e) {
            this.currentPath = e, this.loadFiles()
        },
        goBack() {
            var e = this.getParentPath(this.currentPath);
            e && this.navigateTo(e)
        },
        getParentPath(e) {
            return e === this.currentRoot ? null : ((e = e.split("/")).pop(), e.join("/") || this.currentRoot)
        },
        openFile(t) {
            const e = t.split("/").pop().split(".").pop().toLowerCase();
            if (["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"].includes(e)) {
                const e = "/" + t;
                this.showImagePreview(e)
            } else {
                if ("md" === e) {
                    let e = t;
                    if (e.startsWith("/")) {
                        var n = e.indexOf("/markdown/");
                        if (-1 === n) return console.error("无效的 Markdown 路径:", t), void this.showToast("无效的 Markdown 路径", "error");
                        e = e.substring(n + 10)
                    } else {
                        if (!e.startsWith("markdown/")) return console.error("无效的 Markdown 路径:", t), void this.showToast("无效的 Markdown 路径", "error");
                        e = e.substring(9)
                    }
                    return e && "/" !== e && "" !== e.trim() ? void(window.MarkdownPreviewModal ? window.MarkdownPreviewModal.open(e) : console.error("MarkdownPreviewModal not available")) : (console.error("提取后的 Markdown 路径无效:", e), void this.showToast("无效的 Markdown 路径", "error"))
                }
                this.downloadFile(t)
            }
        },
        showImagePreview(e) {
            const t = document.createElement("div");
            t.style.cssText = "\n      position: fixed;\n      top: 0;\n      left: 0;\n      width: 100%;\n      height: 100%;\n      background: rgba(0, 0, 0, 0.9);\n      z-index: 10000;\n      display: flex;\n      align-items: center;\n      justify-content: center;\n      cursor: pointer;\n    ";
            var n = document.createElement("img");
            n.src = e, n.style.cssText = "\n      max-width: 90%;\n      max-height: 90%;\n      object-fit: contain;\n      border-radius: 8px;\n    ", (e = document.createElement("div")).textContent = "点击关闭", e.style.cssText = "\n      position: absolute;\n      bottom: 30px;\n      color: white;\n      font-size: 16px;\n      opacity: 0.7;\n    ", t.appendChild(n), t.appendChild(e), t.addEventListener("click", () => {
                document.body.removeChild(t)
            });
            const o = e => {
                "Escape" === e.key && (document.body.removeChild(t), document.removeEventListener("keydown", o))
            };
            document.addEventListener("keydown", o), document.body.appendChild(t)
        },
        async downloadFile(e) {
            try {
                var t = await fetch("/api/files/download?path=" + encodeURIComponent(e), {
                    headers: {
                        Authorization: this.getAuthHeader()
                    }
                });
                if (t.ok) {
                    var n = e.split("/").pop(),
                        o = await t.blob(),
                        a = window.URL.createObjectURL(o),
                        i = document.createElement("a");
                    i.href = a, i.download = n, document.body.appendChild(i), i.click(), document.body.removeChild(i), window.URL.revokeObjectURL(a)
                } else {
                    const e = await t.json();
                    showToast(e.message || "下载失败", "error")
                }
            } catch (e) {
                console.error("下载失败:", e), showToast("下载失败", "error")
            }
        },
        showContextMenu(e, t, n) {
            this.selectedFile = {
                path: t,
                isDir: n
            };
            var o = document.querySelector(".fm-context-menu");
            o && o.remove();
            const a = document.createElement("div");
            a.className = "fm-context-menu", a.style.left = e.pageX + "px", a.style.top = e.pageY + "px", o = t.split("/").pop().split(".").pop().toLowerCase(), ["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"].includes(o), [{
                action: "open",
                label: "打开",
                icon: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path><path d="M12 11v6"></path><path d="M9 14l3 3 3-3"></path></svg>'
            }, {
                action: "download",
                label: "下载",
                icon: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>',
                hide: n
            }, {
                action: "rename",
                label: "重命名",
                icon: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>'
            }, {
                action: "delete",
                label: "删除",
                icon: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>️',
                danger: !0
            }].forEach(e => {
                var t;
                e.hide || ((t = document.createElement("div")).className = "fm-context-menu-item" + (e.danger ? " danger" : ""), t.innerHTML = `<span>${e.icon}</span>` + e.label, t.addEventListener("click", () => {
                    this.handleContextAction(e.action), this.hideContextMenu()
                }), a.appendChild(t))
            }), document.body.appendChild(a), a.classList.add("active")
        },
        hideContextMenu() {
            var e = document.querySelector(".fm-context-menu");
            e && e.remove()
        },
        handleContextAction(e) {
            if (this.selectedFile) switch (e) {
                case "open":
                    this.selectedFile.isDir ? this.navigateTo(this.selectedFile.path) : this.openFile(this.selectedFile.path);
                    break;
                case "download":
                    this.downloadFile(this.selectedFile.path);
                    break;
                case "rename":
                    this.openRenameModal();
                    break;
                case "delete":
                    this.openDeleteModal()
            }
        },
        openUploadModal() {
            const t = document.createElement("input");
            t.type = "file", t.multiple = !0, t.style.display = "none", t.addEventListener("change", e => {
                0 < (e = Array.from(e.target.files)).length && this.uploadFiles(e), t.remove()
            }), document.body.appendChild(t), t.click()
        },
        async uploadFiles(e) {
            for (const t of e) try {
                const e = new FormData;
                e.append("file", t), (await (await fetch("/api/files?path=" + encodeURIComponent(this.currentPath), {
                    method: "POST",
                    headers: {
                        Authorization: this.getAuthHeader()
                    },
                    body: e
                })).json()).success ? showToast("成功上传 " + t.name, "success") : showToast(`上传 ${t.name} 失败`, "error")
            } catch (e) {
                console.error("上传失败:", e), showToast(`上传 ${t.name} 失败`, "error")
            }
            this.loadFiles(), this.loadTree()
        },
        openCreateDirModal() {
            var e = document.getElementById("createDirModal");
            const t = document.getElementById("dirNameInput");
            t && (t.value = "", e.classList.add("active"), setTimeout(() => {
                t.focus()
            }, 100))
        },
        async createDirectory(e) {
            try {
                var t = await (await fetch("/api/files/create-dir", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: this.getAuthHeader()
                    },
                    body: JSON.stringify({
                        path: this.currentPath,
                        name: e
                    })
                })).json();
                t.success ? (showToast("文件夹创建成功", "success"), this.loadFiles(), this.loadTree()) : showToast(t.message, "error")
            } catch (e) {
                console.error("创建目录失败:", e), showToast("创建文件夹失败", "error")
            }
        },
        openRenameModal() {
            if (this.selectedFile) {
                var e = this.selectedFile.path.split("/").pop(),
                    t = document.getElementById("renameModal");
                const n = document.getElementById("renameInput");
                n && (n.value = e, t.classList.add("active"), setTimeout(() => {
                    n.focus(), n.select()
                }, 100))
            }
        },
        async renameFile(e) {
            if (this.selectedFile) try {
                var t = await (await fetch("/api/files", {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: this.getAuthHeader()
                    },
                    body: JSON.stringify({
                        old_path: this.selectedFile.path,
                        new_name: e
                    })
                })).json();
                t.success ? (showToast("重命名成功", "success"), this.loadFiles(), this.loadTree()) : showToast(t.message, "error")
            } catch (e) {
                console.error("重命名失败:", e), showToast("重命名失败", "error")
            }
        },
        openDeleteModal() {
            var e;
            this.selectedFile && (e = this.selectedFile.path.split("/").pop(), confirm(`确定要删除 "${e}" 吗？此操作不可恢复！`)) && this.deleteFile()
        },
        async deleteFile() {
            if (this.selectedFile) try {
                var e = await (await fetch("/api/files?path=" + encodeURIComponent(this.selectedFile.path), {
                    method: "DELETE",
                    headers: {
                        Authorization: this.getAuthHeader()
                    }
                })).json();
                e.success ? (showToast("删除成功", "success"), this.loadFiles(), this.loadTree()) : showToast(e.message, "error")
            } catch (e) {
                console.error("删除失败:", e), showToast("删除失败", "error")
            }
        }
    },
    AttachmentUploader = {
        currentArticleId: null,
        init() {
            var e = document.getElementById("uploadAttachmentBtn");
            e && e.addEventListener("click", () => this.uploadAttachment())
        },
        async uploadAttachment() {
            const e = document.getElementById("attachmentFile"),
                t = document.getElementById("uploadAttachmentArticleId"),
                n = document.getElementById("uploadAttachmentProgress"),
                o = document.getElementById("uploadAttachmentResult"),
                a = document.getElementById("uploadAttachmentProgressBar"),
                i = document.getElementById("uploadAttachmentStatus"),
                d = document.getElementById("uploadAttachmentFileInfo");
            if (e.files && 0 !== e.files.length) {
                var r = e.files[0],
                    l = (this.currentArticleId = t.dataset.articleId, n.style.display = "block", o.style.display = "none", a.style.width = "0%", i.textContent = "正在上传...", new FormData);
                l.append("file", r), l.append("passage_id", this.currentArticleId);
                try {
                    const e = localStorage.getItem("auth_token"),
                        t = await fetch("/api/admin/attachments", {
                            method: "POST",
                            headers: e ? {
                                Authorization: "Bearer " + e
                            } : {},
                            body: l
                        }),
                        n = await t.json();
                    a.style.width = "100%", n.success ? (i.textContent = "上传成功！", o.style.display = "block", d.innerHTML = `
          <div><strong>文件名：</strong>${n.data.fileName}</div>
          <div><strong>文件大小：</strong>${this.formatFileSize(n.data.size)}</div>
          <div><strong>文件类型：</strong>${n.data.type}</div>
          <div><strong>访问URL：</strong><a href="${n.data.url}" target="_blank" style="color: #007bff;">${n.data.url}</a></div>
        `, showToast("附件上传成功！", "success"), setTimeout(() => {
                        closeModal("uploadAttachmentModal")
                    }, 3e3)) : (i.textContent = "上传失败：" + (n.message || "未知错误"), showToast("附件上传失败：" + (n.message || "未知错误"), "error"))
                } catch (e) {
                    console.error("上传附件失败:", e), a.style.width = "100%", i.textContent = "上传失败：网络错误", showToast("附件上传失败，请稍后重试", "error")
                }
            } else showToast("请选择要上传的文件", "error")
        },
        formatFileSize(e) {
            var t;
            return 0 === e ? "0 Bytes" : (t = Math.floor(Math.log(e) / Math.log(1024)), Math.round(e / Math.pow(1024, t) * 100) / 100 + " " + ["Bytes", "KB", "MB", "GB"][t])
        }
    };
document.addEventListener("DOMContentLoaded", () => {
    AttachmentUploader.init();
    var e = document.querySelector('[data-tab="filemanager"]');
    e && e.addEventListener("click", () => {
        setTimeout(() => {
            FileManager.initialized || (FileManager.init(), FileManager.initialized = !0)
        }, 100)
    })
});
