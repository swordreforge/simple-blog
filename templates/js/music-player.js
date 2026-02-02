class MusicPlayer{constructor(){this.audio=null,this.isPlaying=!1,this.currentTrackIndex=0,this.playlist=[],this.autoPlayPending=!1,this.selectedPlaylistIndex=0,this.settings={enabled:!1,autoPlay:!1,controlSize:"medium",customCSS:"",playerColor:"rgba(66, 133, 244, 0.9)",position:"bottom-right"},this.init()}async init(){try{await this.loadSettings(),this.settings.enabled&&(this.createPlayer(),await this.restoreState()||(await this.loadPlaylist(),this.settings.autoPlay&&0<this.playlist.length&&setTimeout(()=>{this.tryAutoPlay()},500)),this.setupUserInteractionListener(),setInterval(()=>this.saveState(),5e3),window.addEventListener("beforeunload",()=>this.saveState()),this.audio)&&this.audio.addEventListener("timeupdate",()=>{isFinite(this.audio.currentTime)&&Math.floor(this.audio.currentTime)%5==0&&this.saveState()})}catch(t){console.error("音乐播放器初始化失败:",t)}}async tryAutoPlay(){if(console.log("尝试自动播放...",{autoPlay:this.settings.autoPlay,playlistLength:this.playlist.length}),this.settings.autoPlay&&0!==this.playlist.length)try{var e=this.playlist[0],i=(this.audio.src=e.url,document.querySelector("#volumeBar")),s=localStorage.getItem("musicPlayerState");let t=80;if(s)try{var a=JSON.parse(s);t=a.volume||80}catch(t){console.warn("Failed to parse saved state:",t)}i&&(i.value=t),this.audio.volume=t/100,this.currentTrackIndex=0,this.updateTrackInfo(e),this.updatePlaylistUI();var l=this.audio.play();void 0!==l&&l.then(()=>{this.isPlaying=!0,this.updatePlayButton(),console.log("音乐自动播放成功")}).catch(t=>{console.log("自动播放被阻止，等待用户交互:",t.message),this.autoPlayPending=!0,this.showAutoPlayHint()})}catch(t){console.error("自动播放尝试失败:",t)}else console.log("自动播放条件不满足")}showAutoPlayHint(){var e,i=document.getElementById("musicPlayer");if(i){let t=i.querySelector(".autoplay-hint");t||((t=document.createElement("div")).className="autoplay-hint",t.innerHTML=`
      <span>🎵 点击页面任意位置开始播放</span>
    `,(e=document.createElement("style")).textContent=`
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
    `,document.head.appendChild(e),i.appendChild(t),setTimeout(()=>{t&&t.parentNode&&t.parentNode.removeChild(t)},3e3))}}setupUserInteractionListener(){const t=["click","keydown","touchstart","scroll"];let e;e=()=>{this.autoPlayPending&&this.settings.autoPlay&&0<this.playlist.length&&(console.log("检测到用户交互，开始播放音乐"),this.playTrack(0),this.autoPlayPending=!1,t.forEach(t=>{document.removeEventListener(t,e)}))},t.forEach(t=>{document.addEventListener(t,e,{once:!0,passive:!0})})}async loadSettings(){try{var t,e,i=await fetch("/api/settings/music");i.ok&&(e={enabled:void 0!==(t=await i.json()).enabled?t.enabled:t.music_enabled,autoPlay:void 0!==t.autoPlay?t.autoPlay:t.auto_play,controlSize:void 0!==t.controlSize?t.controlSize:t.control_size,customCSS:void 0!==t.customCSS?t.customCSS:t.custom_css,playerColor:void 0!==t.playerColor?t.playerColor:t.player_color,position:void 0!==t.position?t.position:t.music_position},this.settings={...this.settings,...e},console.log("音乐设置已加载:",this.settings))}catch(t){console.error("加载音乐设置失败:",t)}}async loadPlaylist(){try{var t,e,i,s,a=await fetch("/api/music/playlist");a.ok&&(t=await a.json(),e=Array.isArray(t)?t:[],this.playlist=e.map(t=>({id:t.id,title:t.title,artist:t.artist,url:"/music/"+t.file_name,duration:t.duration||"未知",cover:t.cover_image||"/img/avatar.webp"})),this.updatePlaylistUI(),0<this.playlist.length&&!this.isPlaying&&(i=this.playlist[0],this.currentTrackIndex=0,this.updateTrackInfo(i),this.audio)&&(this.audio.src=i.url,s=document.querySelector("#volumeBar"),this.audio.volume=s?s.value/100:.8),this.preloadDurations())}catch(t){console.error("加载播放列表失败:",t)}}async preloadDurations(){for(let i=0;i<this.playlist.length;i++){var e=this.playlist[i];if("未知"===e.duration)try{const s=new Audio(e.url);await new Promise((t,e)=>{s.addEventListener("loadedmetadata",()=>{isNaN(s.duration)||(this.playlist[i].duration=this.formatTime(s.duration),this.updatePlaylistUI()),t()}),s.addEventListener("error",t),s.addEventListener("timeout",t)})}catch(t){console.warn(`Failed to load duration for ${e.title}:`,t)}}}createPlayer(){var t=document.createElement("div");t.id="musicPlayer",t.className=`music-player size-${this.settings.controlSize} position-`+this.settings.position,this.settings.customCSS&&(t.style.cssText+=this.settings.customCSS),document.documentElement.style.setProperty("--music-player-color",this.settings.playerColor),t.innerHTML=`
      <!-- 左侧封面 -->
      <div class="music-cover">
        <img id="musicCover" src="/img/avatar.webp" alt="音乐封面">
      </div>

      <!-- 中间区域 -->
      <div class="music-middle">
        <!-- 音乐信息 - 上栏 -->
        <div class="music-info">
          <div class="music-title" id="musicTitle">未播放</div>
          <div class="music-artist" id="musicArtist">-</div>
        </div>

        <!-- 播放控制 - 下栏 -->
        <div class="music-controls">
          <button class="rewind-btn" title="后退 {{.second}}秒">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="11 19 2 12 11 5 11 19"></polygon>
              <polygon points="22 19 13 12 22 5 22 19"></polygon>
            </svg>
          </button>
          <button class="play-btn" title="播放/暂停">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" id="playIcon">
              <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
          </button>
          <button class="forward-btn" title="前进 {{.second}}秒">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="13 19 22 12 13 5 13 19"></polygon>
              <polygon points="2 19 11 12 2 5 2 19"></polygon>
            </svg>
          </button>
        </div>
      </div>

      <!-- 音量控制 -->
      <div class="music-volume">
        <button class="volume-btn" title="音量 / 静音">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" id="volumeIcon">
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
          </svg>
        </button>
        <div class="music-volume-slider" id="volumeSlider">
          <input type="range" id="volumeBar" min="0" max="100" value="80" orient="vertical">
        </div>
      </div>

      <!-- 播放列表按钮 -->
      <button class="music-playlist-btn" title="歌单">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"></line>
          <line x1="8" y1="12" x2="21" y2="12"></line>
          <line x1="8" y1="18" x2="21" y2="18"></line>
          <line x1="3" y1="6" x2="3.01" y2="6"></line>
          <line x1="3" y1="12" x2="3.01" y2="12"></line>
          <line x1="3" y1="18" x2="3.01" y2="18"></line>
        </svg>
      </button>

      <!-- 倒计时 - 最右侧 -->
      <div class="music-countdown">
        <span id="countdownTime">-0:00</span>
      </div>

      <div class="music-playlist" id="musicPlaylist"></div>
    `,document.body.appendChild(t),this.audio=new Audio,this.bindEvents()}bindEvents(){var t,e=document.getElementById("musicPlayer");e&&(e.querySelector(".play-btn").addEventListener("click",()=>this.togglePlay()),e.querySelector(".rewind-btn").addEventListener("click",()=>this.rewind()),e.querySelector(".forward-btn").addEventListener("click",()=>this.forward()),e.querySelector("#volumeBar").addEventListener("input",t=>{this.audio&&(this.audio.volume=t.target.value/100,this.saveState())}),(t=e.querySelector(".volume-btn")).addEventListener("click",()=>this.toggleVolumeSlider()),t.addEventListener("contextmenu",t=>{t.preventDefault(),this.toggleMute()}),e.querySelector(".music-playlist-btn").addEventListener("click",()=>this.togglePlaylist()),this.audio&&(this.audio.addEventListener("timeupdate",()=>this.updateProgress()),this.audio.addEventListener("ended",()=>this.playNext()),this.audio.addEventListener("error",t=>{console.error("音频播放错误:",t),this.playNext()})),document.addEventListener("click",t=>{var e=document.querySelector(".music-volume");e&&!e.contains(t.target)&&(e=document.getElementById("volumeSlider"))&&e.classList.remove("show")}))}togglePlay(){this.isPlaying?this.pause():this.play()}play(){this.audio?!this.audio.src&&0<this.playlist.length?this.playTrack(0):this.audio.src?(this.audio.play(),this.isPlaying=!0,this.updatePlayButton(),this.saveState()):console.warn("播放列表为空，无法播放"):console.warn("音频元素未初始化")}pause(){this.audio&&(this.audio.pause(),this.isPlaying=!1,this.updatePlayButton(),this.saveState())}playTrack(e){if(0<=e&&e<this.playlist.length){var t=this.playlist[e];if(this.currentTrackIndex=e,this.autoPlayPending=!1,this.audio){this.audio.src=t.url;var i=document.querySelector("#volumeBar");if(i)this.audio.volume=i.value/100;else{i=localStorage.getItem("musicPlayerState");if(i)try{var s=JSON.parse(i);this.audio.volume=(s.volume||80)/100}catch(t){this.audio.volume=.8}else this.audio.volume=.8}const a=()=>{var t;this.audio&&isFinite(this.audio.duration)&&(t=this.formatTime(this.audio.duration),this.playlist[e].duration=t,this.updatePlaylistUI(),this.audio.removeEventListener("loadedmetadata",a))};this.audio.addEventListener("loadedmetadata",a),this.audio.play(),this.isPlaying=!0,this.updatePlayButton(),this.updateTrackInfo(t),this.updatePlaylistUI(),this.saveState()}}}playPrevious(){var t=this.currentTrackIndex-1;0<=t?this.playTrack(t):this.playTrack(this.playlist.length-1)}playNext(){var t=this.currentTrackIndex+1;t<this.playlist.length?this.playTrack(t):this.playTrack(0)}rewind(){var t;this.audio&&isFinite(this.audio.currentTime)&&(t=Math.max(0,this.audio.currentTime-5),this.audio.currentTime=t)}forward(){var t;this.audio&&isFinite(this.audio.duration)&&isFinite(this.audio.currentTime)&&(t=Math.min(this.audio.duration,this.audio.currentTime+5),this.audio.currentTime=t)}toggleMute(){this.audio&&(this.audio.muted=!this.audio.muted,this.updateVolumeButton())}toggleVolumeSlider(){var t=document.getElementById("volumeSlider");t&&t.classList.toggle("show")}togglePlaylist(){const e=document.getElementById("musicPlaylist");e&&(e.classList.contains("show")?(e.classList.remove("show"),this.playlistKeyHandler&&(document.removeEventListener("keydown",this.playlistKeyHandler),this.playlistKeyHandler=null)):(e.classList.add("show"),this.selectedPlaylistIndex=this.currentTrackIndex,this.updatePlaylistUI(),this.playlistKeyHandler=t=>{e.classList.contains("show")&&("ArrowUp"===t.key?(t.preventDefault(),t.stopPropagation(),this.selectedPlaylistIndex=Math.max(0,this.selectedPlaylistIndex-1),this.updatePlaylistUI()):"ArrowDown"===t.key?(t.preventDefault(),t.stopPropagation(),this.selectedPlaylistIndex=Math.min(this.playlist.length-1,this.selectedPlaylistIndex+1),this.updatePlaylistUI()):"Enter"===t.key?(t.preventDefault(),t.stopPropagation(),this.playTrack(this.selectedPlaylistIndex),e.classList.remove("show"),document.removeEventListener("keydown",this.playlistKeyHandler),this.playlistKeyHandler=null):"Escape"===t.key&&(t.preventDefault(),t.stopPropagation(),e.classList.remove("show"),document.removeEventListener("keydown",this.playlistKeyHandler),this.playlistKeyHandler=null))},document.addEventListener("keydown",this.playlistKeyHandler)))}updatePlayButton(){var t=document.getElementById("playIcon");t&&(this.isPlaying?t.innerHTML='<rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect>':t.innerHTML='<polygon points="5 3 19 12 5 21 5 3"></polygon>')}updateVolumeButton(){var t=document.getElementById("volumeIcon");t&&this.audio&&(this.audio.muted||0===this.audio.volume?t.innerHTML='<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><line x1="23" y1="9" x2="17" y2="15"></line><line x1="17" y1="9" x2="23" y2="15"></line>':t.innerHTML='<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>')}updateProgress(){var t,e;this.audio&&isFinite(this.audio.duration)&&isFinite(this.audio.currentTime)&&(t=document.querySelector("#countdownTime"))&&(e=this.audio.duration-this.audio.currentTime,t.textContent="-"+this.formatTime(e))}updateTrackInfo(t){var e=document.querySelector("#musicTitle"),i=document.querySelector("#musicArtist"),s=document.querySelector("#musicCover");e&&(e.textContent=this.removeTimestamp(t.title)),i&&(i.textContent=t.artist),s&&(s.src=t.cover||"/img/avatar.webp")}updatePlaylistUI(){var t=document.getElementById("musicPlaylist");t&&(t.innerHTML=this.playlist.map((t,e)=>{return`
      <div class="music-playlist-item ${e===this.currentTrackIndex?"active":""} ${e===this.selectedPlaylistIndex?"selected":""}" data-index="${e}">
        <div class="music-playlist-item-title">${this.removeTimestamp(t.title)}</div>
        <div class="music-playlist-item-duration">${t.duration}</div>
      </div>
    `}).join(""),t.querySelectorAll(".music-playlist-item").forEach(e=>{e.addEventListener("click",()=>{var t=parseInt(e.dataset.index);this.selectedPlaylistIndex=t,this.playTrack(t)})}))}removeTimestamp(t){var e=t.match(/^\d+_/);return e?t.substring(e[0].length):t}formatTime(t){return isNaN(t)?"0:00":Math.floor(t/60)+":"+Math.floor(t%60).toString().padStart(2,"0")}show(){var t=document.getElementById("musicPlayer");t&&t.classList.remove("hidden")}hide(){var t=document.getElementById("musicPlayer");t&&t.classList.add("hidden")}saveState(){try{var t=document.querySelector("#volumeBar"),e={currentTrackIndex:this.currentTrackIndex,isPlaying:this.isPlaying,currentTime:this.audio&&isFinite(this.audio.currentTime)?this.audio.currentTime:0,volume:t?t.value:this.audio?100*this.audio.volume:80,playlist:this.playlist};localStorage.setItem("musicPlayerState",JSON.stringify(e))}catch(t){console.warn("Failed to save music player state:",t)}}async restoreState(){try{var t,e,i,s=localStorage.getItem("musicPlayerState");return s?(t=JSON.parse(s),await this.loadPlaylist(),!!(t.playlist&&0<t.playlist.length&&0<this.playlist.length&&this.playlist[0].url===t.playlist[0].url)&&(this.currentTrackIndex=t.currentTrackIndex||0,this.currentTrackIndex<this.playlist.length&&(e=this.playlist[this.currentTrackIndex],this.audio.src=e.url,isFinite(t.currentTime)&&(this.audio.currentTime=t.currentTime),(i=document.querySelector("#volumeBar"))?(i.value=t.volume||80,this.audio.volume=i.value/100):this.audio.volume=t.volume?t.volume/100:.8,this.updateTrackInfo(e),this.updatePlaylistUI(),t.isPlaying)&&this.audio.play().then(()=>{this.isPlaying=!0,this.updatePlayButton()}).catch(t=>{console.log("恢复播放失败，等待用户交互:",t.message),this.autoPlayPending=!0}),!0)):!1}catch(t){return console.warn("Failed to restore music player state:",t),!1}}}let musicPlayer=null;document.addEventListener("DOMContentLoaded",()=>{musicPlayer=new MusicPlayer,window.MusicPlayer=MusicPlayer,window.musicPlayer=musicPlayer});
