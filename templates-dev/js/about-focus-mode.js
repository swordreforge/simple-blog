function d(t) {
  (document.body.classList.remove('about-focus-mode'), e(), f('聚焦模式已暂停 (按 i 重新进入)'));
  const o = e => {
    'i' === e.key && (e.preventDefault(), document.removeEventListener('keydown', o), h(t));
  };
  document.addEventListener('keydown', o);
}
function k(t) {
  var o;
  (e(),
    t.items[t.selectedIndex] &&
      ((o = t.items[t.selectedIndex]).element.classList.add('about-focus-selected'),
      o.element.scrollIntoView({ behavior: 'smooth', block: 'center' })));
}
function h(e) {
  ((e.focusMode = !0),
    (e.selectedIndex = 0),
    document.body.classList.add('about-focus-mode'),
    l(e),
    k(e),
    f('已进入关于页面聚焦模式 (按 q 退出，上下键导航)'));
}
function l(e) {
  ((e.items = []),
    document.querySelectorAll('.about-card').forEach(t => {
      let o;
      e.items.push({
        element: t,
        type: 'main-card',
        title: (null == (o = t.querySelector('h2')) ? void 0 : o.textContent) || '卡片',
      });
    }),
    document.querySelectorAll('.feature-item').forEach(t => {
      let o;
      e.items.push({
        element: t,
        type: 'feature',
        title: (null == (o = t.querySelector('h3')) ? void 0 : o.textContent) || '特性',
      });
    }),
    document.querySelectorAll('.team-member').forEach(t => {
      let o;
      e.items.push({
        element: t,
        type: 'team',
        title: (null == (o = t.querySelector('h3')) ? void 0 : o.textContent) || '团队成员',
      });
    }),
    document.querySelectorAll('.contact-item').forEach(t => {
      let o;
      e.items.push({
        element: t,
        type: 'contact',
        title: (null == (o = t.querySelector('h3')) ? void 0 : o.textContent) || '联系方式',
      });
    }));
}
function f(e) {
  const t = document.createElement('div');
  ((t.className = 'about-focus-toast'),
    (t.textContent = e),
    document.body.appendChild(t),
    setTimeout(() => {
      t.classList.add('show');
    }, 10),
    setTimeout(() => {
      (t.classList.remove('show'), setTimeout(() => t.remove(), 300));
    }, 2e3));
}
function e() {
  document.querySelectorAll('.about-focus-selected').forEach(e => {
    e.classList.remove('about-focus-selected');
  });
}
class m {
  constructor() {
    ((this.focusMode = !1), (this.selectedIndex = 0), (this.items = []), this.init());
  }
  init() {
    document.addEventListener('keydown', t => {
      var o = document.activeElement;
      if (!o || ('INPUT' !== o.tagName && 'TEXTAREA' !== o.tagName && !o.isContentEditable))
        if ('i' !== t.key || this.focusMode) {
          if ('q' === t.key && this.focusMode)
            (t.preventDefault(),
              (this.focusMode = !1),
              document.body.classList.remove('about-focus-mode'),
              e(),
              f('已退出关于页面聚焦模式'));
          else if (this.focusMode)
            if ('Escape' === t.key) (t.preventDefault(), d(this));
            else if ('ArrowDown' === t.key)
              (t.preventDefault(),
                this.selectedIndex < this.items.length - 1 && (this.selectedIndex++, k(this)));
            else if ('ArrowUp' === t.key)
              (t.preventDefault(), 0 < this.selectedIndex && (this.selectedIndex--, k(this)));
            else if ('Enter' === t.key) {
              var s;
              (t.preventDefault(),
                this.items[this.selectedIndex] &&
                  ((s = this.items[this.selectedIndex]).element.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start',
                  }),
                  f('查看: ' + s.title)));
            }
        } else (t.preventDefault(), h(this));
    });
  }
}
'loading' === document.readyState
  ? document.addEventListener('DOMContentLoaded', () => {
      window.g = new m();
    })
  : (window.g = new m());
