let originalAppearanceSettings = {};
async function loadSettings() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/settings/appearance', {
      headers: t,
    });
    if (n.ok) {
      const e = await n.json();
      ((originalAppearanceSettings = {
        ...e,
      }),
        (document.getElementById('backgroundImage').value = e.background_image || '/img/test.webp'),
        (document.getElementById('mobileBackgroundImage').value =
          e.mobile_background_image || '/img/mobile-test.webp'),
        (document.getElementById('globalOpacity').value = e.global_opacity || '0.15'),
        (document.getElementById('opacityValue').textContent = e.global_opacity || '0.15'),
        (document.getElementById('backgroundSize').value = e.background_size || 'cover'),
        (document.getElementById('backgroundPosition').value = e.background_position || 'center'),
        (document.getElementById('backgroundRepeat').value = e.background_repeat || 'no-repeat'),
        (document.getElementById('backgroundAttachment').value =
          e.background_attachment || 'fixed'),
        (document.getElementById('blurAmount').value = e.blur_amount || '20px'),
        (document.getElementById('saturateAmount').value = e.saturate_amount || '180%'),
        (document.getElementById('darkModeEnabled').checked = e.dark_mode_enabled || !1),
        (document.getElementById('navbarGlassColor').value =
          e.navbar_glass_color || 'rgba(255, 255, 255, 0.85)'),
        (document.getElementById('navbarTextColor').value = e.navbar_text_color || '#333333'),
        (document.getElementById('cardGlassColor').value =
          e.card_glass_color || 'rgba(255, 255, 255, 0.75)'),
        (document.getElementById('footerGlassColor').value =
          e.footer_glass_color || 'rgba(255, 255, 255, 0.9)'),
        (document.getElementById('floatingTextEnabled').checked = e.floating_text_enabled || !1),
        (document.getElementById('floatingTexts').value =
          e.floating_texts && Array.isArray(e.floating_texts)
            ? e.floating_texts.join(', ')
            : 'perfect, good, excellent, extraordinary, legend'),
        updateColorPickers(),
        applyDarkMode(e.dark_mode_enabled || !1),
        applyGlassColors(e.navbar_glass_color, e.card_glass_color, e.footer_glass_color),
        updatePreview());
    } else console.error('加载设置失败:', n.statusText);
  } catch (e) {
    console.error('加载设置失败:', e);
  }
}
async function saveSettings() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = {
        background_image: document.getElementById('backgroundImage').value,
        mobile_background_image: document.getElementById('mobileBackgroundImage').value,
        global_opacity: document.getElementById('globalOpacity').value,
        background_size: document.getElementById('backgroundSize').value,
        background_position: document.getElementById('backgroundPosition').value,
        background_repeat: document.getElementById('backgroundRepeat').value,
        background_attachment: document.getElementById('backgroundAttachment').value,
        blur_amount: document.getElementById('blurAmount').value,
        saturate_amount: document.getElementById('saturateAmount').value,
        dark_mode_enabled: document.getElementById('darkModeEnabled').checked,
        navbar_glass_color: document.getElementById('navbarGlassColor').value,
        navbar_text_color: document.getElementById('navbarTextColor').value,
        card_glass_color: document.getElementById('cardGlassColor').value,
        footer_glass_color: document.getElementById('footerGlassColor').value,
        floating_text_enabled: document.getElementById('floatingTextEnabled').checked,
        floating_texts: document
          .getElementById('floatingTexts')
          .value.split(',')
          .map(e => e.trim())
          .filter(e => e.length > 0),
      },
      o = {};
    for (const e in n) {
      const t = originalAppearanceSettings[e],
        a = n[e];
      ('navbar_text_color' === e &&
        (console.log('navbar_text_color 比对:'),
        console.log('  原始值:', t, '(类型:', typeof t, ')'),
        console.log('  当前值:', a, '(类型:', typeof a, ')'),
        console.log('  是否相等:', a === t)),
        a !== t && (o[e] = a));
    }
    if (
      (console.log('原始设置:', originalAppearanceSettings),
      console.log('当前设置:', n),
      console.log('变更设置:', o),
      0 === Object.keys(o).length)
    )
      return void showToast('没有检测到任何变化', 'warning');
    console.log('发送变更的外观字段:', o);
    const a = await fetch('/api/settings/appearance', {
      method: 'PATCH',
      headers: t,
      body: JSON.stringify(o),
    });
    if (a.ok)
      (showToast('设置保存成功！', 'success'),
        Object.assign(originalAppearanceSettings, o),
        'dark_mode_enabled' in o && applyDarkMode(o.dark_mode_enabled),
        updatePreview());
    else {
      const e = await a.json();
      showToast('保存失败：' + (e.error || '未知错误'), 'error');
    }
  } catch (e) {
    (console.error('保存设置失败:', e), showToast('保存失败，请稍后重试', 'error'));
  }
}

function resetSettings() {
  confirm('确定要重置为默认设置吗？') &&
    ((document.getElementById('backgroundImage').value = '/img/test.webp'),
    (document.getElementById('globalOpacity').value = '0.15'),
    (document.getElementById('opacityValue').textContent = '0.15'),
    (document.getElementById('backgroundSize').value = 'cover'),
    (document.getElementById('backgroundPosition').value = 'center'),
    (document.getElementById('backgroundRepeat').value = 'no-repeat'),
    (document.getElementById('backgroundAttachment').value = 'fixed'),
    (document.getElementById('blurAmount').value = '20px'),
    (document.getElementById('saturateAmount').value = '180%'),
    (document.getElementById('darkModeEnabled').checked = !1),
    (document.getElementById('navbarGlassColor').value = 'rgba(255, 255, 255, 0.85)'),
    (document.getElementById('navbarTextColor').value = '#333333'),
    (document.getElementById('cardGlassColor').value = 'rgba(255, 255, 255, 0.75)'),
    (document.getElementById('footerGlassColor').value = 'rgba(255, 255, 255, 0.9)'),
    (document.getElementById('floatingTextEnabled').checked = !1),
    (document.getElementById('floatingTexts').value =
      'perfect, good, excellent, extraordinary, legend'),
    updateColorPickers(),
    applyDarkMode(!1),
    applyGlassColors(
      'rgba(255, 255, 255, 0.85)',
      'rgba(255, 255, 255, 0.75)',
      'rgba(255, 255, 255, 0.9)'
    ),
    updatePreview());
}

function updatePreview() {
  const e = document.getElementById('previewBox');
  if (!e) return;
  const t = document.getElementById('backgroundImage').value,
    n = document.getElementById('mobileBackgroundImage').value,
    o = document.getElementById('globalOpacity').value,
    a = document.getElementById('backgroundSize').value,
    d = document.getElementById('backgroundPosition').value,
    i = document.getElementById('backgroundRepeat').value,
    r = document.getElementById('backgroundAttachment').value,
    l = document.getElementById('blurAmount').value,
    s = document.getElementById('saturateAmount').value,
    c = window.innerWidth <= 768 && n ? n : t;
  ((e.style.backgroundImage = `url('${c}')`),
    (e.style.backgroundSize = a),
    (e.style.backgroundPosition = d),
    (e.style.backgroundRepeat = i),
    (e.style.backgroundAttachment = r),
    e.style.setProperty('--blur-amount', l),
    e.style.setProperty('--saturate-amount', s),
    e.style.setProperty('--global-opacity', o));
  const u = document.createElement('style');
  u.textContent = `\n    #previewBox::before {\n      background-image: url('${c}') !important;\n      background-size: ${a} !important;\n      background-position: ${d} !important;\n      background-repeat: ${i} !important;\n      background-attachment: ${r} !important;\n      filter: blur(${l}) saturate(${s}) !important;\n    }\n    #previewBox {\n      background: rgba(255, 255, 255, ${o}) !important;\n      backdrop-filter: blur(${l}) saturate(${s}) !important;\n      -webkit-backdrop-filter: blur(${l}) saturate(${s}) !important;\n    }\n  `;
  const m = document.getElementById('preview-style');
  (m && m.remove(), (u.id = 'preview-style'), document.head.appendChild(u));
}

function applyDarkMode(e) {
  e
    ? document.documentElement.classList.add('dark-mode')
    : document.documentElement.classList.remove('dark-mode');
}

function updateColorPickers() {
  const e = document.getElementById('navbarGlassColor').value,
    t = document.getElementById('navbarTextColor').value,
    n = document.getElementById('cardGlassColor').value,
    o = document.getElementById('footerGlassColor').value;
  if (e.startsWith('rgba')) {
    const t = parseRgba(e);
    t && (document.getElementById('navbarGlassColorPicker').value = rgbaToHex(t.r, t.g, t.b));
  }
  if (t.startsWith('#')) document.getElementById('navbarTextColorPicker').value = t;
  else if (t.startsWith('rgb')) {
    const e = parseRgba(t);
    e && (document.getElementById('navbarTextColorPicker').value = rgbaToHex(e.r, e.g, e.b));
  }
  if (n.startsWith('rgba')) {
    const e = parseRgba(n);
    e && (document.getElementById('cardGlassColorPicker').value = rgbaToHex(e.r, e.g, e.b));
  }
  if (o.startsWith('rgba')) {
    const e = parseRgba(o);
    e && (document.getElementById('footerGlassColorPicker').value = rgbaToHex(e.r, e.g, e.b));
  }
}

function applyGlassColors(e, t, n) {
  (document.documentElement.style.setProperty('--navbar-glass-color', e),
    document.documentElement.style.setProperty('--card-glass-color', t),
    document.documentElement.style.setProperty('--footer-glass-color', n));
}

function parseRgba(e) {
  const t = e.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
  return t
    ? {
        r: parseInt(t[1]),
        g: parseInt(t[2]),
        b: parseInt(t[3]),
        a: t[4] ? parseFloat(t[4]) : 1,
      }
    : null;
}

function rgbaToHex(e, t, n) {
  const o = e => {
    const t = e.toString(16);
    return 1 === t.length ? '0' + t : t;
  };
  return '#' + o(e) + o(t) + o(n);
}
document.addEventListener('DOMContentLoaded', async function () {
  (await loadSettings(),
    await loadTemplateSettings(),
    await loadMusicSettings(),
    await loadMusicPlaylist());
  const e = document.getElementById('globalOpacity'),
    t = document.getElementById('opacityValue');
  e &&
    t &&
    e.addEventListener('input', function () {
      ((t.textContent = this.value), updatePreview());
    });
  const n = document.getElementById('saveSettingsBtn');
  n && n.addEventListener('click', saveSettings);
  const o = document.getElementById('saveTemplateSettingsBtn');
  o && o.addEventListener('click', saveTemplateSettings);
  const a = document.getElementById('saveMusicSettingsBtn');
  (a && a.addEventListener('click', saveMusicSettings), initMusicDragDrop());
  const d = document.getElementById('musicPlayerColorPicker'),
    i = document.getElementById('musicPlayerColor');
  d &&
    i &&
    d.addEventListener('input', function () {
      const e = parseRgba(i.value);
      if (e) {
        const t = `rgba(${this.value
          .match(/\w\w/g)
          .map(e => parseInt(e, 16))
          .join(', ')}, ${e.a})`;
        i.value = t;
      }
    });
  const r = document.getElementById('live2dEnabled'),
    l = document.getElementById('live2dConfig');
  r &&
    l &&
    r.addEventListener('change', function () {
      l.style.display = this.checked ? 'block' : 'none';
    });
  const s = document.getElementById('resetSettingsBtn');
  s && s.addEventListener('click', resetSettings);
  [
    {
      picker: 'navbarGlassColorPicker',
      input: 'navbarGlassColor',
    },
    {
      picker: 'navbarTextColorPicker',
      input: 'navbarTextColor',
    },
    {
      picker: 'cardGlassColorPicker',
      input: 'cardGlassColor',
    },
    {
      picker: 'footerGlassColorPicker',
      input: 'footerGlassColor',
    },
  ].forEach(({ picker: e, input: t }) => {
    const n = document.getElementById(e),
      o = document.getElementById(t);
    n &&
      o &&
      (n.addEventListener('input', function () {
        const e = o.value;
        if (e.startsWith('rgba')) {
          const t = parseRgba(e);
          if (t) {
            const e = `rgba(${this.value
              .match(/\w\w/g)
              .map(e => parseInt(e, 16))
              .join(', ')}, ${t.a})`;
            o.value = e;
          }
        } else o.value = this.value;
      }),
      o.addEventListener('input', function () {
        updateColorPickers();
      }));
  });
  ([
    'backgroundImage',
    'mobileBackgroundImage',
    'backgroundSize',
    'backgroundPosition',
    'backgroundRepeat',
    'backgroundAttachment',
    'blurAmount',
    'saturateAmount',
  ].forEach(e => {
    const t = document.getElementById(e);
    t && (t.addEventListener('input', updatePreview), t.addEventListener('change', updatePreview));
  }),
    window.addEventListener('resize', updatePreview));
  const c = document.querySelector('[data-tab="settings"]');
  c &&
    c.addEventListener('click', function () {
      (loadSettings(), loadTemplateSettings());
    });
});
let originalTemplateSettings = {};
async function loadTemplateSettings() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/settings/template', {
      method: 'GET',
      headers: t,
    });
    if (n.ok) {
      const e = await n.json();
      ((originalTemplateSettings = {
        ...e,
      }),
        (document.getElementById('templateName').value = e.name || ''),
        (document.getElementById('templateGreting').value = e.greting || ''),
        (document.getElementById('templateYear').value = e.year || ''),
        (document.getElementById('templateFoodes').value = e.foodes || ''),
        (document.getElementById('globalAvatar').value = e.global_avatar || '/img/avatar.webp'),
        (document.getElementById('templateArticleTitle').checked = e.article_title || !1),
        (document.getElementById('templateArticleTitlePrefix').value =
          e.article_title_prefix || ''),
        (document.getElementById('templateSwitchNotice').checked = e.switch_notice || !1),
        (document.getElementById('templateSwitchNoticeText').value = e.switch_notice_text || ''),
        (document.getElementById('externalLinkWarning').checked = e.external_link_warning || !1),
        (document.getElementById('externalLinkWhitelist').value = e.external_link_whitelist || ''),
        (document.getElementById('externalLinkWarningText').value =
          e.external_link_warning_text || ''),
        (document.getElementById('live2dEnabled').checked = e.live2d_enabled || !1),
        (document.getElementById('live2dShowOnIndex').checked = !1 !== e.live2d_show_on_index),
        (document.getElementById('live2dShowOnPassage').checked = !1 !== e.live2d_show_on_passage),
        (document.getElementById('live2dShowOnCollect').checked = !1 !== e.live2d_show_on_collect),
        (document.getElementById('live2dShowOnAbout').checked = !1 !== e.live2d_show_on_about),
        (document.getElementById('live2dShowOnAdmin').checked = e.live2d_show_on_admin || !1),
        (document.getElementById('live2dModelId').value = e.live2d_model_id || '1'),
        (document.getElementById('live2dModelPath').value = e.live2d_model_path || ''),
        (document.getElementById('live2dCDNPath').value =
          e.live2d_cdn_path || 'https://unpkg.com/live2d-widget-model@1.0.5/'),
        (document.getElementById('live2dPosition').value = e.live2d_position || 'right'),
        (document.getElementById('live2dWidth').value = e.live2d_width || '280px'),
        (document.getElementById('live2dHeight').value = e.live2d_height || '250px'));
      ((document.getElementById('live2dConfig').style.display = e.live2d_enabled
        ? 'block'
        : 'none'),
        (document.getElementById('sponsorEnabled').checked = e.sponsor_enabled || !1),
        (document.getElementById('sponsorTitle').value = e.sponsor_title || '感谢您的支持'),
        (document.getElementById('sponsorImage').value = e.sponsor_image || '/img/avatar.webp'),
        (document.getElementById('sponsorDescription').value =
          e.sponsor_description || '如果您觉得这个博客对您有帮助，欢迎赞助支持！'),
        (document.getElementById('sponsorButtonText').value =
          e.sponsor_button_text || '❤️ 赞助支持'),
        (document.getElementById('beianEnabled').checked = e.beian_enabled || !1),
        (document.getElementById('icpNumber').value = e.icp_number || ''),
        (document.getElementById('policeRecordCode').value = e.police_record_code || ''),
        (document.getElementById('policeRecordContent').value = e.police_record_content || ''));
    } else console.error('加载模板设置失败');
  } catch (e) {
    console.error('加载模板设置失败:', e);
  }
}
async function saveTemplateSettings() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = {
        name: document.getElementById('templateName').value,
        greting: document.getElementById('templateGreting').value,
        year: document.getElementById('templateYear').value,
        foodes: document.getElementById('templateFoodes').value,
        global_avatar: document.getElementById('globalAvatar').value,
        article_title: document.getElementById('templateArticleTitle').checked,
        article_title_prefix: document.getElementById('templateArticleTitlePrefix').value,
        switch_notice: document.getElementById('templateSwitchNotice').checked,
        switch_notice_text: document.getElementById('templateSwitchNoticeText').value,
        external_link_warning: document.getElementById('externalLinkWarning').checked,
        external_link_whitelist: document.getElementById('externalLinkWhitelist').value,
        external_link_warning_text: document.getElementById('externalLinkWarningText').value,
        live2d_enabled: document.getElementById('live2dEnabled').checked,
        live2d_show_on_index: document.getElementById('live2dShowOnIndex').checked,
        live2d_show_on_passage: document.getElementById('live2dShowOnPassage').checked,
        live2d_show_on_collect: document.getElementById('live2dShowOnCollect').checked,
        live2d_show_on_about: document.getElementById('live2dShowOnAbout').checked,
        live2d_show_on_admin: document.getElementById('live2dShowOnAdmin').checked,
        live2d_model_id: document.getElementById('live2dModelId').value,
        live2d_model_path: document.getElementById('live2dModelPath').value,
        live2d_cdn_path: document.getElementById('live2dCDNPath').value,
        live2d_position: document.getElementById('live2dPosition').value,
        live2d_width: document.getElementById('live2dWidth').value,
        live2d_height: document.getElementById('live2dHeight').value,
        sponsor_enabled: document.getElementById('sponsorEnabled').checked,
        sponsor_title: document.getElementById('sponsorTitle').value,
        sponsor_image: document.getElementById('sponsorImage').value,
        sponsor_description: document.getElementById('sponsorDescription').value,
        sponsor_button_text: document.getElementById('sponsorButtonText').value,
        beian_enabled: document.getElementById('beianEnabled').checked,
        icp_number: document.getElementById('icpNumber').value,
        police_record_code: document.getElementById('policeRecordCode').value,
        police_record_content: document.getElementById('policeRecordContent').value,
      },
      o = {};
    for (const e in n) n[e] !== originalTemplateSettings[e] && (o[e] = n[e]);
    if (0 === Object.keys(o).length) return void showToast('没有检测到任何变化', 'warning');
    console.log('发送变更的字段:', o);
    const a = await fetch('/api/settings/template', {
      method: 'PATCH',
      headers: t,
      body: JSON.stringify(o),
    });
    if (a.ok)
      (showToast('模板设置保存成功！', 'success'), Object.assign(originalTemplateSettings, o));
    else {
      const e = await a.json();
      showToast('保存失败：' + (e.error || '未知错误'), 'error');
    }
  } catch (e) {
    (console.error('保存模板设置失败:', e), showToast('保存失败，请稍后重试', 'error'));
  }
}
let originalMusicSettings = {};
async function loadMusicSettings() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/settings/music', {
      method: 'GET',
      headers: t,
    });
    if (n.ok) {
      const e = await n.json();
      ((originalMusicSettings = {
        ...e,
      }),
        (document.getElementById('musicEnabled').checked = e.enabled || !1),
        (document.getElementById('musicAutoPlay').checked = e.auto_play || !1),
        (document.getElementById('musicControlSize').value = e.control_size || 'medium'),
        (document.getElementById('musicPlayerColor').value =
          e.player_color || 'rgba(66, 133, 244, 0.9)'),
        (document.getElementById('musicPosition').value = e.position || 'bottom-right'),
        (document.getElementById('musicCustomCSS').value = e.custom_css || ''));
      const t = parseRgba(e.player_color || 'rgba(66, 133, 244, 0.9)');
      t && (document.getElementById('musicPlayerColorPicker').value = rgbaToHex(t.r, t.g, t.b));
    } else console.error('加载音乐设置失败');
  } catch (e) {
    console.error('加载音乐设置失败:', e);
  }
}
async function saveMusicSettings() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = {
        enabled: document.getElementById('musicEnabled').checked,
        auto_play: document.getElementById('musicAutoPlay').checked,
        control_size: document.getElementById('musicControlSize').value,
        player_color: document.getElementById('musicPlayerColor').value,
        position: document.getElementById('musicPosition').value,
        custom_css: document.getElementById('musicCustomCSS').value,
      },
      o = {};
    for (const e in n) n[e] !== originalMusicSettings[e] && (o[e] = n[e]);
    if (0 === Object.keys(o).length) return void showToast('没有检测到任何变化', 'warning');
    console.log('发送变更的音乐设置字段:', o);
    const a = await fetch('/api/settings/music', {
      method: 'PATCH',
      headers: t,
      body: JSON.stringify(o),
    });
    if (a.ok) (showToast('音乐设置保存成功！', 'success'), Object.assign(originalMusicSettings, o));
    else {
      const e = await a.json();
      showToast('保存失败：' + (e.error || '未知错误'), 'error');
    }
  } catch (e) {
    (console.error('保存音乐设置失败:', e), showToast('保存失败，请稍后重试', 'error'));
  }
}
let musicUploadQueue = [],
  isUploading = !1,
  uploadAbortController = null;

function initMusicDragDrop() {
  const e = document.getElementById('musicDropZone'),
    t = document.getElementById('musicFileUpload'),
    n = document.querySelector('.browse-link');
  if (!e || !t) return;

  function o(e) {
    (e.preventDefault(), e.stopPropagation());
  }
  (e.addEventListener('click', () => {
    t.click();
  }),
    n &&
      n.addEventListener('click', e => {
        (e.stopPropagation(), t.click());
      }),
    t.addEventListener('change', e => {
      (handleFileSelect(e.target.files), (t.value = ''));
    }),
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(t => {
      e.addEventListener(t, o, !1);
    }),
    ['dragenter', 'dragover'].forEach(t => {
      e.addEventListener(
        t,
        () => {
          e.classList.add('dragover');
        },
        !1
      );
    }),
    e.addEventListener(
      'dragleave',
      t => {
        e.contains(t.relatedTarget) || e.classList.remove('dragover');
      },
      !1
    ),
    e.addEventListener(
      'drop',
      t => {
        e.classList.remove('dragover');
        handleFileSelect(t.dataTransfer.files);
      },
      !1
    ));
  const a = document.getElementById('clearUploadBtn');
  a &&
    a.addEventListener('click', () => {
      isUploading
        ? showToast('正在上传中，请先取消上传', 'warning')
        : ((musicUploadQueue = []), updateUploadListUI());
    });
  const d = document.getElementById('uploadAllMusicBtn');
  d && d.addEventListener('click', startBatchUpload);
  const i = document.getElementById('cancelUploadBtn');
  i && i.addEventListener('click', cancelBatchUpload);
}

function handleFileSelect(e) {
  if (!e || 0 === e.length) return;
  const t = Array.from(e).filter(e => {
    const t = '.' + e.name.split('.').pop().toLowerCase(),
      n =
        [
          'audio/mpeg',
          'audio/mp3',
          'audio/wav',
          'audio/wave',
          'audio/ogg',
          'audio/x-m4a',
          'audio/mp4',
        ].includes(e.type) || ['.mp3', '.wav', '.ogg', '.m4a'].includes(t);
    return (n || showToast(`跳过不支持的文件: ${e.name}`, 'warning'), n);
  });
  0 !== t.length
    ? (t.forEach(e => {
        musicUploadQueue.some(t => t.file.name === e.name && t.file.size === e.size) ||
          musicUploadQueue.push({
            file: e,
            id: Date.now() + Math.random(),
            status: 'pending',
            progress: 0,
            error: null,
          });
      }),
      updateUploadListUI())
    : showToast('没有有效的音频文件', 'warning');
}

function updateUploadListUI() {
  const e = document.getElementById('musicUploadList'),
    t = document.getElementById('musicUploadItems');
  e &&
    t &&
    (0 !== musicUploadQueue.length
      ? ((e.style.display = 'block'),
        (t.innerHTML = musicUploadQueue
          .map(
            (e, t) =>
              `\n    <div class="upload-item" data-id="${e.id}">\n      <div class="upload-item-cover">\n        <div class="cover-preview ${e.coverFile ? 'has-cover' : ''}" onclick="triggerCoverSelect('${e.id}')">\n          ${e.coverFile ? `<img src="${e.coverPreview}" alt="封面">\n             <div class="cover-remove-btn" onclick="event.stopPropagation(); removeCover('${e.id}')">×</div>` : '<span class="cover-placeholder">🖼️</span>'}\n          <div class="cover-upload-hint">点击上传封面</div>\n        </div>\n        <input type="file" class="cover-input" id="coverInput-${e.id}" accept="image/jpeg,image/jpg,image/png,image/gif,image/webp" onchange="handleCoverSelect('${e.id}', this)">\n      </div>\n      <div class="upload-item-icon" style="display: none;">🎵</div>\n      <div class="upload-item-info">\n        <div class="upload-item-name">${e.file.name}</div>\n        <div class="upload-item-meta">\n          <span>${formatFileSize(e.file.size)}</span>\n          <span>${e.file.type || 'audio/*'}</span>\n        </div>\n      </div>\n      <div class="upload-item-progress">\n        <div class="progress-bar">\n          <div class="progress-fill" style="width: ${e.progress}%"></div>\n        </div>\n        <div class="progress-text">${e.progress}%</div>\n      </div>\n      <div class="upload-item-status ${e.status}">\n        ${getStatusText(e.status)}\n      </div>\n      <div class="upload-item-action">\n        <button class="remove-upload-btn" onclick="removeUploadItem('${e.id}')" ${'uploading' === e.status ? 'disabled' : ''}>×</button>\n      </div>\n    </div>\n  `
          )
          .join('')))
      : (e.style.display = 'none'));
}

function formatFileSize(e) {
  if (0 === e) return '0 B';
  const t = Math.floor(Math.log(e) / Math.log(1024));
  return Math.round((e / Math.pow(1024, t)) * 100) / 100 + ' ' + ['B', 'KB', 'MB', 'GB'][t];
}

function getStatusText(e) {
  return (
    {
      pending: '等待上传',
      uploading: '上传中',
      success: '上传成功',
      error: '上传失败',
    }[e] || e
  );
}

function removeUploadItem(e) {
  isUploading
    ? showToast('正在上传中，请先取消上传', 'warning')
    : ((musicUploadQueue = musicUploadQueue.filter(t => t.id != e)), updateUploadListUI());
}

function triggerCoverSelect(e) {
  const t = document.getElementById(`coverInput-${e}`);
  t && t.click();
}

function handleCoverSelect(e, t) {
  const n = t.files[0];
  if (!n) return;
  if (!['image/jpeg', 'image/jpg', 'image/png', 'image/gif', 'image/webp'].includes(n.type))
    return (showToast('请选择有效的图片文件（JPG、PNG、GIF、WebP）', 'error'), void (t.value = ''));
  if (n.size > 5242880)
    return (showToast('封面图片大小不能超过 5MB', 'error'), void (t.value = ''));
  const o = new FileReader();
  ((o.onload = function (t) {
    const o = musicUploadQueue.find(t => t.id == e);
    o && ((o.coverFile = n), (o.coverPreview = t.target.result), updateUploadListUI());
  }),
    o.readAsDataURL(n));
}

function removeCover(e) {
  const t = musicUploadQueue.find(t => t.id == e);
  if (t) {
    ((t.coverFile = null), (t.coverPreview = null));
    const n = document.getElementById(`coverInput-${e}`);
    (n && (n.value = ''), updateUploadListUI());
  }
}
async function startBatchUpload() {
  if (isUploading) return void showToast('正在上传中...', 'info');
  const e = musicUploadQueue.filter(e => 'pending' === e.status);
  if (0 === e.length) return void showToast('没有等待上传的文件', 'warning');
  ((isUploading = !0), (uploadAbortController = new AbortController()));
  const t = document.getElementById('uploadAllMusicBtn'),
    n = document.getElementById('cancelUploadBtn');
  (t && (t.disabled = !0), n && (n.disabled = !1));
  let o = 0,
    a = 0;
  for (const t of e) {
    if (!isUploading) break;
    ((t.status = 'uploading'), updateUploadListUI());
    try {
      (await uploadSingleMusicFile(t, uploadAbortController.signal),
        (t.status = 'success'),
        (t.progress = 100),
        o++);
    } catch (e) {
      ((t.status = 'error'),
        (t.error = e.message),
        a++,
        console.error(`上传文件 ${t.file.name} 失败:`, e));
    }
    updateUploadListUI();
  }
  ((isUploading = !1),
    (uploadAbortController = null),
    t && (t.disabled = !1),
    n && (n.disabled = !0),
    o > 0 && (showToast(`成功上传 ${o} 个文件`, 'success'), loadMusicPlaylist()),
    a > 0 && showToast(`${a} 个文件上传失败`, 'error'),
    setTimeout(() => {
      ((musicUploadQueue = musicUploadQueue.filter(e => 'success' !== e.status)),
        updateUploadListUI());
    }, 2e3));
}

function uploadSingleMusicFile(e, t) {
  return new Promise((n, o) => {
    const a = new FormData();
    a.append('file', e.file);
    const d = e.file.name.replace(/\.[^/.]+$/, '');
    (a.append('title', d),
      a.append('artist', '未知艺术家'),
      e.coverFile && a.append('cover', e.coverFile));
    const i = localStorage.getItem('auth_token');
    const r = new XMLHttpRequest();
    (r.upload.addEventListener('progress', t => {
      if (t.lengthComputable) {
        const n = Math.round((t.loaded / t.total) * 100);
        ((e.progress = n), updateUploadListUI());
      }
    }),
      r.addEventListener('load', () => {
        if (r.status >= 200 && r.status < 300) n();
        else
          try {
            const e = JSON.parse(r.responseText);
            o(new Error(e.error || '上传失败'));
          } catch (e) {
            o(new Error(`上传失败: ${r.status}`));
          }
      }),
      r.addEventListener('error', () => {
        o(new Error('网络错误'));
      }),
      r.addEventListener('abort', () => {
        o(new Error('上传已取消'));
      }),
      t.addEventListener('abort', () => {
        r.abort();
      }),
      r.open('POST', '/api/music/upload'),
      i && r.setRequestHeader('Authorization', `Bearer ${i}`),
      r.send(a));
  });
}

function cancelBatchUpload() {
  if (!isUploading) return;
  (uploadAbortController && uploadAbortController.abort(),
    (isUploading = !1),
    (uploadAbortController = null),
    musicUploadQueue.forEach(e => {
      'uploading' === e.status && ((e.status = 'pending'), (e.progress = 0));
    }),
    updateUploadListUI(),
    showToast('已取消上传', 'info'));
  const e = document.getElementById('uploadAllMusicBtn'),
    t = document.getElementById('cancelUploadBtn');
  (e && (e.disabled = !1), t && (t.disabled = !0));
}
async function uploadMusicFile() {
  const e = document.getElementById('musicFileUpload');
  e.files && 0 !== e.files.length
    ? (handleFileSelect(e.files), (e.value = ''))
    : showToast('请选择要上传的音乐文件', 'warning');
}
async function loadMusicPlaylist() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/music/playlist', {
      method: 'GET',
      headers: t,
    });
    if (n.ok) {
      updateMusicPlaylistUI(await n.json());
    } else console.error('加载播放列表失败');
  } catch (e) {
    console.error('加载播放列表失败:', e);
  }
}

function updateMusicPlaylistUI(e) {
  const t = document.getElementById('musicPlaylistContainer');
  t &&
    (e && 0 !== e.length
      ? (t.innerHTML = e
          .map((e, t) => {
            let n = e.title;
            const o = n.match(/^\d+_/);
            return (
              o && (n = n.substring(o[0].length)),
              `\n    <div style="display: flex; align-items: center; gap: 10px; padding: 10px; border-bottom: 1px solid rgba(0, 0, 0, 0.05);">\n      <div style="width: 50px; height: 50px; border-radius: 8px; overflow: hidden; background: rgba(0, 0, 0, 0.05); display: flex; align-items: center; justify-content: center;">\n        ${e.cover_image ? `<img src="${e.cover_image}" alt="${e.title}" style="width: 100%; height: 100%; object-fit: cover;">` : '<span style="font-size: 24px;">🎵</span>'}\n      </div>\n      <div style="flex: 1;">\n        <div style="font-weight: 500;">${n}</div>\n        <div style="font-size: 0.85em; color: #666;">${e.artist}</div>\n      </div>\n      <div style="font-size: 0.85em; color: #999;">${e.duration}</div>\n      <div style="display: flex; gap: 5px;">\n        <button onclick="editMusicTitle(${e.id}, '${n.replace(/'/g, "\\'")}')" style="background: #6c757d; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;">编辑标题</button>\n        <button onclick="changeMusicCover(${e.id})" style="background: #007bff; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;">更换封面</button>\n        <button onclick="deleteMusicTrack(${e.id})" style="background: #e74c3c; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;">删除</button>\n      </div>\n    </div>\n    `
            );
          })
          .join(''))
      : (t.innerHTML =
          '<div style="text-align: center; color: #999; padding: 20px;">暂无音乐文件</div>'));
}
async function deleteMusicTrack(e) {
  if (confirm('确定要删除这首音乐吗？'))
    try {
      const t = localStorage.getItem('auth_token'),
        n = {
          'Content-Type': 'application/json',
        };
      t && (n.Authorization = `Bearer ${t}`);
      const o = await fetch(`/api/music/${e}`, {
        method: 'DELETE',
        headers: n,
      });
      if (o.ok) (showToast('删除成功！', 'success'), loadMusicPlaylist());
      else {
        const e = await o.json();
        showToast('删除失败：' + (e.error || '未知错误'), 'error');
      }
    } catch (e) {
      (console.error('删除音乐失败:', e), showToast('删除失败，请稍后重试', 'error'));
    }
}
async function changeMusicCover(e) {
  const t = document.createElement('input');
  ((t.type = 'file'),
    (t.accept = 'image/jpeg,image/jpg,image/png,image/gif,image/webp'),
    (t.onchange = async t => {
      const n = t.target.files[0];
      if (!n) return;
      if (!['image/jpeg', 'image/jpg', 'image/png', 'image/gif', 'image/webp'].includes(n.type))
        return void showToast('请选择有效的图片文件（JPEG, PNG, GIF, WebP）', 'error');
      if (n.size > 5242880) return void showToast('图片大小不能超过 5MB', 'error');
      const o = new FormData();
      o.append('cover', n);
      try {
        const t = localStorage.getItem('auth_token'),
          n = {};
        t && (n.Authorization = `Bearer ${t}`);
        const a = await fetch(`/api/music/${e}/cover`, {
          method: 'POST',
          body: o,
          headers: n,
        });
        if (a.ok) {
          await a.json();
          (showToast('封面更新成功！', 'success'), loadMusicPlaylist());
        } else {
          const e = await a.json();
          showToast('封面更新失败：' + (e.message || '未知错误'), 'error');
        }
      } catch (e) {
        (console.error('更新封面失败:', e), showToast('更新封面失败，请稍后重试', 'error'));
      }
    }),
    t.click());
}

function editMusicTitle(e, t) {
  const n = document.createElement('div');
  ((n.style.cssText =
    '\n    position: fixed;\n    top: 0;\n    left: 0;\n    width: 100%;\n    height: 100%;\n    background: rgba(0, 0, 0, 0.5);\n    display: flex;\n    justify-content: center;\n    align-items: center;\n    z-index: 10000;\n  '),
    (n.innerHTML = `\n    <div style="background: white; padding: 30px; border-radius: 10px; width: 400px; max-width: 90%;">\n      <h3 style="margin: 0 0 20px 0;">编辑标题</h3>\n      <input type="text" id="musicTitleInput" value="${t}" \n             style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 5px; margin-bottom: 20px; box-sizing: border-box;">\n      <div style="display: flex; gap: 10px; justify-content: flex-end;">\n        <button id="cancelBtn" style="padding: 8px 20px; background: #6c757d; color: white; border: none; border-radius: 5px; cursor: pointer;">取消</button>\n        <button id="saveBtn" style="padding: 8px 20px; background: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer;">保存</button>\n      </div>\n    </div>\n  `),
    document.body.appendChild(n));
  const o = n.querySelector('#musicTitleInput'),
    a = n.querySelector('#cancelBtn'),
    d = n.querySelector('#saveBtn');
  ((a.onclick = () => {
    document.body.removeChild(n);
  }),
    (d.onclick = async () => {
      const t = o.value.trim();
      if (t)
        try {
          const o = localStorage.getItem('auth_token'),
            a = {
              'Content-Type': 'application/json',
            };
          o && (a.Authorization = `Bearer ${o}`);
          const d = await fetch(`/api/music/${e}?action=title`, {
            method: 'PUT',
            body: JSON.stringify({
              title: t,
            }),
            headers: a,
          });
          if (d.ok) {
            await d.json();
            (showToast('标题更新成功！', 'success'),
              document.body.removeChild(n),
              loadMusicPlaylist());
          } else {
            const e = await d.json();
            showToast('标题更新失败：' + (e.message || '未知错误'), 'error');
          }
        } catch (e) {
          (console.error('更新标题失败:', e), showToast('更新标题失败，请稍后重试', 'error'));
        }
      else showToast('标题不能为空', 'error');
    }),
    (n.onclick = e => {
      e.target === n && document.body.removeChild(n);
    }));
  const i = e => {
    'Escape' === e.key &&
      (document.body.removeChild(n), document.removeEventListener('keydown', i));
  };
  document.addEventListener('keydown', i);
}
let mainCards = [],
  subCards = [];
async function loadMainCards() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/about/main-cards/admin', {
      method: 'GET',
      headers: t,
    });
    n.ok
      ? ((mainCards = await n.json()), updateMainCardsTable(), updateMainCardFilter())
      : showToast('加载主卡片失败', 'error');
  } catch (e) {
    (console.error('加载主卡片失败:', e), showToast('加载主卡片失败', 'error'));
  }
}

function updateMainCardsTable() {
  const e = document.getElementById('mainCardsTableBody');
  e &&
    (0 !== mainCards.length
      ? (e.innerHTML = mainCards
          .map(
            e =>
              `\n    <tr>\n      <td>${e.sort_order}</td>\n      <td>${e.icon || ''}</td>\n      <td>${e.title}</td>\n      <td>${e.layout_type}</td>\n      <td>${getSubCardCount(e.id)}</td>\n      <td>\n        <span style="color: ${e.is_enabled ? '#28a745' : '#dc3545'}; font-weight: bold;">\n          ${e.is_enabled ? '✓ 启用' : '✕ 禁用'}\n        </span>\n      </td>\n      <td>\n        <button class="btn-secondary" onclick="editMainCard(${e.id})">编辑</button>\n        <button class="btn-secondary" onclick="toggleMainCardEnabled(${e.id}, ${e.is_enabled})">\n          ${e.is_enabled ? '禁用' : '启用'}\n        </button>\n        <button class="btn-danger" onclick="deleteMainCard(${e.id})">删除</button>\n      </td>\n    </tr>\n  `
          )
          .join(''))
      : (e.innerHTML =
          '<tr><td colspan="7" style="text-align: center; color: #999;">暂无主卡片</td></tr>'));
}

function getSubCardCount(e) {
  return subCards.filter(t => t.main_card_id === e).length;
}

function updateMainCardFilter() {
  const e = document.getElementById('subCardMainCardFilter');
  if (!e) return;
  const t = e.value;
  ((e.innerHTML =
    '<option value="">全部主卡片</option>' +
    mainCards.map(e => `<option value="${e.id}">${e.title}</option>`).join('')),
    (e.value = t));
}
async function loadSubCards() {
  try {
    const e = localStorage.getItem('auth_token'),
      t = {
        'Content-Type': 'application/json',
      };
    e && (t.Authorization = `Bearer ${e}`);
    const n = await fetch('/api/about/sub-cards/admin', {
      method: 'GET',
      headers: t,
    });
    n.ok
      ? ((subCards = await n.json()), updateSubCardsTable(), updateMainCardsTable())
      : showToast('加载次卡片失败', 'error');
  } catch (e) {
    (console.error('加载次卡片失败:', e), showToast('加载次卡片失败', 'error'));
  }
}

function updateSubCardsTable() {
  const e = document.getElementById('subCardsTableBody');
  if (!e) return;
  const t = document.getElementById('subCardMainCardFilter').value;
  let n = subCards;
  (t && (n = subCards.filter(e => e.main_card_id === parseInt(t))),
    0 !== n.length
      ? (e.innerHTML = n
          .map(e => {
            mainCards.find(t => t.id === e.main_card_id);
            return `\n      <tr>\n        <td>${e.sort_order}</td>\n        <td>${e.icon || ''}</td>\n        <td>${e.title}</td>\n        <td>${e.description.substring(0, 30)}${e.description.length > 30 ? '...' : ''}</td>\n        <td>${e.link_url || '-'}</td>\n        <td>\n          <span style="color: ${e.is_enabled ? '#28a745' : '#dc3545'}; font-weight: bold;">\n            ${e.is_enabled ? '✓ 启用' : '✕ 禁用'}\n          </span>\n        </td>\n        <td>\n          <button class="btn-secondary" onclick="editSubCard(${e.id})">编辑</button>\n          <button class="btn-secondary" onclick="toggleSubCardEnabled(${e.id}, ${e.is_enabled})">\n            ${e.is_enabled ? '禁用' : '启用'}\n          </button>\n          <button class="btn-danger" onclick="deleteSubCard(${e.id})">删除</button>\n        </td>\n      </tr>\n    `;
          })
          .join(''))
      : (e.innerHTML =
          '<tr><td colspan="7" style="text-align: center; color: #999;">暂无次卡片</td></tr>'));
}
async function addMainCard() {
  ((document.getElementById('mainCardModalTitle').textContent = '添加主卡片'),
    document.getElementById('mainCardForm').reset(),
    (document.getElementById('mainCardId').value = ''),
    (document.getElementById('mainCardSortOrder').value = mainCards.length + 1),
    (document.getElementById('mainCardEnabled').checked = !0),
    openModal('mainCardModal'));
}
async function editMainCard(e) {
  const t = mainCards.find(t => t.id === e);
  t &&
    ((document.getElementById('mainCardModalTitle').textContent = '编辑主卡片'),
    (document.getElementById('mainCardId').value = t.id),
    (document.getElementById('mainCardTitle').value = t.title || ''),
    (document.getElementById('mainCardIcon').value = t.icon || ''),
    (document.getElementById('mainCardLayoutType').value = t.layout_type || 'grid'),
    (document.getElementById('mainCardCustomCss').value = t.custom_css || ''),
    (document.getElementById('mainCardSortOrder').value = t.sort_order || 0),
    (document.getElementById('mainCardEnabled').checked = t.is_enabled),
    openModal('mainCardModal'));
}
async function deleteMainCard(e) {
  ((currentAction = 'delete-main-card'),
    (currentItemId = e),
    (document.getElementById('confirmMessage').textContent =
      '确定要删除这个主卡片吗？所有关联的次卡片也会被删除。此操作不可撤销。'),
    openModal('confirmModal'));
}
async function toggleMainCardEnabled(e, t) {
  try {
    const n = localStorage.getItem('auth_token');
    (
      await fetch(`/api/about/main-cards/enabled?id=${e}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${n}`,
        },
        body: JSON.stringify({
          enabled: !t,
        }),
      })
    ).ok
      ? loadMainCards()
      : showToast('操作失败', 'error');
  } catch (e) {
    (console.error('切换状态失败:', e), showToast('操作失败', 'error'));
  }
}
async function addSubCard() {
  const e = document.getElementById('subCardMainCardFilter').value;
  if (!e) return void showToast('请先选择一个主卡片', 'warning');
  ((document.getElementById('subCardModalTitle').textContent = '添加次卡片'),
    document.getElementById('subCardForm').reset(),
    (document.getElementById('subCardId').value = ''));
  const t = document.getElementById('subCardMainCardId');
  ((t.innerHTML =
    '<option value="">请选择主卡片</option>' +
    mainCards.map(e => `<option value="${e.id}">${e.title}</option>`).join('')),
    (t.value = e),
    (document.getElementById('subCardSortOrder').value =
      subCards.filter(t => t.main_card_id === parseInt(e)).length + 1),
    (document.getElementById('subCardEnabled').checked = !0),
    openModal('subCardModal'));
}
async function editSubCard(e) {
  const t = subCards.find(t => t.id === e);
  if (!t) return;
  ((document.getElementById('subCardModalTitle').textContent = '编辑次卡片'),
    (document.getElementById('subCardId').value = t.id));
  const n = document.getElementById('subCardMainCardId');
  ((n.innerHTML =
    '<option value="">请选择主卡片</option>' +
    mainCards.map(e => `<option value="${e.id}">${e.title}</option>`).join('')),
    (n.value = t.main_card_id),
    (document.getElementById('subCardTitle').value = t.title || ''),
    (document.getElementById('subCardDescription').value = t.description || ''),
    (document.getElementById('subCardIcon').value = t.icon || ''),
    (document.getElementById('subCardLinkUrl').value = t.link_url || ''),
    (document.getElementById('subCardCustomCss').value = t.custom_css || ''),
    (document.getElementById('subCardSortOrder').value = t.sort_order || 0),
    (document.getElementById('subCardEnabled').checked = t.is_enabled),
    openModal('subCardModal'));
}
async function deleteSubCard(e) {
  ((currentAction = 'delete-sub-card'),
    (currentItemId = e),
    (document.getElementById('confirmMessage').textContent =
      '确定要删除这个次卡片吗？此操作不可撤销。'),
    openModal('confirmModal'));
}
async function toggleSubCardEnabled(e, t) {
  try {
    const n = localStorage.getItem('auth_token');
    (
      await fetch(`/api/about/sub-cards/enabled?id=${e}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${n}`,
        },
        body: JSON.stringify({
          enabled: !t,
        }),
      })
    ).ok
      ? loadSubCards()
      : showToast('操作失败', 'error');
  } catch (e) {
    (console.error('切换状态失败:', e), showToast('操作失败', 'error'));
  }
}
document.addEventListener('DOMContentLoaded', function () {
  const e = document.getElementById('addMainCardBtn');
  e && e.addEventListener('click', addMainCard);
  const t = document.getElementById('addSubCardBtn');
  t && t.addEventListener('click', addSubCard);
  const n = document.getElementById('refreshAboutCardsBtn');
  n &&
    n.addEventListener('click', function () {
      (loadMainCards(), loadSubCards());
    });
  const o = document.getElementById('subCardMainCardFilter');
  o && o.addEventListener('change', updateSubCardsTable);
  const a = document.querySelector('[data-tab="about"]');
  (a &&
    a.addEventListener('click', function () {
      (loadMainCards(), loadSubCards(), loadFriendLinks());
    }),
    initializeFriendLinksManagement());
});
const mainCardForm = document.getElementById('mainCardForm');
mainCardForm &&
  mainCardForm.addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = document.getElementById('mainCardId').value,
      n = document.getElementById('mainCardTitle').value,
      o = document.getElementById('mainCardIcon').value,
      a = document.getElementById('mainCardLayoutType').value,
      d = document.getElementById('mainCardCustomCss').value,
      i = parseInt(document.getElementById('mainCardSortOrder').value),
      r = document.getElementById('mainCardEnabled').checked;
    try {
      const e = {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${localStorage.getItem('auth_token')}`,
      };
      let l = '/api/about/main-cards',
        s = 'POST';
      t && ((l = `/api/about/main-cards/update?id=${t}`), (s = 'PUT'));
      (
        await fetch(l, {
          method: s,
          headers: e,
          body: JSON.stringify({
            title: n,
            icon: o,
            layout_type: a,
            custom_css: d,
            sort_order: i,
            is_enabled: r,
          }),
        })
      ).ok
        ? (showToast(t ? '更新成功' : '添加成功', 'success'),
          closeModal('mainCardModal'),
          loadMainCards())
        : showToast('操作失败', 'error');
    } catch (e) {
      (console.error('操作失败:', e), showToast('操作失败', 'error'));
    }
  });
const subCardForm = document.getElementById('subCardForm');

function escapeHtml(e) {
  if (!e) return '';
  const t = document.createElement('div');
  return ((t.textContent = e), t.innerHTML);
}
subCardForm &&
  subCardForm.addEventListener('submit', async function (e) {
    e.preventDefault();
    const t = document.getElementById('subCardId').value,
      n = parseInt(document.getElementById('subCardMainCardId').value),
      o = document.getElementById('subCardTitle').value,
      a = document.getElementById('subCardDescription').value,
      d = document.getElementById('subCardIcon').value,
      i = document.getElementById('subCardLinkUrl').value,
      r = document.getElementById('subCardCustomCss').value,
      l = parseInt(document.getElementById('subCardSortOrder').value),
      s = document.getElementById('subCardEnabled').checked;
    try {
      const e = {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${localStorage.getItem('auth_token')}`,
      };
      let c = '/api/about/sub-cards',
        u = 'POST';
      t && ((c = `/api/about/sub-cards/update?id=${t}`), (u = 'PUT'));
      (
        await fetch(c, {
          method: u,
          headers: e,
          body: JSON.stringify({
            main_card_id: n,
            title: o,
            description: a,
            icon: d,
            link_url: i,
            custom_css: r,
            sort_order: l,
            is_enabled: s,
          }),
        })
      ).ok
        ? (showToast(t ? '更新成功' : '添加成功', 'success'),
          closeModal('subCardModal'),
          loadSubCards())
        : showToast('操作失败', 'error');
    } catch (e) {
      (console.error('操作失败:', e), showToast('操作失败', 'error'));
    }
  });
let friendLinks = [],
  selectedFriendLinkIds = [];
async function loadFriendLinks() {
  try {
    const e = {
        Authorization: `Bearer ${localStorage.getItem('auth_token')}`,
      },
      t = await fetch('/api/admin/friend-links?include_disabled=true', {
        headers: e,
      });
    if (t.ok) {
      const e = await t.json();
      e.success
        ? ((friendLinks = e.data), updateFriendLinksTable())
        : showToast(e.message || '加载友链列表失败', 'error');
    } else showToast('加载友链列表失败', 'error');
  } catch (e) {
    (console.error('加载友链列表失败:', e), showToast('加载友链列表失败', 'error'));
  }
}

function updateFriendLinksTable() {
  const e = document.getElementById('friendLinksTableBody');
  if (0 === friendLinks.length)
    return (
      (e.innerHTML =
        '\n      <tr>\n        <td colspan="7" style="text-align: center; padding: 40px;">\n          <div style="opacity: 0.5;">暂无友情链接</div>\n          <div style="margin-top: 10px;">\n            <button class="btn-primary" onclick="openAddFriendLinkModal()">添加第一个友链</button>\n          </div>\n        </td>\n      </tr>\n    '),
      void updateBatchDeleteButton()
    );
  ((e.innerHTML = friendLinks
    .map(
      e =>
        `\n    <tr style="${e.is_enabled ? '' : 'opacity: 0.6;'}">\n      <td>\n        <input type="checkbox" class="friend-link-checkbox" data-id="${e.id}"\n               style="width: auto;"\n               ${selectedFriendLinkIds.includes(e.id) ? 'checked' : ''}>\n      </td>\n      <td>\n        <img src="${e.avatar_url || '/img/avatar.webp'}" alt="${e.nickname}"\n             style="width: 40px; height: 40px; border-radius: 50%; object-fit: cover;"\n             onerror="this.src='/img/avatar.webp'">\n      </td>\n      <td><strong>${escapeHtml(e.nickname)}</strong></td>\n      <td>\n        <a href="${escapeHtml(e.link_url)}" target="_blank" rel="noopener noreferrer"\n           style="color: #007bff; max-width: 200px; display: inline-block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">\n          ${escapeHtml(e.link_url)}\n        </a>\n      </td>\n      <td style="max-width: 250px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">\n        ${escapeHtml(e.motto || '-')}\n      </td>\n      <td>\n        <span style="color: ${e.is_enabled ? '#28a745' : '#dc3545'}; font-weight: bold;">\n          ${e.is_enabled ? '✓ 启用' : '✕ 禁用'}\n        </span>\n      </td>\n      <td>\n        <button class="btn-secondary" onclick="toggleFriendLinkStatus(${e.id})" title="${e.is_enabled ? '禁用' : '启用'}">\n          ${e.is_enabled ? '禁用' : '启用'}\n        </button>\n        <button class="btn-primary" onclick="openEditFriendLinkModal(${e.id})" title="编辑">编辑</button>\n        <button class="btn-danger" onclick="deleteFriendLink(${e.id})" title="删除">删除</button>\n      </td>\n    </tr>\n  `
    )
    .join('')),
    document.querySelectorAll('.friend-link-checkbox').forEach(e => {
      e.addEventListener('change', function () {
        toggleFriendLinkSelection(parseInt(this.getAttribute('data-id')));
      });
    }),
    updateBatchDeleteButton(),
    updateSelectAllCheckbox());
}

function toggleFriendLinkSelection(e) {
  (console.log('toggleFriendLinkSelection called with id:', e),
    console.log('Current selectedFriendLinkIds:', selectedFriendLinkIds));
  const t = selectedFriendLinkIds.indexOf(e);
  (t > -1
    ? (selectedFriendLinkIds.splice(t, 1), console.log('Removed from selection'))
    : (selectedFriendLinkIds.push(e), console.log('Added to selection')),
    console.log('New selectedFriendLinkIds:', selectedFriendLinkIds),
    updateBatchDeleteButton(),
    updateSelectAllCheckbox());
}

function toggleSelectAllFriendLinks() {
  const e = document.getElementById('selectAllFriendLinks');
  ((selectedFriendLinkIds = e.checked ? friendLinks.map(e => e.id) : []),
    document.querySelectorAll('.friend-link-checkbox').forEach(t => {
      t.checked = e.checked;
    }),
    updateBatchDeleteButton());
}

function updateSelectAllCheckbox() {
  const e = document.getElementById('selectAllFriendLinks'),
    t = document.querySelectorAll('.friend-link-checkbox');
  if (0 === t.length) return ((e.checked = !1), void (e.disabled = !0));
  ((e.disabled = !1),
    (e.checked = t.length > 0 && selectedFriendLinkIds.length === friendLinks.length));
}

function updateBatchDeleteButton() {
  const e = document.getElementById('batchDeleteFriendLinksBtn'),
    t = document.getElementById('friendLinkSelectedCount');
  selectedFriendLinkIds.length > 0
    ? ((e.style.display = 'inline-flex'), (t.textContent = selectedFriendLinkIds.length))
    : (e.style.display = 'none');
}
async function batchDeleteFriendLinks() {
  if (0 !== selectedFriendLinkIds.length) {
    if (confirm(`确定要删除选中的 ${selectedFriendLinkIds.length} 个友链吗？\n此操作不可恢复！`))
      try {
        const e = localStorage.getItem('auth_token'),
          t = await fetch('/api/admin/friend-links/batch-delete', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              Authorization: `Bearer ${e}`,
            },
            body: JSON.stringify({
              ids: selectedFriendLinkIds,
            }),
          }),
          n = await t.json();
        n.success
          ? (showToast(n.message || '批量删除成功', 'success'),
            (selectedFriendLinkIds = []),
            updateBatchDeleteButton(),
            await loadFriendLinks())
          : showToast(n.message || '批量删除失败', 'error');
      } catch (e) {
        (console.error('批量删除失败:', e), showToast('批量删除失败，请重试', 'error'));
      }
  } else showToast('请先选择要删除的友链', 'warning');
}

function openAddFriendLinkModal() {
  ((document.getElementById('friendLinkModalTitle').textContent = '添加友情链接'),
    (document.getElementById('friendLinkId').value = ''),
    (document.getElementById('friendLinkNickname').value = ''),
    (document.getElementById('friendLinkUrl').value = ''),
    (document.getElementById('friendLinkAvatar').value = ''),
    (document.getElementById('friendLinkMotto').value = ''),
    (document.getElementById('friendLinkSortOrder').value = '0'),
    (document.getElementById('friendLinkEnabled').checked = !0),
    openModal('friendLinkModal'));
}

function openEditFriendLinkModal(e) {
  const t = friendLinks.find(t => t.id === e);
  t
    ? ((document.getElementById('friendLinkModalTitle').textContent = '编辑友情链接'),
      (document.getElementById('friendLinkId').value = t.id),
      (document.getElementById('friendLinkNickname').value = t.nickname),
      (document.getElementById('friendLinkUrl').value = t.link_url),
      (document.getElementById('friendLinkAvatar').value = t.avatar_url),
      (document.getElementById('friendLinkMotto').value = t.motto),
      (document.getElementById('friendLinkSortOrder').value = t.sort_order),
      (document.getElementById('friendLinkEnabled').checked = t.is_enabled),
      openModal('friendLinkModal'))
    : showToast('友链不存在', 'error');
}
async function saveFriendLink(e) {
  e.preventDefault();
  const t = document.getElementById('friendLinkId').value,
    n = !!t,
    o = {
      nickname: document.getElementById('friendLinkNickname').value.trim(),
      link_url: document.getElementById('friendLinkUrl').value.trim(),
      avatar_url: document.getElementById('friendLinkAvatar').value.trim(),
      motto: document.getElementById('friendLinkMotto').value.trim(),
      sort_order: parseInt(document.getElementById('friendLinkSortOrder').value) || 0,
      is_enabled: document.getElementById('friendLinkEnabled').checked,
    };
  if (o.nickname && o.link_url)
    try {
      const e = {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${localStorage.getItem('auth_token')}`,
        },
        a = n ? `/api/admin/friend-links/${t}` : '/api/admin/friend-links',
        d = n ? 'PUT' : 'POST',
        i = await fetch(a, {
          method: d,
          headers: e,
          body: JSON.stringify(o),
        }),
        r = await i.json();
      r.success
        ? (showToast(n ? '友链更新成功' : '友链添加成功', 'success'),
          closeModal('friendLinkModal'),
          await loadFriendLinks())
        : showToast(r.message || '操作失败', 'error');
    } catch (e) {
      (console.error('保存友链失败:', e), showToast('操作失败，请重试', 'error'));
    }
  else showToast('昵称和链接地址不能为空', 'error');
}
async function toggleFriendLinkStatus(e) {
  const t = friendLinks.find(t => t.id === e);
  if (t)
    try {
      const n = {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${localStorage.getItem('auth_token')}`,
        },
        o = await fetch('/api/admin/friend-links/batch-update-status', {
          method: 'POST',
          headers: n,
          body: JSON.stringify({
            ids: [e],
            is_enabled: !t.is_enabled,
          }),
        }),
        a = await o.json();
      a.success
        ? (showToast(a.message || '状态更新成功', 'success'), await loadFriendLinks())
        : showToast(a.message || '状态更新失败', 'error');
    } catch (e) {
      (console.error('切换状态失败:', e), showToast('操作失败，请重试', 'error'));
    }
}

function deleteFriendLink(e) {
  const t = friendLinks.find(t => t.id === e);
  if (!t) return;
  if (!confirm(`确定要删除友链 "${t.nickname}" 吗？\n此操作不可恢复！`)) return;
  const n = localStorage.getItem('auth_token');
  fetch(`/api/admin/friend-links/${e}`, {
    method: 'DELETE',
    headers: {
      Authorization: `Bearer ${n}`,
    },
  })
    .then(e => e.json())
    .then(e => {
      e.success
        ? (showToast('友链删除成功', 'success'), loadFriendLinks())
        : showToast(e.message || '删除失败', 'error');
    })
    .catch(e => {
      (console.error('删除友链失败:', e), showToast('删除失败，请重试', 'error'));
    });
}

function initializeFriendLinksManagement() {
  const e = document.getElementById('addFriendLinkBtn');
  e && e.addEventListener('click', openAddFriendLinkModal);
  const t = document.getElementById('refreshFriendLinksBtn');
  t && t.addEventListener('click', loadFriendLinks);
  const n = document.getElementById('selectAllFriendLinks');
  n && n.addEventListener('change', toggleSelectAllFriendLinks);
  const o = document.getElementById('batchDeleteFriendLinksBtn');
  o && o.addEventListener('click', batchDeleteFriendLinks);
  const a = document.getElementById('friendLinkForm');
  a && a.addEventListener('submit', saveFriendLink);
}
const FileManager = {
    currentPath: 'img',
    currentRoot: 'img',
    selectedFile: null,
    filesToUpload: [],
    getAuthHeader() {
      const e = localStorage.getItem('auth_token');
      return e ? `Bearer ${e}` : '';
    },
    init() {
      (this.bindEvents(), this.loadTree(), this.loadFiles());
    },
    bindEvents() {
      (document
        .getElementById('fmUploadBtn')
        .addEventListener('click', () => this.openUploadModal()),
        document
          .getElementById('fmCreateDirBtn')
          .addEventListener('click', () => this.openCreateDirModal()),
        document.getElementById('fmBackBtn').addEventListener('click', () => this.goBack()),
        document.addEventListener('click', e => {
          e.target.closest('.fm-context-menu') ||
            e.target.closest('.fm-file-item') ||
            this.hideContextMenu();
        }),
        document.addEventListener('keydown', e => {
          'Escape' === e.key && this.hideContextMenu();
        }));
    },
    async loadTree() {
      try {
        const e = ['img', 'markdown', 'attachments', 'music'],
          t = document.getElementById('fmTree');
        t.innerHTML = '';
        for (const n of e) {
          const e = this.createTreeItem(n, !0);
          (t.appendChild(e), await this.loadSubDirectories(n, e));
        }
      } catch (e) {
        console.error('加载目录树失败:', e);
      }
    },
    async loadSubDirectories(e, t) {
      try {
        const n = await fetch(`/api/files?path=${encodeURIComponent(e)}`, {
            headers: {
              Authorization: this.getAuthHeader(),
            },
          }),
          o = await n.json();
        if (o.success && o.data.files) {
          const n = o.data.files.filter(e => e.is_dir);
          if (n.length > 0) {
            const o = document.createElement('div');
            o.className = 'fm-tree-children';
            for (const t of n) {
              const n = `${e}/${t.name}`,
                a = this.createTreeItem(t.name, !1, n);
              (o.appendChild(a), await this.loadSubDirectories(n, a));
            }
            t.appendChild(o);
          }
        }
      } catch (e) {
        console.error('加载子目录失败:', e);
      }
    },
    createTreeItem(e, t, n = null) {
      const o = document.createElement('div');
      ((o.className = 'fm-tree-item'), t && e === this.currentRoot && o.classList.add('active'));
      const a = document.createElement('span');
      ((a.className = 'fm-tree-icon'),
        (a.innerHTML = t
          ? '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>'
          : '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path><path d="M12 11v6"></path><path d="M9 14l3 3 3-3"></path></svg>'),
        o.appendChild(a));
      const d = document.createElement('span');
      ((d.textContent = e), o.appendChild(d));
      const i = n || e;
      return (
        o.addEventListener('click', e => {
          (e.stopPropagation(),
            this.navigateTo(i),
            document.querySelectorAll('.fm-tree-item').forEach(e => e.classList.remove('active')),
            o.classList.add('active'));
        }),
        o
      );
    },
    async loadFiles() {
      try {
        const e = await fetch(`/api/files?path=${encodeURIComponent(this.currentPath)}`, {
            headers: {
              Authorization: this.getAuthHeader(),
            },
          }),
          t = await e.json();
        t.success
          ? (this.renderFiles(t.data.files),
            this.updateBreadcrumb(t.data.current_path),
            this.updateBackButton(t.data.parent_path),
            this.updateFileCount(t.data.files.length))
          : showToast(t.message, 'error');
      } catch (e) {
        (console.error('加载文件失败:', e), showToast('加载文件失败', 'error'));
      }
    },
    renderFiles(e) {
      const t = document.getElementById('fmFileList'),
        n = document.getElementById('fmEmptyState');
      if (0 === e.length) return ((t.innerHTML = ''), void (n.style.display = 'flex'));
      n.style.display = 'none';
      const o = [...e].sort((e, t) =>
        e.is_dir && !t.is_dir ? -1 : !e.is_dir && t.is_dir ? 1 : e.name.localeCompare(t.name)
      );
      ((t.innerHTML = o.map(e => this.createFileItem(e)).join('')),
        t.querySelectorAll('.fm-file-item').forEach(e => {
          (e.addEventListener('click', t => {
            t.stopPropagation();
            const n = e.dataset.path;
            'true' === e.dataset.isDir ? this.navigateTo(n) : this.openFile(n);
          }),
            e.addEventListener('contextmenu', t => {
              t.preventDefault();
              const n = e.dataset.path,
                o = 'true' === e.dataset.isDir;
              this.showContextMenu(t, n, o);
            }));
        }));
    },
    createFileItem(e) {
      let t = '📄';
      e.is_dir
        ? (t = '📁')
        : ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.bmp', '.svg'].includes(e.extension)
          ? (t = `<img src="/${e.path}" alt="${e.name}" onerror="this.parentElement.innerHTML='🖼️'">`)
          : '.md' === e.extension && (t = '📝');
      const n = this.formatFileSize(e.size);
      return `\n      <div class="fm-file-item" data-path="${e.path}" data-is-dir="${e.is_dir}">\n        <div class="fm-file-icon">${t}</div>\n        <div class="fm-file-name">${e.name}</div>\n        <div class="fm-file-meta">${e.is_dir ? '文件夹' : n}</div>\n      </div>\n    `;
    },
    formatFileSize(e) {
      if (0 === e) return '0 B';
      const t = Math.floor(Math.log(e) / Math.log(1024));
      return parseFloat((e / Math.pow(1024, t)).toFixed(2)) + ' ' + ['B', 'KB', 'MB', 'GB'][t];
    },
    updateBreadcrumb(e) {
      document.getElementById('fmBreadcrumb').textContent = e || '/';
    },
    updateBackButton(e) {
      document.getElementById('fmBackBtn').disabled = !e;
    },
    updateFileCount(e) {
      document.getElementById('fmInfo').textContent = `${e} 个项目`;
    },
    navigateTo(e) {
      ((this.currentPath = e), this.loadFiles());
    },
    goBack() {
      const e = this.getParentPath(this.currentPath);
      e && this.navigateTo(e);
    },
    getParentPath(e) {
      if (e === this.currentRoot) return null;
      const t = e.split('/');
      t.pop();
      return t.join('/') || this.currentRoot;
    },
    openFile(e) {
      const t = e.split('/').pop().split('.').pop().toLowerCase();
      if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(t)) {
        const t = `/${e}`;
        return void this.showImagePreview(t);
      }
      if ('md' === t) {
        let t = e;
        if (t.startsWith('/')) {
          const n = t.indexOf('/markdown/');
          if (-1 === n)
            return (
              console.error('无效的 Markdown 路径:', e),
              void this.showToast('无效的 Markdown 路径', 'error')
            );
          t = t.substring(n + 10);
        } else {
          if (!t.startsWith('markdown/'))
            return (
              console.error('无效的 Markdown 路径:', e),
              void this.showToast('无效的 Markdown 路径', 'error')
            );
          t = t.substring(9);
        }
        return t && '/' !== t && '' !== t.trim()
          ? void (window.MarkdownPreviewModal
              ? window.MarkdownPreviewModal.open(t)
              : console.error('MarkdownPreviewModal not available'))
          : (console.error('提取后的 Markdown 路径无效:', t),
            void this.showToast('无效的 Markdown 路径', 'error'));
      }
      this.downloadFile(e);
    },
    showImagePreview(e) {
      const t = document.createElement('div');
      t.style.cssText =
        '\n      position: fixed;\n      top: 0;\n      left: 0;\n      width: 100%;\n      height: 100%;\n      background: rgba(0, 0, 0, 0.9);\n      z-index: 10000;\n      display: flex;\n      align-items: center;\n      justify-content: center;\n      cursor: pointer;\n    ';
      const n = document.createElement('img');
      ((n.src = e),
        (n.style.cssText =
          '\n      max-width: 90%;\n      max-height: 90%;\n      object-fit: contain;\n      border-radius: 8px;\n    '));
      const o = document.createElement('div');
      ((o.textContent = '点击关闭'),
        (o.style.cssText =
          '\n      position: absolute;\n      bottom: 30px;\n      color: white;\n      font-size: 16px;\n      opacity: 0.7;\n    '),
        t.appendChild(n),
        t.appendChild(o),
        t.addEventListener('click', () => {
          document.body.removeChild(t);
        }));
      const a = e => {
        'Escape' === e.key &&
          (document.body.removeChild(t), document.removeEventListener('keydown', a));
      };
      (document.addEventListener('keydown', a), document.body.appendChild(t));
    },
    async downloadFile(e) {
      try {
        const t = await fetch(`/api/files/download?path=${encodeURIComponent(e)}`, {
          headers: {
            Authorization: this.getAuthHeader(),
          },
        });
        if (!t.ok) {
          const e = await t.json();
          return void showToast(e.message || '下载失败', 'error');
        }
        const n = e.split('/').pop(),
          o = await t.blob(),
          a = window.URL.createObjectURL(o),
          d = document.createElement('a');
        ((d.href = a),
          (d.download = n),
          document.body.appendChild(d),
          d.click(),
          document.body.removeChild(d),
          window.URL.revokeObjectURL(a));
      } catch (e) {
        (console.error('下载失败:', e), showToast('下载失败', 'error'));
      }
    },
    showContextMenu(e, t, n) {
      this.selectedFile = {
        path: t,
        isDir: n,
      };
      const o = document.querySelector('.fm-context-menu');
      o && o.remove();
      const a = document.createElement('div');
      ((a.className = 'fm-context-menu'),
        (a.style.left = e.pageX + 'px'),
        (a.style.top = e.pageY + 'px'));
      const d = t.split('/').pop().split('.').pop().toLowerCase();
      ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'].includes(d);
      ([
        {
          action: 'open',
          label: '打开',
          icon: '📂',
        },
        {
          action: 'download',
          label: '下载',
          icon: '⬇️',
          hide: n,
        },
        {
          action: 'rename',
          label: '重命名',
          icon: '✏️',
        },
        {
          action: 'delete',
          label: '删除',
          icon: '🗑️',
          danger: !0,
        },
      ].forEach(e => {
        if (e.hide) return;
        const t = document.createElement('div');
        ((t.className = 'fm-context-menu-item' + (e.danger ? ' danger' : '')),
          (t.innerHTML = `<span>${e.icon}</span>${e.label}`),
          t.addEventListener('click', () => {
            (this.handleContextAction(e.action), this.hideContextMenu());
          }),
          a.appendChild(t));
      }),
        document.body.appendChild(a),
        a.classList.add('active'));
    },
    hideContextMenu() {
      const e = document.querySelector('.fm-context-menu');
      e && e.remove();
    },
    handleContextAction(e) {
      if (this.selectedFile)
        switch (e) {
          case 'open':
            this.selectedFile.isDir
              ? this.navigateTo(this.selectedFile.path)
              : this.openFile(this.selectedFile.path);
            break;
          case 'download':
            this.downloadFile(this.selectedFile.path);
            break;
          case 'rename':
            this.openRenameModal();
            break;
          case 'delete':
            this.openDeleteModal();
        }
    },
    openUploadModal() {
      const e = document.createElement('input');
      ((e.type = 'file'),
        (e.multiple = !0),
        (e.style.display = 'none'),
        e.addEventListener('change', t => {
          const n = Array.from(t.target.files);
          (n.length > 0 && this.uploadFiles(n), e.remove());
        }),
        document.body.appendChild(e),
        e.click());
    },
    async uploadFiles(e) {
      for (const t of e)
        try {
          const e = new FormData();
          e.append('file', t);
          const n = await fetch(`/api/files?path=${encodeURIComponent(this.currentPath)}`, {
            method: 'POST',
            headers: {
              Authorization: this.getAuthHeader(),
            },
            body: e,
          });
          (await n.json()).success
            ? showToast(`成功上传 ${t.name}`, 'success')
            : showToast(`上传 ${t.name} 失败`, 'error');
        } catch (e) {
          (console.error('上传失败:', e), showToast(`上传 ${t.name} 失败`, 'error'));
        }
      (this.loadFiles(), this.loadTree());
    },
    openCreateDirModal() {
      const e = prompt('请输入文件夹名称:');
      e && e.trim() && this.createDirectory(e.trim());
    },
    async createDirectory(e) {
      try {
        const t = await fetch('/api/files/create-dir', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              Authorization: this.getAuthHeader(),
            },
            body: JSON.stringify({
              path: this.currentPath,
              dir_name: e,
            }),
          }),
          n = await t.json();
        n.success
          ? (showToast('文件夹创建成功', 'success'), this.loadFiles(), this.loadTree())
          : showToast(n.message, 'error');
      } catch (e) {
        (console.error('创建目录失败:', e), showToast('创建文件夹失败', 'error'));
      }
    },
    openRenameModal() {
      if (!this.selectedFile) return;
      const e = this.selectedFile.path.split('/').pop(),
        t = prompt('请输入新名称:', e);
      t && t.trim() && t.trim() !== e && this.renameFile(t.trim());
    },
    async renameFile(e) {
      if (this.selectedFile)
        try {
          const t = await fetch('/api/files', {
              method: 'PUT',
              headers: {
                'Content-Type': 'application/json',
                Authorization: this.getAuthHeader(),
              },
              body: JSON.stringify({
                old_path: this.selectedFile.path,
                new_name: e,
              }),
            }),
            n = await t.json();
          n.success
            ? (showToast('重命名成功', 'success'), this.loadFiles(), this.loadTree())
            : showToast(n.message, 'error');
        } catch (e) {
          (console.error('重命名失败:', e), showToast('重命名失败', 'error'));
        }
    },
    openDeleteModal() {
      if (!this.selectedFile) return;
      const e = this.selectedFile.path.split('/').pop();
      confirm(`确定要删除 "${e}" 吗？此操作不可恢复！`) && this.deleteFile();
    },
    async deleteFile() {
      if (this.selectedFile)
        try {
          const e = await fetch(`/api/files?path=${encodeURIComponent(this.selectedFile.path)}`, {
              method: 'DELETE',
              headers: {
                Authorization: this.getAuthHeader(),
              },
            }),
            t = await e.json();
          t.success
            ? (showToast('删除成功', 'success'), this.loadFiles(), this.loadTree())
            : showToast(t.message, 'error');
        } catch (e) {
          (console.error('删除失败:', e), showToast('删除失败', 'error'));
        }
    },
  },
  AttachmentUploader = {
    currentArticleId: null,
    init() {
      const e = document.getElementById('uploadAttachmentBtn');
      e && e.addEventListener('click', () => this.uploadAttachment());
    },
    async uploadAttachment() {
      const e = document.getElementById('attachmentFile'),
        t = document.getElementById('uploadAttachmentArticleId'),
        n = document.getElementById('uploadAttachmentProgress'),
        o = document.getElementById('uploadAttachmentResult'),
        a = document.getElementById('uploadAttachmentProgressBar'),
        d = document.getElementById('uploadAttachmentStatus'),
        i = document.getElementById('uploadAttachmentFileInfo');
      if (!e.files || 0 === e.files.length) return void showToast('请选择要上传的文件', 'error');
      const r = e.files[0];
      ((this.currentArticleId = t.dataset.articleId),
        (n.style.display = 'block'),
        (o.style.display = 'none'),
        (a.style.width = '0%'),
        (d.textContent = '正在上传...'));
      const l = new FormData();
      (l.append('file', r), l.append('passage_id', this.currentArticleId));
      try {
        const e = localStorage.getItem('auth_token'),
          t = await fetch('/api/admin/attachments', {
            method: 'POST',
            headers: e
              ? {
                  Authorization: `Bearer ${e}`,
                }
              : {},
            body: l,
          }),
          n = await t.json();
        ((a.style.width = '100%'),
          n.success
            ? ((d.textContent = '上传成功！'),
              (o.style.display = 'block'),
              (i.innerHTML = `\n          <div><strong>文件名：</strong>${n.data.fileName}</div>\n          <div><strong>文件大小：</strong>${this.formatFileSize(n.data.size)}</div>\n          <div><strong>文件类型：</strong>${n.data.type}</div>\n          <div><strong>访问URL：</strong><a href="${n.data.url}" target="_blank" style="color: #007bff;">${n.data.url}</a></div>\n        `),
              showToast('附件上传成功！', 'success'),
              setTimeout(() => {
                closeModal('uploadAttachmentModal');
              }, 3e3))
            : ((d.textContent = '上传失败：' + (n.message || '未知错误')),
              showToast('附件上传失败：' + (n.message || '未知错误'), 'error')));
      } catch (e) {
        (console.error('上传附件失败:', e),
          (a.style.width = '100%'),
          (d.textContent = '上传失败：网络错误'),
          showToast('附件上传失败，请稍后重试', 'error'));
      }
    },
    formatFileSize(e) {
      if (0 === e) return '0 Bytes';
      const t = Math.floor(Math.log(e) / Math.log(1024));
      return Math.round((e / Math.pow(1024, t)) * 100) / 100 + ' ' + ['Bytes', 'KB', 'MB', 'GB'][t];
    },
  };
document.addEventListener('DOMContentLoaded', () => {
  AttachmentUploader.init();
  const e = document.querySelector('[data-tab="filemanager"]');
  e &&
    e.addEventListener('click', () => {
      setTimeout(() => {
        FileManager.initialized || (FileManager.init(), (FileManager.initialized = !0));
      }, 100);
    });
});
