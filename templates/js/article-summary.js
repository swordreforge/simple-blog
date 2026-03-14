/* ESBuild compressed */
var d=(a,l,n)=>new Promise((c,o)=>{var t=r=>{try{s(n.next(r))}catch(i){o(i)}},e=r=>{try{s(n.throw(r))}catch(i){o(i)}},s=r=>r.done?c(r.value):Promise.resolve(r.value).then(t,e);s((n=n.apply(a,l)).next())});(function(){"use strict";let a=!0,l=!1;function n(){return d(this,null,function*(){if(!l){try{const t=yield fetch("/api/settings/template");t.ok&&(a=(yield t.json()).passage_summarize_enabled!==!1)}catch(t){a=!0}l=!0}})}function c(t,e=null){if(t.hasAttribute("data-summary-processed"))return;const s=t.querySelector(".article-header"),r=t.querySelector(".article-content"),i=t.querySelector(".article-title");if(!s||!r||(t.setAttribute("data-summary-processed","true"),!a))return;let m=null;if(e&&e.summary&&(m=e.summary),!m)return;const u=o(m);s.insertAdjacentElement("afterend",u)}function o(t){const e=document.createElement("div");e.className="article-summary",e.innerHTML=`
      <div class="summary-content">
        <div class="summary-header">
          <span class="summary-label">\u6458\u8981</span>
          <button class="summary-toggle" title="\u5C55\u5F00/\u6536\u8D77\u6458\u8981">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </button>
        </div>
        <div class="summary-text">${t}</div>
      </div>
    `;const s=e.querySelector(".summary-toggle");return e.classList.add("expanded"),s.addEventListener("click",()=>{e.classList.toggle("expanded"),e.classList.contains("expanded")?s.style.transform="rotate(0deg)":s.style.transform="rotate(180deg)"}),e}window.ArticleSummary={init:n,processArticle:c,isEnabled:()=>a},document.readyState==="loading"?document.addEventListener("DOMContentLoaded",n):n()})();
