!function(){let a=null,d=null,n=null,l=null,s=null,p=!1;function c(){var e,t;a||((a=document.createElement("div")).id="markdown-preview-modal",a.className="markdown-preview-modal",a.style.cssText=`
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
    `,(d=document.createElement("div")).className="markdown-preview-content",d.style.cssText=`
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
    `,(e=document.createElement("div")).className="markdown-preview-header",e.style.cssText=`
      padding: 16px 20px;
      border-bottom: 1px solid #e0e0e0;
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: #f5f5f5;
    `,(l=document.createElement("h3")).className="markdown-preview-title",l.style.cssText=`
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      color: #333;
    `,(n=document.createElement("button")).className="markdown-preview-close",n.innerHTML="×",n.style.cssText=`
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
    `,n.addEventListener("mouseenter",()=>{n.style.background="#e0e0e0",n.style.color="#333"}),n.addEventListener("mouseleave",()=>{n.style.background="none",n.style.color="#666"}),e.appendChild(l),e.appendChild(n),(s=document.createElement("div")).className="markdown-preview-body",s.style.cssText=`
      padding: 20px;
      overflow-y: auto;
      flex: 1;
      font-family: 'Segoe UI', 'Helvetica Neue', 'PingFang SC', 'Microsoft YaHei', sans-serif;
      line-height: 1.6;
      color: #333;
    `,(t=document.createElement("div")).className="markdown-preview-loading",t.innerHTML=`
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
        <div style="font-size: 14px;">加载中...</div>
      </div>
      <style>
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      </style>
    `,d.appendChild(e),d.appendChild(s),a.appendChild(d),document.body.appendChild(a),n.addEventListener("click",o),a.addEventListener("click",e=>{e.target===a&&o()}),document.addEventListener("keydown",e=>{"Escape"===e.key&&"flex"===a.style.display&&o()}))}function o(){a&&(a.style.opacity="0",d.style.transform="scale(0.9)",setTimeout(()=>{a.style.display="none",s.innerHTML=""},300))}window.MarkdownPreviewModal={open:async function(e){if(!p){c(),p=!0,s.innerHTML="";var t=document.createElement("div");t.className="markdown-preview-loading",t.innerHTML=`
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
        <div style="font-size: 14px;">加载中...</div>
      </div>
      <style>
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      </style>
    `,s.appendChild(t),a.style.display="flex",requestAnimationFrame(()=>{a.style.opacity="1",d.style.transform="scale(1)"});try{var n=await(await fetch("/api/markdown/preview?path="+encodeURIComponent(e))).json();if(!n.success)throw new Error(n.message||"加载失败");var o,i=n.data,r=(l.textContent=i.title,function(e){let t=e;return t=(t=(t=(t=`<p style="margin: 10px 0;">${t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=(t=t.replace(/&/g,"&amp;")).replace(/</g,"&lt;")).replace(/>/g,"&gt;")).replace(/^### (.*$)/gim,"<h3>$1</h3>")).replace(/^## (.*$)/gim,"<h2>$1</h2>")).replace(/^# (.*$)/gim,"<h1>$1</h1>")).replace(/\*\*(.*?)\*\*/gim,"<strong>$1</strong>")).replace(/\*(.*?)\*/gim,"<em>$1</em>")).replace(/\[([^\]]+)\]\(([^)]+)\)/gim,'<a href="$2" target="_blank">$1</a>')).replace(/!\[([^\]]*)\]\(([^)]+)\)/gim,'<img src="$2" alt="$1" style="max-width: 100%; height: auto; border-radius: 4px; margin: 10px 0;">')).replace(/```(\w+)?\n([\s\S]*?)```/gim,"<pre><code>$2</code></pre>")).replace(/`([^`]+)`/gim,'<code style="background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: monospace;">$1</code>')).replace(/^> (.*$)/gim,'<blockquote style="border-left: 4px solid #ddd; padding-left: 16px; margin: 10px 0; color: #666;">$1</blockquote>')).replace(/^---$/gim,'<hr style="border: none; border-top: 1px solid #ddd; margin: 20px 0;">')).replace(/^\- (.*$)/gim,'<li style="margin: 4px 0;">$1</li>')).replace(/^(\d+)\. (.*$)/gim,'<li style="margin: 4px 0;">$2</li>')).replace(/\n\n/g,'</p><p style="margin: 10px 0;">')).replace(/\n/g,"<br>")}</p>`).replace(/<li>/g,'<ul style="margin: 10px 0; padding-left: 20px;"><li>')).replace(/<\/li>/g,"</li></ul>")).replace(/<\/ul><ul>/g,"")}(i.content));s.innerHTML=r,document.getElementById("markdown-preview-styles")||((o=document.createElement("style")).id="markdown-preview-styles",o.textContent=`
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
      `,document.head.appendChild(o))}catch(e){console.error("Failed to load markdown:",e),s.innerHTML=`
        <div style="
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
          color: #e74c3c;
        ">
          <div style="font-size: 48px; margin-bottom: 12px;">⚠️</div>
          <div style="font-size: 14px; font-weight: 600;">加载失败</div>
          <div style="font-size: 12px; color: #666; margin-top: 4px;">${e.message}</div>
        </div>
      `}finally{p=!1}}},close:o},"loading"===document.readyState?document.addEventListener("DOMContentLoaded",()=>{c()}):c()}();
