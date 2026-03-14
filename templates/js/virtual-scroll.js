/* ESBuild compressed */
function n(t){var i=Math.max(0,Math.floor(t.scrollTop/t.g)-t.bufferSize),e=Math.min(t.items.length,Math.ceil((t.scrollTop+t.h.clientHeight)/t.g)+t.bufferSize);if(i!==t.l||e!==t.j){t.l=i,t.j=e,t.i.innerHTML="";for(let s=i;s<e;s++)i=document.createElement("div"),i.style.cssText=`
        position: absolute;
        top: ${s*t.g}px;
        left: 0;
        right: 0;
        height: ${t.g}px;
        will-change: transform;
      `,t.i.appendChild(i)}}class h{constructor(i={}){this.container=i.container,this.g=i.g||40,this.bufferSize=i.bufferSize||5,this.items=[],this.scrollTop=this.j=this.l=0,this.init()}init(){this.h=document.createElement("div"),this.h.style.cssText=`
      position: relative;
      height: 100%;
      overflow: auto;
    `,this.i=document.createElement("div"),this.i.style.cssText=`
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      will-change: transform;
    `,this.h.appendChild(this.i),this.container.innerHTML="",this.container.appendChild(this.h),this.h.addEventListener("scroll",this.o.bind(this)),window.addEventListener("resize",this.m.bind(this))}o(){this.scrollTop=this.h.scrollTop,n(this)}m(){n(this)}}class o extends h{constructor(i={}){super(Object.assign({},i,{g:i.g||42}))}}typeof module!="undefined"&&module.exports&&(module.exports={v:h,u:o});
