/* ESBuild compressed */
var m=(e,a,s)=>new Promise((n,t)=>{var o=i=>{try{l(s.next(i))}catch(d){t(d)}},f=i=>{try{l(s.throw(i))}catch(d){t(d)}},l=i=>i.done?n(i.value):Promise.resolve(i.value).then(o,f);l((s=s.apply(e,a)).next())});function p(){return m(this,null,function*(){var e=document.getElementById("aboutContainer");try{var a=yield fetch("/api/about/main-cards");if(!a.ok)throw new Error("\u83B7\u53D6\u5361\u7247\u6570\u636E\u5931\u8D25");var s=yield a.json();e.innerHTML="";for(const t of s){var n=yield h(t);e.appendChild(n)}document.querySelectorAll(".about-card, .feature-item, .team-member, .contact-item").forEach((t,o)=>{t.style.opacity="0",t.style.transform="translateY(20px)",t.style.transition="opacity 0.6s ease, transform 0.6s ease",setTimeout(()=>{t.style.opacity="1",t.style.transform="translateY(0)"},200+100*o)})}catch(t){console.error("\u52A0\u8F7D\u5361\u7247\u5931\u8D25:",t),e.innerHTML='<div class="about-card"><p>\u52A0\u8F7D\u5931\u8D25\uFF0C\u8BF7\u5237\u65B0\u9875\u9762\u91CD\u8BD5</p></div>'}})}function h(e){return m(this,null,function*(){var a=document.createElement("div");if(a.className="about-card",e.custom_css){const t=document.createElement("style");t.textContent=e.custom_css,a.appendChild(t)}const s=e.icon?`<h2>${e.icon} ${e.title}</h2>`:`<h2>${e.title}</h2>`;a.innerHTML=s;try{const t=yield fetch("/api/about/sub-cards?main_card_id="+e.id);if(t.ok){var n=yield t.json();if(0<n.length){const o=b(n,e.layout_type);a.innerHTML+=o}}}catch(t){console.error("\u52A0\u8F7D\u6B21\u5361\u7247\u5931\u8D25:",t)}return a})}function b(e,a){if(e.length===0)return"";let s="";switch(a){case"grid":s="features-grid";break;case"flex":s="contact-info";break;default:s="team-grid"}let n=`<div class="${s}">`;return e.forEach(t=>{let o="";t.custom_css&&(o=` style="${t.custom_css}"`),n+=a==="flex"?`
        <div class="contact-item"${o}>
          <div class="contact-icon">${t.icon||"\u{1F4CC}"}</div>
          <div class="contact-details">
            <h3>${t.title}</h3>
            <p>${t.description}</p>
          </div>
        </div>
      `:a==="grid"?`
        <div class="feature-item"${o}>
          <h3><i>${t.icon||"\u2B50"}</i> ${t.title}</h3>
          <p>${t.description}</p>
        </div>
      `:`
        <div class="team-member"${o}>
          <div class="member-avatar">${t.icon||"\u{1F464}"}</div>
          <h3>${t.title}</h3>
          <p>${t.description}</p>
        </div>
      `}),n+="</div>"}document.addEventListener("DOMContentLoaded",function(){p()});let y=0,r=!1;const c=document.getElementById("mainNav"),v=document.getElementById("scrollIndicator"),E=document.getElementById("scrollProgress"),u=(c.classList.add("scrolled-top"),window.addEventListener("scroll",function(){var e=window.pageYOffset||document.documentElement.scrollTop,a=e/(document.documentElement.scrollHeight-window.innerHeight)*100;100<e?(v.classList.add("active"),E.style.height=a+"%"):v.classList.remove("active"),e>y&&100<e?r||(c.classList.add("hidden"),r=!0):(r&&(c.classList.remove("hidden"),r=!1),e===0?(c.classList.add("scrolled-top"),c.classList.remove("scrolled")):(c.classList.remove("scrolled-top"),c.classList.add("scrolled"))),y=e},{passive:!0}),document.querySelectorAll('nav a[href^="#"]').forEach(e=>{e.addEventListener("click",function(a){var s=this.getAttribute("href");s.startsWith("#")&&(a.preventDefault(),a=document.querySelector(s))&&window.scrollTo({top:a.offsetTop-100,behavior:"smooth"})})}),window.addEventListener("load",function(){document.body.style.opacity="0",document.body.style.transition="opacity 0.5s ease",setTimeout(()=>{document.body.style.opacity="1"},100),document.querySelectorAll(".about-card, .feature-item, .team-member, .contact-item").forEach((e,a)=>{e.style.opacity="0",e.style.transform="translateY(20px)",e.style.transition="opacity 0.6s ease, transform 0.6s ease",setTimeout(()=>{e.style.opacity="1",e.style.transform="translateY(0)"},200+100*a)})}),document.getElementById("main-title"));u&&(u.addEventListener("mouseenter",function(){this.style.animationPlayState="paused"}),u.addEventListener("mouseleave",function(){this.style.animationPlayState="running"}));
