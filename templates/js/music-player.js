/* ESBuild compressed */
var v=Object.defineProperty;var h=Object.getOwnPropertySymbols;var g=Object.prototype.hasOwnProperty,P=Object.prototype.propertyIsEnumerable;var y=(o,t,e)=>t in o?v(o,t,{enumerable:!0,configurable:!0,writable:!0,value:e}):o[t]=e,c=(o,t)=>{for(var e in t||(t={}))g.call(t,e)&&y(o,e,t[e]);if(h)for(var e of h(t))P.call(t,e)&&y(o,e,t[e]);return o};var d=(o,t,e)=>new Promise((i,s)=>{var l=r=>{try{n(e.next(r))}catch(u){s(u)}},a=r=>{try{n(e.throw(r))}catch(u){s(u)}},n=r=>r.done?i(r.value):Promise.resolve(r.value).then(l,a);n((e=e.apply(o,t)).next())});class m{constructor(){this.audio=null,this.isPlaying=!1,this.currentTrackIndex=0,this.playlist=[],this.autoPlayPending=!1,this.selectedPlaylistIndex=0,this.settings={enabled:!1,autoPlay:!1,controlSize:"medium",customCSS:"",playerColor:"rgba(66, 133, 244, 0.9)",position:"bottom-right"},this.init()}init(){return d(this,null,function*(){try{yield this.loadSettings(),this.settings.enabled&&(this.createPlayer(),(yield this.restoreState())||(yield this.loadPlaylist(),this.settings.autoPlay&&0<this.playlist.length&&setTimeout(()=>{this.tryAutoPlay()},500)),this.setupUserInteractionListener(),setInterval(()=>this.saveState(),5e3),window.addEventListener("beforeunload",()=>this.saveState()),this.audio)&&this.audio.addEventListener("timeupdate",()=>{isFinite(this.audio.currentTime)&&Math.floor(this.audio.currentTime)%5==0&&this.saveState()})}catch(t){console.error("\u97F3\u4E50\u64AD\u653E\u5668\u521D\u59CB\u5316\u5931\u8D25:",t)}})}tryAutoPlay(){return d(this,null,function*(){if(console.log("\u5C1D\u8BD5\u81EA\u52A8\u64AD\u653E...",{autoPlay:this.settings.autoPlay,playlistLength:this.playlist.length}),this.settings.autoPlay&&this.playlist.length!==0)try{var t=this.playlist[0],e=(this.audio.src=t.url,document.querySelector("#volumeBar")),i=localStorage.getItem("musicPlayerState");let a=80;if(i)try{var s=JSON.parse(i);a=s.volume||80}catch(n){console.warn("Failed to parse saved state:",n)}e&&(e.value=a),this.audio.volume=a/100,this.currentTrackIndex=0,this.updateTrackInfo(t),this.updatePlaylistUI();var l=this.audio.play();l!==void 0&&l.then(()=>{this.isPlaying=!0,this.updatePlayButton(),console.log("\u97F3\u4E50\u81EA\u52A8\u64AD\u653E\u6210\u529F")}).catch(n=>{console.log("\u81EA\u52A8\u64AD\u653E\u88AB\u963B\u6B62\uFF0C\u7B49\u5F85\u7528\u6237\u4EA4\u4E92:",n.message),this.autoPlayPending=!0,this.showAutoPlayHint()})}catch(a){console.error("\u81EA\u52A8\u64AD\u653E\u5C1D\u8BD5\u5931\u8D25:",a)}else console.log("\u81EA\u52A8\u64AD\u653E\u6761\u4EF6\u4E0D\u6EE1\u8DB3")})}showAutoPlayHint(){var t,e=document.getElementById("musicPlayer");if(e){let i=e.querySelector(".autoplay-hint");i||((i=document.createElement("div")).className="autoplay-hint",i.innerHTML=`
      <span>\u{1F3B5} \u70B9\u51FB\u9875\u9762\u4EFB\u610F\u4F4D\u7F6E\u5F00\u59CB\u64AD\u653E</span>
    `,(t=document.createElement("style")).textContent=`
      .autoplay-hint {
        position: absolute;
        top: -40px;
        left: 50%;
        transform: translateX(-50%);
        background: var(--music-player-color, rgba(66, 133, 244, 0.9));
        color: white;
        padding: 8px 16px;
        border-radius: 20px;
        font-size: 12px;
        white-space: nowrap;
        animation: fadeInOut 3s ease-in-out;
        pointer-events: none;
        z-index: 1000;
      }

      @keyframes fadeInOut {
        0% { opacity: 0; transform: translateX(-50%) translateY(-10px); }
        20% { opacity: 1; transform: translateX(-50%) translateY(0); }
        80% { opacity: 1; transform: translateX(-50%) translateY(0); }
        100% { opacity: 0; transform: translateX(-50%) translateY(-10px); }
      }
    `,document.head.appendChild(t),e.appendChild(i),setTimeout(()=>{i&&i.parentNode&&i.parentNode.removeChild(i)},3e3))}}setupUserInteractionListener(){const t=["click","keydown","touchstart","scroll"];let e;e=()=>{this.autoPlayPending&&this.settings.autoPlay&&0<this.playlist.length&&(console.log("\u68C0\u6D4B\u5230\u7528\u6237\u4EA4\u4E92\uFF0C\u5F00\u59CB\u64AD\u653E\u97F3\u4E50"),this.playTrack(0),this.autoPlayPending=!1,t.forEach(i=>{document.removeEventListener(i,e)}))},t.forEach(i=>{document.addEventListener(i,e,{once:!0,passive:!0})})}loadSettings(){return d(this,null,function*(){try{var t,e,i=yield fetch("/api/settings/music");i.ok&&(e={enabled:(t=yield i.json()).enabled!==void 0?t.enabled:t.music_enabled,autoPlay:t.autoPlay!==void 0?t.autoPlay:t.auto_play,controlSize:t.controlSize!==void 0?t.controlSize:t.control_size,customCSS:t.customCSS!==void 0?t.customCSS:t.custom_css,playerColor:t.playerColor!==void 0?t.playerColor:t.player_color,position:t.position!==void 0?t.position:t.music_position},this.settings=c(c({},this.settings),e),console.log("\u97F3\u4E50\u8BBE\u7F6E\u5DF2\u52A0\u8F7D:",this.settings))}catch(s){console.error("\u52A0\u8F7D\u97F3\u4E50\u8BBE\u7F6E\u5931\u8D25:",s)}})}loadPlaylist(){return d(this,null,function*(){try{var t,e,i,s,l=yield fetch("/api/music/playlist");l.ok&&(t=yield l.json(),e=Array.isArray(t)?t:[],this.playlist=e.map(a=>({id:a.id,title:a.title,artist:a.artist,url:"/music/"+a.file_name,duration:a.duration||"\u672A\u77E5",cover:a.cover_image||"/img/avatar.webp"})),this.updatePlaylistUI(),0<this.playlist.length&&!this.isPlaying&&(i=this.playlist[0],this.currentTrackIndex=0,this.updateTrackInfo(i),this.audio)&&(this.audio.src=i.url,s=document.querySelector("#volumeBar"),this.audio.volume=s?s.value/100:.8),this.preloadDurations())}catch(a){console.error("\u52A0\u8F7D\u64AD\u653E\u5217\u8868\u5931\u8D25:",a)}})}preloadDurations(){return d(this,null,function*(){for(let e=0;e<this.playlist.length;e++){var t=this.playlist[e];if(t.duration==="\u672A\u77E5")try{const i=new Audio(t.url);yield new Promise((s,l)=>{i.addEventListener("loadedmetadata",()=>{isNaN(i.duration)||(this.playlist[e].duration=this.formatTime(i.duration),this.updatePlaylistUI()),s()}),i.addEventListener("error",s),i.addEventListener("timeout",s)})}catch(i){console.warn(`Failed to load duration for ${t.title}:`,i)}}})}createPlayer(){var t=document.createElement("div");t.id="musicPlayer",t.className=`music-player size-${this.settings.controlSize} position-`+this.settings.position,this.settings.customCSS&&(t.style.cssText+=this.settings.customCSS),document.documentElement.style.setProperty("--music-player-color",this.settings.playerColor),t.innerHTML=`
      <!-- \u5DE6\u4FA7\u5C01\u9762 -->
      <div class="music-cover">
        <img id="musicCover" src="/img/avatar.webp" alt="\u97F3\u4E50\u5C01\u9762">
      </div>

      <!-- \u4E2D\u95F4\u533A\u57DF -->
      <div class="music-middle">
        <!-- \u97F3\u4E50\u4FE1\u606F - \u4E0A\u680F -->
        <div class="music-info">
          <div class="music-title" id="musicTitle">\u672A\u64AD\u653E</div>
          <div class="music-artist" id="musicArtist">-</div>
        </div>

        <!-- \u64AD\u653E\u63A7\u5236 - \u4E0B\u680F -->
        <div class="music-controls">
          <button class="rewind-btn" title="\u540E\u9000 {{.second}}\u79D2">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="11 19 2 12 11 5 11 19"></polygon>
              <polygon points="22 19 13 12 22 5 22 19"></polygon>
            </svg>
          </button>
          <button class="play-btn" title="\u64AD\u653E/\u6682\u505C">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" id="playIcon">
              <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
          </button>
          <button class="forward-btn" title="\u524D\u8FDB {{.second}}\u79D2">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="13 19 22 12 13 5 13 19"></polygon>
              <polygon points="2 19 11 12 2 5 2 19"></polygon>
            </svg>
          </button>
        </div>
      </div>

      <!-- \u97F3\u91CF\u63A7\u5236 -->
      <div class="music-volume">
        <button class="volume-btn" title="\u97F3\u91CF / \u9759\u97F3">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" id="volumeIcon">
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
          </svg>
        </button>
        <div class="music-volume-slider" id="volumeSlider">
          <input type="range" id="volumeBar" min="0" max="100" value="80" orient="vertical">
        </div>
      </div>

      <!-- \u64AD\u653E\u5217\u8868\u6309\u94AE -->
      <button class="music-playlist-btn" title="\u6B4C\u5355">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"></line>
          <line x1="8" y1="12" x2="21" y2="12"></line>
          <line x1="8" y1="18" x2="21" y2="18"></line>
          <line x1="3" y1="6" x2="3.01" y2="6"></line>
          <line x1="3" y1="12" x2="3.01" y2="12"></line>
          <line x1="3" y1="18" x2="3.01" y2="18"></line>
        </svg>
      </button>

      <!-- \u5012\u8BA1\u65F6 - \u6700\u53F3\u4FA7 -->
      <div class="music-countdown">
        <span id="countdownTime">0:00</span>
      </div>

      <div class="music-playlist" id="musicPlaylist"></div>
    `,document.body.appendChild(t),this.audio=new Audio,this.bindEvents()}bindEvents(){var t,e=document.getElementById("musicPlayer");e&&(e.querySelector(".play-btn").addEventListener("click",()=>this.togglePlay()),e.querySelector(".rewind-btn").addEventListener("click",()=>this.rewind()),e.querySelector(".forward-btn").addEventListener("click",()=>this.forward()),e.querySelector("#volumeBar").addEventListener("input",i=>{this.audio&&(this.audio.volume=i.target.value/100,this.saveState())}),(t=e.querySelector(".volume-btn")).addEventListener("click",()=>this.toggleVolumeSlider()),t.addEventListener("contextmenu",i=>{i.preventDefault(),this.toggleMute()}),e.querySelector(".music-playlist-btn").addEventListener("click",()=>this.togglePlaylist()),this.audio&&(this.audio.addEventListener("timeupdate",()=>this.updateProgress()),this.audio.addEventListener("ended",()=>this.playNext()),this.audio.addEventListener("error",i=>{console.error("\u97F3\u9891\u64AD\u653E\u9519\u8BEF:",i),this.playNext()})),document.addEventListener("click",i=>{var s=document.querySelector(".music-volume");s&&!s.contains(i.target)&&(s=document.getElementById("volumeSlider"))&&s.classList.remove("show")}))}togglePlay(){this.isPlaying?this.pause():this.play()}play(){this.audio?!this.audio.src&&0<this.playlist.length?this.playTrack(0):this.audio.src?(this.audio.play(),this.isPlaying=!0,this.updatePlayButton(),this.saveState()):console.warn("\u64AD\u653E\u5217\u8868\u4E3A\u7A7A\uFF0C\u65E0\u6CD5\u64AD\u653E"):console.warn("\u97F3\u9891\u5143\u7D20\u672A\u521D\u59CB\u5316")}pause(){this.audio&&(this.audio.pause(),this.isPlaying=!1,this.updatePlayButton(),this.saveState())}playTrack(t){if(0<=t&&t<this.playlist.length){var e=this.playlist[t];if(this.currentTrackIndex=t,this.autoPlayPending=!1,this.audio){this.audio.src=e.url;var i=document.querySelector("#volumeBar");if(i)this.audio.volume=i.value/100;else if(i=localStorage.getItem("musicPlayerState"),i)try{var s=JSON.parse(i);this.audio.volume=(s.volume||80)/100}catch(a){this.audio.volume=.8}else this.audio.volume=.8;const l=()=>{var a;this.audio&&isFinite(this.audio.duration)&&(a=this.formatTime(this.audio.duration),this.playlist[t].duration=a,this.updatePlaylistUI(),this.audio.removeEventListener("loadedmetadata",l))};this.audio.addEventListener("loadedmetadata",l),this.audio.play(),this.isPlaying=!0,this.updatePlayButton(),this.updateTrackInfo(e),this.updatePlaylistUI(),this.saveState()}}}playPrevious(){var t=this.currentTrackIndex-1;0<=t?this.playTrack(t):this.playTrack(this.playlist.length-1)}playNext(){var t=this.currentTrackIndex+1;t<this.playlist.length?this.playTrack(t):this.playTrack(0)}rewind(){var t;this.audio&&isFinite(this.audio.currentTime)&&(t=Math.max(0,this.audio.currentTime-5),this.audio.currentTime=t)}forward(){var t;this.audio&&isFinite(this.audio.duration)&&isFinite(this.audio.currentTime)&&(t=Math.min(this.audio.duration,this.audio.currentTime+5),this.audio.currentTime=t)}toggleMute(){this.audio&&(this.audio.muted=!this.audio.muted,this.updateVolumeButton())}toggleVolumeSlider(){var t=document.getElementById("volumeSlider");t&&t.classList.toggle("show")}togglePlaylist(){const t=document.getElementById("musicPlaylist");t&&(t.classList.contains("show")?(t.classList.remove("show"),this.playlistKeyHandler&&(document.removeEventListener("keydown",this.playlistKeyHandler),this.playlistKeyHandler=null)):(t.classList.add("show"),this.selectedPlaylistIndex=this.currentTrackIndex,this.updatePlaylistUI(),this.playlistKeyHandler=e=>{t.classList.contains("show")&&(e.key==="ArrowUp"?(e.preventDefault(),e.stopPropagation(),this.selectedPlaylistIndex=Math.max(0,this.selectedPlaylistIndex-1),this.updatePlaylistUI()):e.key==="ArrowDown"?(e.preventDefault(),e.stopPropagation(),this.selectedPlaylistIndex=Math.min(this.playlist.length-1,this.selectedPlaylistIndex+1),this.updatePlaylistUI()):e.key==="Enter"?(e.preventDefault(),e.stopPropagation(),this.playTrack(this.selectedPlaylistIndex),t.classList.remove("show"),document.removeEventListener("keydown",this.playlistKeyHandler),this.playlistKeyHandler=null):e.key==="Escape"&&(e.preventDefault(),e.stopPropagation(),t.classList.remove("show"),document.removeEventListener("keydown",this.playlistKeyHandler),this.playlistKeyHandler=null))},document.addEventListener("keydown",this.playlistKeyHandler)))}updatePlayButton(){var t=document.getElementById("playIcon");t&&(this.isPlaying?t.innerHTML='<rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect>':t.innerHTML='<polygon points="5 3 19 12 5 21 5 3"></polygon>')}updateVolumeButton(){var t=document.getElementById("volumeIcon");t&&this.audio&&(this.audio.muted||this.audio.volume===0?t.innerHTML='<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><line x1="23" y1="9" x2="17" y2="15"></line><line x1="17" y1="9" x2="23" y2="15"></line>':t.innerHTML='<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>')}updateProgress(){var t,e;this.audio&&isFinite(this.audio.duration)&&isFinite(this.audio.currentTime)&&(t=document.querySelector("#countdownTime"))&&(e=this.audio.duration-this.audio.currentTime,t.textContent="-"+this.formatTime(e))}updateTrackInfo(t){var e=document.querySelector("#musicTitle"),i=document.querySelector("#musicArtist"),s=document.querySelector("#musicCover");e&&(e.textContent=this.removeTimestamp(t.title)),i&&(i.textContent=t.artist),s&&(s.src=t.cover||"/img/avatar.webp")}updatePlaylistUI(){var t=document.getElementById("musicPlaylist");t&&(t.innerHTML=this.playlist.map((e,i)=>`
      <div class="music-playlist-item ${i===this.currentTrackIndex?"active":""} ${i===this.selectedPlaylistIndex?"selected":""}" data-index="${i}">
        <div class="music-playlist-item-title">${this.removeTimestamp(e.title)}</div>
        <div class="music-playlist-item-duration">${e.duration}</div>
      </div>
    `).join(""),t.querySelectorAll(".music-playlist-item").forEach(e=>{e.addEventListener("click",()=>{var i=parseInt(e.dataset.index);this.selectedPlaylistIndex=i,this.playTrack(i)})}))}removeTimestamp(t){var e=t.match(/^\d+_/);return e?t.substring(e[0].length):t}formatTime(t){return isNaN(t)?"0:00":Math.floor(t/60)+":"+Math.floor(t%60).toString().padStart(2,"0")}show(){var t=document.getElementById("musicPlayer");t&&t.classList.remove("hidden")}hide(){var t=document.getElementById("musicPlayer");t&&t.classList.add("hidden")}saveState(){try{var t=document.querySelector("#volumeBar"),e={currentTrackIndex:this.currentTrackIndex,isPlaying:this.isPlaying,currentTime:this.audio&&isFinite(this.audio.currentTime)?this.audio.currentTime:0,volume:t?t.value:this.audio?100*this.audio.volume:80,playlist:this.playlist};localStorage.setItem("musicPlayerState",JSON.stringify(e))}catch(i){console.warn("Failed to save music player state:",i)}}restoreState(){return d(this,null,function*(){try{var t,e,i,s=localStorage.getItem("musicPlayerState");return s?(t=JSON.parse(s),yield this.loadPlaylist(),!!(t.playlist&&0<t.playlist.length&&0<this.playlist.length&&this.playlist[0].url===t.playlist[0].url)&&(this.currentTrackIndex=t.currentTrackIndex||0,this.currentTrackIndex<this.playlist.length&&(e=this.playlist[this.currentTrackIndex],this.audio.src=e.url,isFinite(t.currentTime)&&(this.audio.currentTime=t.currentTime),(i=document.querySelector("#volumeBar"))?(i.value=t.volume||80,this.audio.volume=i.value/100):this.audio.volume=t.volume?t.volume/100:.8,this.updateTrackInfo(e),this.updatePlaylistUI(),t.isPlaying)&&this.audio.play().then(()=>{this.isPlaying=!0,this.updatePlayButton()}).catch(l=>{console.log("\u6062\u590D\u64AD\u653E\u5931\u8D25\uFF0C\u7B49\u5F85\u7528\u6237\u4EA4\u4E92:",l.message),this.autoPlayPending=!0}),!0)):!1}catch(l){return console.warn("Failed to restore music player state:",l),!1}})}}let p=null;document.addEventListener("DOMContentLoaded",()=>{p=new m,window.MusicPlayer=m,window.musicPlayer=p});
