function e(a){var b=Math.max(0,Math.floor(a.scrollTop/a.g)-a.bufferSize),c=Math.min(a.items.length,Math.ceil((a.scrollTop+a.h.clientHeight)/a.g)+a.bufferSize);if(b!==a.l||c!==a.j){a.l=b;a.j=c;a.i.innerHTML="";for(let d=b;d<c;d++)b=document.createElement("div"),b.style.cssText=`
        position: absolute;
        top: ${d*a.g}px;
        left: 0;
        right: 0;
        height: ${a.g}px;
        will-change: transform;
      `,a.i.appendChild(b)}}
class f{constructor(a={}){this.container=a.container;this.g=a.g||40;this.bufferSize=a.bufferSize||5;this.items=[];this.scrollTop=this.j=this.l=0;this.init()}init(){this.h=document.createElement("div");this.h.style.cssText="\n      position: relative;\n      height: 100%;\n      overflow: auto;\n    ";this.i=document.createElement("div");this.i.style.cssText="\n      position: absolute;\n      top: 0;\n      left: 0;\n      right: 0;\n      will-change: transform;\n    ";this.h.appendChild(this.i);
this.container.innerHTML="";this.container.appendChild(this.h);this.h.addEventListener("scroll",this.o.bind(this));window.addEventListener("resize",this.m.bind(this))}o(){this.scrollTop=this.h.scrollTop;e(this)}m(){e(this)}}class g extends f{constructor(a={}){super(Object.assign({},a,{g:a.g||42}))}}"undefined"!=typeof module&&module.exports&&(module.exports={v:f,u:g});
