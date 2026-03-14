async function loadAboutCards() {
    var e = document.getElementById("aboutContainer");
    try {
        var t = await fetch("/api/about/main-cards");
        if (!t.ok) throw new Error("获取卡片数据失败");
        var a = await t.json();
        e.innerHTML = "";
        for (const n of a) {
            var s = await renderMainCard(n);
            e.appendChild(s)
        }
        document.querySelectorAll(".about-card, .feature-item, .team-member, .contact-item").forEach((t, e) => {
            t.style.opacity = "0", t.style.transform = "translateY(20px)", t.style.transition = "opacity 0.6s ease, transform 0.6s ease", setTimeout(() => {
                t.style.opacity = "1", t.style.transform = "translateY(0)"
            }, 200 + 100 * e)
        })
    } catch (t) {
        console.error("加载卡片失败:", t), e.innerHTML = '<div class="about-card"><p>加载失败，请刷新页面重试</p></div>'
    }
}
async function renderMainCard(t) {
    var e = document.createElement("div");
    if (e.className = "about-card", t.custom_css) {
        const a = document.createElement("style");
        a.textContent = t.custom_css, e.appendChild(a)
    }
    const a = t.icon ? `<h2>${t.icon} ${t.title}</h2>` : `<h2>${t.title}</h2>`;
    e.innerHTML = a;
    try {
        const a = await fetch("/api/about/sub-cards?main_card_id=" + t.id);
        if (a.ok) {
            var s = await a.json();
            if (0 < s.length) {
                const a = renderSubCards(s, t.layout_type);
                e.innerHTML += a
            }
        }
    } catch (t) {
        console.error("加载次卡片失败:", t)
    }
    return e
}

function renderSubCards(t, a) {
    if (0 === t.length) return "";
    let e = "";
    switch (a) {
        case "grid":
            e = "features-grid";
            break;
        case "flex":
            e = "contact-info";
            break;
        default:
            e = "team-grid"
    }
    let s = `<div class="${e}">`;
    return t.forEach(t => {
        let e = "";
        t.custom_css && (e = ` style="${t.custom_css}"`), s += "flex" === a ? `
        <div class="contact-item"${e}>
          <div class="contact-icon">${t.icon||"📌"}</div>
          <div class="contact-details">
            <h3>${t.title}</h3>
            <p>${t.description}</p>
          </div>
        </div>
      ` : "grid" === a ? `
        <div class="feature-item"${e}>
          <h3><i>${t.icon||"⭐"}</i> ${t.title}</h3>
          <p>${t.description}</p>
        </div>
      ` : `
        <div class="team-member"${e}>
          <div class="member-avatar">${t.icon||"👤"}</div>
          <h3>${t.title}</h3>
          <p>${t.description}</p>
        </div>
      `
    }), s += "</div>"
}
document.addEventListener("DOMContentLoaded", function() {
    loadAboutCards()
});
let lastScrollTop = 0,
    isNavHidden = !1;
const nav = document.getElementById("mainNav"),
    scrollIndicator = document.getElementById("scrollIndicator"),
    scrollProgress = document.getElementById("scrollProgress"),
    mainTitle = (nav.classList.add("scrolled-top"), window.addEventListener("scroll", function() {
        var t = window.pageYOffset || document.documentElement.scrollTop,
            e = t / (document.documentElement.scrollHeight - window.innerHeight) * 100;
        100 < t ? (scrollIndicator.classList.add("active"), scrollProgress.style.height = e + "%") : scrollIndicator.classList.remove("active"), t > lastScrollTop && 100 < t ? isNavHidden || (nav.classList.add("hidden"), isNavHidden = !0) : (isNavHidden && (nav.classList.remove("hidden"), isNavHidden = !1), 0 === t ? (nav.classList.add("scrolled-top"), nav.classList.remove("scrolled")) : (nav.classList.remove("scrolled-top"), nav.classList.add("scrolled"))), lastScrollTop = t
    }, {
        passive: !0
    }), document.querySelectorAll('nav a[href^="#"]').forEach(t => {
        t.addEventListener("click", function(t) {
            var e = this.getAttribute("href");
            e.startsWith("#") && (t.preventDefault(), t = document.querySelector(e)) && window.scrollTo({
                top: t.offsetTop - 100,
                behavior: "smooth"
            })
        })
    }), window.addEventListener("load", function() {
        document.body.style.opacity = "0", document.body.style.transition = "opacity 0.5s ease", setTimeout(() => {
            document.body.style.opacity = "1"
        }, 100), document.querySelectorAll(".about-card, .feature-item, .team-member, .contact-item").forEach((t, e) => {
            t.style.opacity = "0", t.style.transform = "translateY(20px)", t.style.transition = "opacity 0.6s ease, transform 0.6s ease", setTimeout(() => {
                t.style.opacity = "1", t.style.transform = "translateY(0)"
            }, 200 + 100 * e)
        })
    }), document.getElementById("main-title"));
mainTitle && (mainTitle.addEventListener("mouseenter", function() {
    this.style.animationPlayState = "paused"
}), mainTitle.addEventListener("mouseleave", function() {
    this.style.animationPlayState = "running"
}));
