/* ESBuild compressed */
var x=(t,a,n)=>new Promise((s,i)=>{var p=o=>{try{d(n.next(o))}catch(r){i(r)}},c=o=>{try{d(n.throw(o))}catch(r){i(r)}},d=o=>o.done?s(o.value):Promise.resolve(o.value).then(p,c);d((n=n.apply(t,a)).next())});(function(){let t=null,a=null,n=null,s=null,i=null,p=!1;function c(){var o,r;t||((t=document.createElement("div")).id="markdown-preview-modal",t.className="markdown-preview-modal",t.style.cssText=`
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(5px);
      z-index: 10000;
      display: none;
      align-items: center;
      justify-content: center;
      opacity: 0;
      transition: opacity 0.3s ease;
    `,(a=document.createElement("div")).className="markdown-preview-content",a.style.cssText=`
      background: white;
      width: 90%;
      max-width: 800px;
      max-height: 80vh;
      border-radius: 12px;
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
      overflow: hidden;
      transform: scale(0.9);
      transition: transform 0.3s ease;
      display: flex;
      flex-direction: column;
    `,(o=document.createElement("div")).className="markdown-preview-header",o.style.cssText=`
      padding: 16px 20px;
      border-bottom: 1px solid #e0e0e0;
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: #f5f5f5;
    `,(s=document.createElement("h3")).className="markdown-preview-title",s.style.cssText=`
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      color: #333;
    `,(n=document.createElement("button")).className="markdown-preview-close",n.innerHTML="\xD7",n.style.cssText=`
      background: none;
      border: none;
      font-size: 24px;
      color: #666;
      cursor: pointer;
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 50%;
      transition: all 0.2s ease;
    `,n.addEventListener("mouseenter",()=>{n.style.background="#e0e0e0",n.style.color="#333"}),n.addEventListener("mouseleave",()=>{n.style.background="none",n.style.color="#666"}),o.appendChild(s),o.appendChild(n),(i=document.createElement("div")).className="markdown-preview-body",i.style.cssText=`
      padding: 20px;
      overflow-y: auto;
      flex: 1;
      font-family: 'Segoe UI', 'Helvetica Neue', 'PingFang SC', 'Microsoft YaHei', sans-serif;
      line-height: 1.6;
      color: #333;
    `,(r=document.createElement("div")).className="markdown-preview-loading",r.innerHTML=`
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 200px;
        color: #666;
      ">
        <div style="
          width: 40px;
          height: 40px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #3498db;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 12px;
        "></div>
        <div style="font-size: 14px;">\u52A0\u8F7D\u4E2D...</div>
      </div>
      <style>
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      </style>
    `,a.appendChild(o),a.appendChild(i),t.appendChild(a),document.body.appendChild(t),n.addEventListener("click",d),t.addEventListener("click",l=>{l.target===t&&d()}),document.addEventListener("keydown",l=>{l.key==="Escape"&&t.style.display==="flex"&&d()}))}function d(){t&&(t.style.opacity="0",a.style.transform="scale(0.9)",setTimeout(()=>{t.style.display="none",i.innerHTML=""},300))}window.MarkdownPreviewModal={open:function(o){return x(this,null,function*(){if(!p){c(),p=!0,i.innerHTML="";var r=document.createElement("div");r.className="markdown-preview-loading",r.innerHTML=`
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 200px;
        color: #666;
      ">
        <div style="
          width: 40px;
          height: 40px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #3498db;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 12px;
        "></div>
        <div style="font-size: 14px;">\u52A0\u8F7D\u4E2D...</div>
      </div>
      <style>
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      </style>
    `,i.appendChild(r),t.style.display="flex",requestAnimationFrame(()=>{t.style.opacity="1",a.style.transform="scale(1)"});try{var l=yield(yield fetch("/api/markdown/preview?path="+encodeURIComponent(o))).json();if(!l.success)throw new Error(l.message||"\u52A0\u8F7D\u5931\u8D25");var m,g=l.data,y=(s.textContent=g.title,function(){let e=g.content;return e=(e=(e=(e=`<p style="margin: 10px 0;">${e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=(e=e.replace(/&/g,"&amp;")).replace(/</g,"&lt;")).replace(/>/g,"&gt;")).replace(/^### (.*$)/gim,"<h3>$1</h3>")).replace(/^## (.*$)/gim,"<h2>$1</h2>")).replace(/^# (.*$)/gim,"<h1>$1</h1>")).replace(/\*\*(.*?)\*\*/gim,"<strong>$1</strong>")).replace(/\*(.*?)\*/gim,"<em>$1</em>")).replace(/\[([^\]]+)\]\(([^)]+)\)/gim,'<a href="$2" target="_blank">$1</a>')).replace(/!\[([^\]]*)\]\(([^)]+)\)/gim,'<img src="$2" alt="$1" style="max-width: 100%; height: auto; border-radius: 4px; margin: 10px 0;">')).replace(/```(\w+)?\n([\s\S]*?)```/gim,"<pre><code>$2</code></pre>")).replace(/`([^`]+)`/gim,'<code style="background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: monospace;">$1</code>')).replace(/^> (.*$)/gim,'<blockquote style="border-left: 4px solid #ddd; padding-left: 16px; margin: 10px 0; color: #666;">$1</blockquote>')).replace(/^---$/gim,'<hr style="border: none; border-top: 1px solid #ddd; margin: 20px 0;">')).replace(/^\- (.*$)/gim,'<li style="margin: 4px 0;">$1</li>')).replace(/^(\d+)\. (.*$)/gim,'<li style="margin: 4px 0;">$2</li>')).replace(/\n\n/g,'</p><p style="margin: 10px 0;">')).replace(/\n/g,"<br>")}</p>`).replace(/<li>/g,'<ul style="margin: 10px 0; padding-left: 20px;"><li>')).replace(/<\/li>/g,"</li></ul>")).replace(/<\/ul><ul>/g,"")}());i.innerHTML=y,document.getElementById("markdown-preview-styles")||((m=document.createElement("style")).id="markdown-preview-styles",m.textContent=`
        .markdown-preview-body h1,
        .markdown-preview-body h2,
        .markdown-preview-body h3 {
          margin-top: 20px;
          margin-bottom: 10px;
          color: #333;
          font-weight: 600;
        }

        .markdown-preview-body h1 {
          font-size: 24px;
          border-bottom: 2px solid #e0e0e0;
          padding-bottom: 10px;
        }

        .markdown-preview-body h2 {
          font-size: 20px;
        }

        .markdown-preview-body h3 {
          font-size: 18px;
        }

        .markdown-preview-body p {
          margin: 10px 0;
          line-height: 1.6;
        }

        .markdown-preview-body a {
          color: #007bff;
          text-decoration: none;
        }

        .markdown-preview-body a:hover {
          text-decoration: underline;
        }

        .markdown-preview-body pre {
          background: #f4f4f4;
          padding: 16px;
          border-radius: 4px;
          overflow-x: auto;
          margin: 10px 0;
        }

        .markdown-preview-body code {
          font-family: 'Consolas', 'Monaco', monospace;
          font-size: 14px;
        }

        .markdown-preview-body blockquote {
          border-left: 4px solid #007bff;
          padding-left: 16px;
          margin: 10px 0;
          color: #666;
          font-style: italic;
        }

        .markdown-preview-body ul,
        .markdown-preview-body ol {
          margin: 10px 0;
          padding-left: 20px;
        }

        .markdown-preview-body li {
          margin: 4px 0;
        }

        .markdown-preview-body hr {
          border: none;
          border-top: 1px solid #e0e0e0;
          margin: 20px 0;
        }

        .markdown-preview-body img {
          max-width: 100%;
          height: auto;
          border-radius: 4px;
          margin: 10px 0;
        }
      `,document.head.appendChild(m))}catch(e){console.error("Failed to load markdown:",e),i.innerHTML=`
        <div style="
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
          color: #e74c3c;
        ">
          <div style="font-size: 48px; margin-bottom: 12px;">\u26A0\uFE0F</div>
          <div style="font-size: 14px; font-weight: 600;">\u52A0\u8F7D\u5931\u8D25</div>
          <div style="font-size: 12px; color: #666; margin-top: 4px;">${e.message}</div>
        </div>
      `}finally{p=!1}}})},close:d},document.readyState==="loading"?document.addEventListener("DOMContentLoaded",()=>{c()}):c()})();
