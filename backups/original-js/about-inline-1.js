async function loadAboutCards() {
      const t = document.getElementById('aboutContainer');
      try {
        const e = await fetch('/api/about/main-cards');
        if (!e.ok) throw new Error('获取卡片数据失败');
        const n = await e.json();
        t.innerHTML = '';
        for (const e of n) {
          const n = await renderMainCard(e);
          t.appendChild(n);
        }
        document
          .querySelectorAll('.about-card, .feature-item, .team-member, .contact-item')
          .forEach((t, e) => {
            ((t.style.opacity = '0'),
              (t.style.transform = 'translateY(20px)'),
              (t.style.transition = 'opacity 0.6s ease, transform 0.6s ease'),
              setTimeout(
                () => {
                  ((t.style.opacity = '1'), (t.style.transform = 'translateY(0)'));
                },
                200 + 100 * e
              ));
          });
      } catch (e) {
        (console.error('加载卡片失败:', e),
          (t.innerHTML = '<div class="about-card"><p>加载失败，请刷新页面重试</p></div>'));
      }
    }
    async function renderMainCard(t) {
      const e = document.createElement('div');
      if (((e.className = 'about-card'), t.custom_css)) {
        const n = document.createElement('style');
        ((n.textContent = t.custom_css), e.appendChild(n));
      }
      const n = t.icon ? `<h2>${t.icon} ${t.title}</h2>` : `<h2>${t.title}</h2>`;
      e.innerHTML = n;
      try {
        const n = await fetch(`/api/about/sub-cards?main_card_id=${t.id}`);
        if (n.ok) {
          const s = await n.json();
          if (s.length > 0) {
            const n = renderSubCards(s, t.layout_type);
            e.innerHTML += n;
          }
        }
      } catch (t) {
        console.error('加载次卡片失败:', t);
      }
      return e;
    }
    function renderSubCards(t, e) {
      if (0 === t.length) return '';
      let n = '';
      switch (e) {
        case 'grid':
          n = 'features-grid';
          break;
        case 'flex':
          n = 'contact-info';
          break;
        default:
          n = 'team-grid';
      }
      let s = `<div class="${n}">`;
      return (
        t.forEach(t => {
          let n = '';
          (t.custom_css && (n = ` style="${t.custom_css}"`),
            (s +=
              'flex' === e
                ? `\n        <div class="contact-item"${n}>\n          <div class="contact-icon">${t.icon || '📌'}</div>\n          <div class="contact-details">\n            <h3>${t.title}</h3>\n            <p>${t.description}</p>\n          </div>\n        </div>\n      `
                : 'grid' === e
                  ? `\n        <div class="feature-item"${n}>\n          <h3><i>${t.icon || '⭐'}</i> ${t.title}</h3>\n          <p>${t.description}</p>\n        </div>\n      `
                  : `\n        <div class="team-member"${n}>\n          <div class="member-avatar">${t.icon || '👤'}</div>\n          <h3>${t.title}</h3>\n          <p>${t.description}</p>\n        </div>\n      `));
        }),
        (s += '</div>'),
        s
      );
    }
    document.addEventListener('DOMContentLoaded', function () {
      loadAboutCards();

      // 响应式加载背景图片
      const desktopBg = '{{ settings.background_image | safe }}';
      const mobileBg = '{{ settings.mobile_background_image | safe }}';

      function applyBackgroundImage() {
        const isMobile = window.innerWidth <= 768;
        const backgroundImage = isMobile && mobileBg ? mobileBg : desktopBg;
        if (backgroundImage) {
          document.body.style.backgroundImage = `url('${backgroundImage}')`;
        }
      }

      // 初始应用
      applyBackgroundImage();

      // 监听窗口大小变化
      window.addEventListener('resize', applyBackgroundImage);
    });
    let lastScrollTop = 0,
      isNavHidden = !1;
    const nav = document.getElementById('mainNav'),
      scrollIndicator = document.getElementById('scrollIndicator'),
      scrollProgress = document.getElementById('scrollProgress');
    (nav.classList.add('scrolled-top'),
      window.addEventListener(
        'scroll',
        function () {
          const t = window.pageYOffset || document.documentElement.scrollTop,
            e = (t / (document.documentElement.scrollHeight - window.innerHeight)) * 100;
          (t > 100
            ? (scrollIndicator.classList.add('active'), (scrollProgress.style.height = `${e}%`))
            : scrollIndicator.classList.remove('active'),
            t > lastScrollTop && t > 100
              ? isNavHidden || (nav.classList.add('hidden'), (isNavHidden = !0))
              : (isNavHidden && (nav.classList.remove('hidden'), (isNavHidden = !1)),
                0 === t
                  ? (nav.classList.add('scrolled-top'), nav.classList.remove('scrolled'))
                  : (nav.classList.remove('scrolled-top'), nav.classList.add('scrolled'))),
            (lastScrollTop = t));
        },
        { passive: !0 }
      ),
      document.querySelectorAll('nav a[href^="#"]').forEach(t => {
        t.addEventListener('click', function (t) {
          const e = this.getAttribute('href');
          if (e.startsWith('#')) {
            t.preventDefault();
            const n = document.querySelector(e);
            n && window.scrollTo({ top: n.offsetTop - 100, behavior: 'smooth' });
          }
        });
      }),
      window.addEventListener('load', function () {
        ((document.body.style.opacity = '0'),
          (document.body.style.transition = 'opacity 0.5s ease'),
          setTimeout(() => {
            document.body.style.opacity = '1';
          }, 100));
        document
          .querySelectorAll('.about-card, .feature-item, .team-member, .contact-item')
          .forEach((t, e) => {
            ((t.style.opacity = '0'),
              (t.style.transform = 'translateY(20px)'),
              (t.style.transition = 'opacity 0.6s ease, transform 0.6s ease'),
              setTimeout(
                () => {
                  ((t.style.opacity = '1'), (t.style.transform = 'translateY(0)'));
                },
                200 + 100 * e
              ));
          });
      }));
    const mainTitle = document.getElementById('main-title');
    mainTitle &&
      (mainTitle.addEventListener('mouseenter', function () {
        this.style.animationPlayState = 'paused';
      }),
      mainTitle.addEventListener('mouseleave', function () {
        this.style.animationPlayState = 'running';
      }));