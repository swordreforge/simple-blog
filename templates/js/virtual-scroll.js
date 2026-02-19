class VirtualScroll{constructor(e={}){this.container=e.container,this.itemHeight=e.itemHeight||40,this.bufferSize=e.bufferSize||5,this.onRenderItem=e.onRenderItem,this.items=[],this.visibleStart=0,this.visibleEnd=0,this.totalHeight=0,this.scrollTop=0,this.init()}init(){this.contentContainer=document.createElement("div"),this.contentContainer.style.cssText=`
      position: relative;
      height: 100%;
      overflow: auto;
    `,this.virtualContent=document.createElement("div"),this.virtualContent.style.cssText=`
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      will-change: transform;
    `,this.contentContainer.appendChild(this.virtualContent),this.container.innerHTML="",this.container.appendChild(this.contentContainer),this.contentContainer.addEventListener("scroll",this.handleScroll.bind(this)),window.addEventListener("resize",this.handleResize.bind(this))}setItems(e){this.items=e,this.totalHeight=e.length*this.itemHeight,this.virtualContent.style.height=this.totalHeight+"px",this.render()}handleScroll(){this.scrollTop=this.contentContainer.scrollTop,this.render()}handleResize(){this.render()}render(){var e=this.contentContainer.clientHeight,t=Math.max(0,Math.floor(this.scrollTop/this.itemHeight)-this.bufferSize),i=Math.min(this.items.length,Math.ceil((this.scrollTop+e)/this.itemHeight)+this.bufferSize);if(t!==this.visibleStart||i!==this.visibleEnd){this.visibleStart=t,this.visibleEnd=i,this.virtualContent.innerHTML="";for(let e=t;e<i;e++){var s=this.items[e],l=e*this.itemHeight,o=document.createElement("div");o.style.cssText=`
        position: absolute;
        top: ${l}px;
        left: 0;
        right: 0;
        height: ${this.itemHeight}px;
        will-change: transform;
      `,this.onRenderItem&&this.onRenderItem(o,s,e),this.virtualContent.appendChild(o)}}}scrollToItem(e,t="smooth"){e<0||e>=this.items.length||(e=e*this.itemHeight,this.contentContainer.scrollTo({top:e,behavior:t}))}scrollToTop(e="smooth"){this.contentContainer.scrollTo({top:0,behavior:e})}getScrollPosition(){return this.scrollTop}destroy(){this.contentContainer.removeEventListener("scroll",this.handleScroll.bind(this)),window.removeEventListener("resize",this.handleResize.bind(this)),this.container.innerHTML=""}}class SidebarVirtualScroll extends VirtualScroll{constructor(e={}){super({...e,itemHeight:e.itemHeight||42}),this.folders=e.folders||[],this.onFolderToggle=e.onFolderToggle,this.onFileClick=e.onFileClick,this.flattenCache=new Map}setFolders(e){this.folders=e,this.flattenCache.clear(),this.updateFlattenedItems()}updateFlattenedItems(){var e=this.flattenFolders(this.folders);this.setItems(e)}flattenFolders(e,i=0,s=!0){const l=[];return e.forEach(t=>{l.push({type:"folder",id:t.id,name:t.name,level:i,open:t.open,parentId:t.parentId,originalFolder:t,fileCount:this.countFilesInFolder(t)}),t.open&&s&&(t.subfolders&&0<t.subfolders.length&&l.push(...this.flattenFolders(t.subfolders,i+1,!0)),t.files)&&0<t.files.length&&t.files.forEach(e=>{l.push({type:"file",id:e.id,name:e.title,level:i+1,file:e,parentId:t.id})})}),l}countFilesInFolder(e){let t=e.files?e.files.length:0;return e.subfolders&&e.subfolders.forEach(e=>{t+=this.countFilesInFolder(e)}),t}toggleFolder(e){var t=this.findFolderById(this.folders,e);t&&(t.open=!t.open,this.flattenCache.clear(),this.updateFlattenedItems(),this.onFolderToggle)&&this.onFolderToggle(e,t.open)}findFolderById(e,t){for(const s of e){if(s.id===t)return s;if(s.subfolders){var i=this.findFolderById(s.subfolders,t);if(i)return i}}return null}expandAll(){this.expandFoldersRecursive(this.folders),this.flattenCache.clear(),this.updateFlattenedItems()}expandFoldersRecursive(e){e.forEach(e=>{e.open=!0,e.subfolders&&this.expandFoldersRecursive(e.subfolders)})}collapseAll(){this.collapseFoldersRecursive(this.folders),this.flattenCache.clear(),this.updateFlattenedItems()}collapseFoldersRecursive(e){e.forEach(e=>{e.open=!1,e.subfolders&&this.collapseFoldersRecursive(e.subfolders)})}renderItem(e,t,i){"folder"===t.type?this.renderFolderItem(e,t,i):"file"===t.type&&this.renderFileItem(e,t,i)}renderFolderItem(e,t,i){var s=10+15*t.level,l=t.open?"90deg":"0deg",o=t.open?t.fileCount:t.fileCount+"+";e.className="virtual-folder-item",e.style.cssText=`
      position: absolute;
      top: ${i*this.itemHeight}px;
      left: 0;
      right: 0;
      height: ${this.itemHeight}px;
      padding-left: ${s}px;
      display: flex;
      align-items: center;
      cursor: pointer;
      transition: background-color 0.2s;
    `,e.innerHTML=`
      <span class="folder-icon" style="transform: rotate(${l}); transition: transform 0.3s;">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
        </svg>
      </span>
      <span class="folder-name" style="flex: 1; font-weight: 500; color: #f0f0f0; font-size: 0.9em;">${t.name}</span>
      <span class="file-count" style="background-color: rgba(255, 255, 255, 0.1); color: #aaa; font-size: 0.8em; padding: 2px 8px; border-radius: 10px;">${o}</span>
    `,e.addEventListener("mouseenter",()=>{e.style.backgroundColor="rgba(255, 255, 255, 0.05)"}),e.addEventListener("mouseleave",()=>{e.style.backgroundColor="transparent"}),e.addEventListener("click",()=>{this.toggleFolder(t.id)})}renderFileItem(e,t,i){var s=10+15*t.level,l="published"!==t.file.status,i=(e.className="virtual-file-item"+(l?" file-unpublished":""),e.style.cssText=`
      position: absolute;
      top: ${i*this.itemHeight}px;
      left: 0;
      right: 0;
      height: ${this.itemHeight}px;
      padding-left: ${s}px;
      padding-right: 10px;
      display: flex;
      align-items: center;
      cursor: pointer;
      transition: all 0.2s;
      ${l?"opacity: 0.7;":""}
    `,l?`<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
        </svg>`:`<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
          <polyline points="13 2 13 9 20 9"></polyline>
        </svg>`),s=t.file.date?t.file.date.split(" ")[0]:"";e.innerHTML=`
      <span class="file-icon" style="margin-right: 8px; color: ${l?"#ff9800":"#007bff"};">${i}</span>
      <span class="file-name" style="flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #ffffff; font-size: 0.95em; ${l?"color: #ff9800; font-style: italic;":""}">${t.name}</span>
      <span class="file-date" style="font-size: 0.8em; color: #aaa; margin-left: 8px;">${s}</span>
    `,e.addEventListener("mouseenter",()=>{e.style.backgroundColor="rgba(255, 255, 255, 0.05)"}),e.addEventListener("mouseleave",()=>{e.style.backgroundColor="transparent"}),e.addEventListener("click",()=>{this.onFileClick&&this.onFileClick(t.file)}),e.addEventListener("dblclick",()=>{this.onFileClick&&this.onFileClick(t.file,!0)})}}"undefined"!=typeof module&&module.exports&&(module.exports={VirtualScroll:VirtualScroll,SidebarVirtualScroll:SidebarVirtualScroll});
