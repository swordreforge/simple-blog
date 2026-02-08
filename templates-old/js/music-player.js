/**
 * 音乐播放器管理器
 */
class MusicPlayer {
  constructor() {
    this.audio = null;
    this.isPlaying = false;
    this.currentTrackIndex = 0;
    this.playlist = [];
    this.autoPlayPending = false; // 自动播放待处理标志
    this.selectedPlaylistIndex = 0; // 播放列表选中的索引
    this.settings = {
      enabled: false,
      autoPlay: false,
      controlSize: 'medium',
      customCSS: '',
      playerColor: 'rgba(66, 133, 244, 0.9)',
      position: 'bottom-right'
    };

    this.init();
  }

  async init() {
    try {
      // 加载音乐设置
      await this.loadSettings();

      // 如果未启用，不显示播放器
      if (!this.settings.enabled) {
        return;
      }

      // 创建播放器 UI
      this.createPlayer();

      // 尝试恢复播放状态
      const restored = await this.restoreState();

      // 如果没有恢复状态，则加载播放列表
      if (!restored) {
        await this.loadPlaylist();

        // 如果设置了自动播放，尝试自动播放第一首
        if (this.settings.autoPlay && this.playlist.length > 0) {
          // 延迟尝试自动播放，等待页面完全加载
          setTimeout(() => {
            this.tryAutoPlay();
          }, 500);
        }
      }

      // 监听用户交互，如果自动播放失败，在用户第一次交互时播放
      this.setupUserInteractionListener();

      // 定期保存状态
      setInterval(() => this.saveState(), 5000);

      // 监听页面卸载事件，保存状态
      window.addEventListener('beforeunload', () => this.saveState());

      // 监听音频事件，实时保存状态
      if (this.audio) {
        this.audio.addEventListener('timeupdate', () => {
          // 每5秒保存一次当前播放位置
          if (isFinite(this.audio.currentTime) && Math.floor(this.audio.currentTime) % 5 === 0) {
            this.saveState();
          }
        });
      }
    } catch (error) {
      console.error('音乐播放器初始化失败:', error);
    }
  }

  // 尝试自动播放
  async tryAutoPlay() {
    console.log('尝试自动播放...', {
      autoPlay: this.settings.autoPlay,
      playlistLength: this.playlist.length
    });

    if (!this.settings.autoPlay || this.playlist.length === 0) {
      console.log('自动播放条件不满足');
      return;
    }

    try {
      // 先加载第一首但不播放
      const firstTrack = this.playlist[0];
      this.audio.src = firstTrack.url;

      // 安全地设置音量，优先使用保存的音量
      const volumeBar = document.querySelector('#volumeBar');
      const savedState = localStorage.getItem('musicPlayerState');
      let savedVolume = 80; // 默认音量

      if (savedState) {
        try {
          const state = JSON.parse(savedState);
          savedVolume = state.volume || 80;
        } catch (e) {
          console.warn('Failed to parse saved state:', e);
        }
      }

      if (volumeBar) {
        volumeBar.value = savedVolume;
        this.audio.volume = savedVolume / 100;
      } else {
        this.audio.volume = savedVolume / 100;
      }

      // 设置播放器的初始状态（即使不播放也要显示第一首歌的信息）
      this.currentTrackIndex = 0;
      this.updateTrackInfo(firstTrack);
      this.updatePlaylistUI();

      // 尝试播放
      const playPromise = this.audio.play();

      if (playPromise !== undefined) {
        playPromise.then(() => {
          // 播放成功
          this.isPlaying = true;
          this.updatePlayButton();
          console.log('音乐自动播放成功');
        }).catch(error => {
          // 播放失败（可能是浏览器阻止），等待用户交互
          console.log('自动播放被阻止，等待用户交互:', error.message);
          this.autoPlayPending = true;

          // 显示提示信息
          this.showAutoPlayHint();
        });
      }
    } catch (error) {
      console.error('自动播放尝试失败:', error);
    }
  }

  // 显示自动播放提示
  showAutoPlayHint() {
    const player = document.getElementById('musicPlayer');
    if (!player) return;

    // 检查是否已有提示
    let hint = player.querySelector('.autoplay-hint');
    if (hint) return;

    // 创建提示元素
    hint = document.createElement('div');
    hint.className = 'autoplay-hint';
    hint.innerHTML = `
      <span>🎵 点击页面任意位置开始播放</span>
    `;

    // 添加提示样式
    const style = document.createElement('style');
    style.textContent = `
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
    `;

    document.head.appendChild(style);
    player.appendChild(hint);

    // 3秒后移除提示
    setTimeout(() => {
      if (hint && hint.parentNode) {
        hint.parentNode.removeChild(hint);
      }
    }, 3000);
  }

  // 设置用户交互监听器
  setupUserInteractionListener() {
    const userEvents = ['click', 'keydown', 'touchstart', 'scroll'];
    let interactionHandler = null;

    interactionHandler = () => {
      if (this.autoPlayPending && this.settings.autoPlay && this.playlist.length > 0) {
        console.log('检测到用户交互，开始播放音乐');
        this.playTrack(0);
        this.autoPlayPending = false;

        // 移除事件监听器
        userEvents.forEach(event => {
          document.removeEventListener(event, interactionHandler);
        });
      }
    };

    // 添加事件监听器
    userEvents.forEach(event => {
      document.addEventListener(event, interactionHandler, { once: true, passive: true });
    });
  }

  async loadSettings() {
    try {
      const response = await fetch('/api/settings/music');
      if (response.ok) {
        const settings = await response.json();
        // 将下划线命名转换为驼峰命名
        const normalizedSettings = {
          enabled: settings.enabled !== undefined ? settings.enabled : settings.music_enabled,
          autoPlay: settings.autoPlay !== undefined ? settings.autoPlay : settings.auto_play,
          controlSize: settings.controlSize !== undefined ? settings.controlSize : settings.control_size,
          customCSS: settings.customCSS !== undefined ? settings.customCSS : settings.custom_css,
          playerColor: settings.playerColor !== undefined ? settings.playerColor : settings.player_color,
          position: settings.position !== undefined ? settings.position : settings.music_position
        };
        this.settings = { ...this.settings, ...normalizedSettings };
        console.log('音乐设置已加载:', this.settings);
      }
    } catch (error) {
      console.error('加载音乐设置失败:', error);
    }
  }

  async loadPlaylist() {
    try {
      const response = await fetch('/api/music/playlist');
      if (response.ok) {
        const tracks = await response.json();
        // 确保tracks是数组，防止null
        const trackList = Array.isArray(tracks) ? tracks : [];
        this.playlist = trackList.map(track => ({
          id: track.id,
          title: track.title,
          artist: track.artist,
          url: `/music/${track.file_name}`,
          duration: track.duration || '未知',
          cover: track.cover_image || '/img/avatar.webp'
        }));
        this.updatePlaylistUI();

        // 如果播放列表不为空，显示第一首歌的信息并设置音频源
        if (this.playlist.length > 0 && !this.isPlaying) {
          const firstTrack = this.playlist[0];
          this.currentTrackIndex = 0;
          this.updateTrackInfo(firstTrack);

          // 设置音频源但不自动播放
          if (this.audio) {
            this.audio.src = firstTrack.url;
            const volumeBar = document.querySelector('#volumeBar');
            if (volumeBar) {
              this.audio.volume = volumeBar.value / 100;
            } else {
              this.audio.volume = 0.8;
            }
          }
        }

        // 预加载所有音频时长
        this.preloadDurations();
      }
    } catch (error) {
      console.error('加载播放列表失败:', error);
    }
  }

  // 预加载所有音频的时长
  async preloadDurations() {
    for (let i = 0; i < this.playlist.length; i++) {
      const track = this.playlist[i];
      if (track.duration === '未知') {
        try {
          const tempAudio = new Audio(track.url);
          await new Promise((resolve, reject) => {
            tempAudio.addEventListener('loadedmetadata', () => {
              if (!isNaN(tempAudio.duration)) {
                this.playlist[i].duration = this.formatTime(tempAudio.duration);
                this.updatePlaylistUI();
              }
              resolve();
            });
            tempAudio.addEventListener('error', resolve);
            tempAudio.addEventListener('timeout', resolve);
          });
        } catch (error) {
          console.warn(`Failed to load duration for ${track.title}:`, error);
        }
      }
    }
  }

  createPlayer() {
    // 创建播放器容器
    const playerContainer = document.createElement('div');
    playerContainer.id = 'musicPlayer';
    playerContainer.className = `music-player size-${this.settings.controlSize} position-${this.settings.position}`;

    // 应用自定义 CSS
    if (this.settings.customCSS) {
      playerContainer.style.cssText += this.settings.customCSS;
    }

    // 设置播放器颜色变量
    document.documentElement.style.setProperty('--music-player-color', this.settings.playerColor);

    playerContainer.innerHTML = `
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
    `;

    document.body.appendChild(playerContainer);

    // 创建音频元素
    this.audio = new Audio();

    // 绑定事件
    this.bindEvents();
  }

  bindEvents() {
    const player = document.getElementById('musicPlayer');
    if (!player) return;

    // 播放/暂停按钮
    const playBtn = player.querySelector('.play-btn');
    playBtn.addEventListener('click', () => this.togglePlay());

    // 后退按钮
    const rewindBtn = player.querySelector('.rewind-btn');
    rewindBtn.addEventListener('click', () => this.rewind());

    // 前进按钮
    const forwardBtn = player.querySelector('.forward-btn');
    forwardBtn.addEventListener('click', () => this.forward());

    // 音量条
    const volumeBar = player.querySelector('#volumeBar');
    volumeBar.addEventListener('input', (e) => {
      if (this.audio) {
        this.audio.volume = e.target.value / 100;
        // 立即保存音量设置
        this.saveState();
      }
    });

    // 音量按钮 - 点击展开/收起滑块
    const volumeBtn = player.querySelector('.volume-btn');
    volumeBtn.addEventListener('click', () => this.toggleVolumeSlider());

    // 音量按钮 - 切换静音（右键）
    volumeBtn.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      this.toggleMute();
    });

    // 播放列表按钮
    const playlistBtn = player.querySelector('.music-playlist-btn');
    playlistBtn.addEventListener('click', () => this.togglePlaylist());

    // 音频事件
    if (this.audio) {
      this.audio.addEventListener('timeupdate', () => this.updateProgress());
      this.audio.addEventListener('ended', () => this.playNext());
      this.audio.addEventListener('error', (e) => {
        console.error('音频播放错误:', e);
        this.playNext();
      });
    }

    // 点击其他地方关闭音量滑块
    document.addEventListener('click', (e) => {
      const volumeContainer = document.querySelector('.music-volume');
      if (volumeContainer && !volumeContainer.contains(e.target)) {
        const volumeSlider = document.getElementById('volumeSlider');
        if (volumeSlider) {
          volumeSlider.classList.remove('show');
        }
      }
    });
  }

  togglePlay() {
    if (this.isPlaying) {
      this.pause();
    } else {
      this.play();
    }
  }

  play() {
    if (!this.audio) {
      console.warn('音频元素未初始化');
      return;
    }

    // 如果音频源未设置，播放第一首歌曲
    if (!this.audio.src && this.playlist.length > 0) {
      this.playTrack(0);
      return;
    }

    if (this.audio.src) {
      this.audio.play();
      this.isPlaying = true;
      this.updatePlayButton();
      this.saveState();
    } else {
      console.warn('播放列表为空，无法播放');
    }
  }

  pause() {
    if (this.audio) {
      this.audio.pause();
      this.isPlaying = false;
      this.updatePlayButton();
      this.saveState();
    }
  }

  playTrack(index) {
    if (index >= 0 && index < this.playlist.length) {
      const track = this.playlist[index];
      this.currentTrackIndex = index;

      // 清除自动播放待处理状态
      this.autoPlayPending = false;

      if (this.audio) {
        this.audio.src = track.url;

        // 使用保存的音量设置
        const volumeBar = document.querySelector('#volumeBar');
        if (volumeBar) {
          this.audio.volume = volumeBar.value / 100;
        } else {
          // 如果没有音量条，尝试从保存的状态中获取
          const savedState = localStorage.getItem('musicPlayerState');
          if (savedState) {
            try {
              const state = JSON.parse(savedState);
              this.audio.volume = (state.volume || 80) / 100;
            } catch (e) {
              this.audio.volume = 0.8;
            }
          } else {
            this.audio.volume = 0.8;
          }
        }

        // 监听音频加载完成事件，获取时长
        const onLoadedMetadata = () => {
          if (this.audio && isFinite(this.audio.duration)) {
            const duration = this.formatTime(this.audio.duration);
            this.playlist[index].duration = duration;
            this.updatePlaylistUI();
            this.audio.removeEventListener('loadedmetadata', onLoadedMetadata);
          }
        };

        this.audio.addEventListener('loadedmetadata', onLoadedMetadata);

        this.audio.play();
        this.isPlaying = true;
        this.updatePlayButton();
        this.updateTrackInfo(track);
        this.updatePlaylistUI();

        // 保存状态
        this.saveState();
      }
    }
  }

  playPrevious() {
    const prevIndex = this.currentTrackIndex - 1;
    if (prevIndex >= 0) {
      this.playTrack(prevIndex);
    } else {
      this.playTrack(this.playlist.length - 1);
    }
  }

  playNext() {
    const nextIndex = this.currentTrackIndex + 1;
    if (nextIndex < this.playlist.length) {
      this.playTrack(nextIndex);
    } else {
      this.playTrack(0);
    }
  }

  rewind() {
    if (this.audio && isFinite(this.audio.currentTime)) {
      const newTime = Math.max(0, this.audio.currentTime - 5);
      this.audio.currentTime = newTime;
    }
  }

  forward() {
    if (this.audio && isFinite(this.audio.duration) && isFinite(this.audio.currentTime)) {
      const newTime = Math.min(this.audio.duration, this.audio.currentTime + 5);
      this.audio.currentTime = newTime;
    }
  }

  toggleMute() {
    if (this.audio) {
      this.audio.muted = !this.audio.muted;
      this.updateVolumeButton();
    }
  }

  toggleVolumeSlider() {
    const volumeSlider = document.getElementById('volumeSlider');
    if (volumeSlider) {
      volumeSlider.classList.toggle('show');
    }
  }

  togglePlaylist() {
    const playlist = document.getElementById('musicPlaylist');
    if (playlist) {
      const isShowing = playlist.classList.contains('show');

      if (isShowing) {
        // 关闭播放列表，移除键盘事件监听
        playlist.classList.remove('show');
        if (this.playlistKeyHandler) {
          document.removeEventListener('keydown', this.playlistKeyHandler);
          this.playlistKeyHandler = null;
        }
      } else {
        // 打开播放列表，添加键盘事件监听
        playlist.classList.add('show');
        this.selectedPlaylistIndex = this.currentTrackIndex; // 默认选中当前播放的歌曲
        this.updatePlaylistUI();

        // 添加键盘事件监听
        this.playlistKeyHandler = (e) => {
          if (!playlist.classList.contains('show')) return;

          if (e.key === 'ArrowUp') {
            e.preventDefault();
            e.stopPropagation();
            this.selectedPlaylistIndex = Math.max(0, this.selectedPlaylistIndex - 1);
            this.updatePlaylistUI();
          } else if (e.key === 'ArrowDown') {
            e.preventDefault();
            e.stopPropagation();
            this.selectedPlaylistIndex = Math.min(this.playlist.length - 1, this.selectedPlaylistIndex + 1);
            this.updatePlaylistUI();
          } else if (e.key === 'Enter') {
            e.preventDefault();
            e.stopPropagation();
            this.playTrack(this.selectedPlaylistIndex);
            playlist.classList.remove('show');
            document.removeEventListener('keydown', this.playlistKeyHandler);
            this.playlistKeyHandler = null;
          } else if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            playlist.classList.remove('show');
            document.removeEventListener('keydown', this.playlistKeyHandler);
            this.playlistKeyHandler = null;
          }
        };

        document.addEventListener('keydown', this.playlistKeyHandler);
      }
    }
  }

  updatePlayButton() {
    const playIcon = document.getElementById('playIcon');
    if (playIcon) {
      if (this.isPlaying) {
        playIcon.innerHTML = '<rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect>';
      } else {
        playIcon.innerHTML = '<polygon points="5 3 19 12 5 21 5 3"></polygon>';
      }
    }
  }

  updateVolumeButton() {
    const volumeIcon = document.getElementById('volumeIcon');
    if (volumeIcon && this.audio) {
      if (this.audio.muted || this.audio.volume === 0) {
        volumeIcon.innerHTML = '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><line x1="23" y1="9" x2="17" y2="15"></line><line x1="17" y1="9" x2="23" y2="15"></line>';
      } else {
        volumeIcon.innerHTML = '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>';
      }
    }
  }

  updateProgress() {
    if (this.audio && isFinite(this.audio.duration) && isFinite(this.audio.currentTime)) {
      const countdownTime = document.querySelector('#countdownTime');
      if (countdownTime) {
        const remaining = this.audio.duration - this.audio.currentTime;
        countdownTime.textContent = '-' + this.formatTime(remaining);
      }
    }
  }

  updateTrackInfo(track) {
    const title = document.querySelector('#musicTitle');
    const artist = document.querySelector('#musicArtist');
    const cover = document.querySelector('#musicCover');

    if (title) {
      title.textContent = this.removeTimestamp(track.title);
    }
    if (artist) {
      artist.textContent = track.artist;
    }
    if (cover) {
      cover.src = track.cover || '/img/avatar.webp';
    }
  }

  updatePlaylistUI() {
    const playlist = document.getElementById('musicPlaylist');
    if (!playlist) return;

    playlist.innerHTML = this.playlist.map((track, index) => {
      const isActive = index === this.currentTrackIndex ? 'active' : '';
      const isSelected = index === this.selectedPlaylistIndex ? 'selected' : '';
      return `
      <div class="music-playlist-item ${isActive} ${isSelected}" data-index="${index}">
        <div class="music-playlist-item-title">${this.removeTimestamp(track.title)}</div>
        <div class="music-playlist-item-duration">${track.duration}</div>
      </div>
    `}).join('');

    // 绑定播放列表点击事件
    playlist.querySelectorAll('.music-playlist-item').forEach(item => {
      item.addEventListener('click', () => {
        const index = parseInt(item.dataset.index);
        this.selectedPlaylistIndex = index;
        this.playTrack(index);
      });
    });
  }

  // 移除文件名中的时间戳和下划线前缀
  removeTimestamp(title) {
    const timestampMatch = title.match(/^\d+_/);
    if (timestampMatch) {
      return title.substring(timestampMatch[0].length);
    }
    return title;
  }

  formatTime(seconds) {
    if (isNaN(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  show() {
    const player = document.getElementById('musicPlayer');
    if (player) {
      player.classList.remove('hidden');
    }
  }

  hide() {
    const player = document.getElementById('musicPlayer');
    if (player) {
      player.classList.add('hidden');
    }
  }

  // 保存播放状态到localStorage
  saveState() {
    try {
      const volumeBar = document.querySelector('#volumeBar');
      const state = {
        currentTrackIndex: this.currentTrackIndex,
        isPlaying: this.isPlaying,
        currentTime: (this.audio && isFinite(this.audio.currentTime)) ? this.audio.currentTime : 0,
        volume: volumeBar ? volumeBar.value : (this.audio ? this.audio.volume * 100 : 80),
        playlist: this.playlist
      };
      localStorage.setItem('musicPlayerState', JSON.stringify(state));
    } catch (error) {
      console.warn('Failed to save music player state:', error);
    }
  }

  // 从localStorage恢复播放状态
  async restoreState() {
    try {
      const stateStr = localStorage.getItem('musicPlayerState');
      if (!stateStr) return false;

      const state = JSON.parse(stateStr);

      // 等待播放列表加载完成
      await this.loadPlaylist();

      // 检查播放列表是否匹配
      if (state.playlist && state.playlist.length > 0 &&
          this.playlist.length > 0 &&
          this.playlist[0].url === state.playlist[0].url) {

        // 恢复播放状态
        this.currentTrackIndex = state.currentTrackIndex || 0;

        // 加载歌曲但不自动播放
        if (this.currentTrackIndex < this.playlist.length) {
          const track = this.playlist[this.currentTrackIndex];
          this.audio.src = track.url;
          // 添加有效性检查
          if (isFinite(state.currentTime)) {
            this.audio.currentTime = state.currentTime;
          }

          // 恢复音量设置
          const volumeBar = document.querySelector('#volumeBar');
          if (volumeBar) {
            // 如果保存了音量，使用保存的值；否则使用默认值
            volumeBar.value = state.volume || 80;
            this.audio.volume = volumeBar.value / 100;
          } else {
            this.audio.volume = state.volume ? state.volume / 100 : 0.8;
          }

          this.updateTrackInfo(track);
          this.updatePlaylistUI();

          // 如果之前在播放，恢复播放
          if (state.isPlaying) {
            this.audio.play().then(() => {
              this.isPlaying = true;
              this.updatePlayButton();
            }).catch(error => {
              console.log('恢复播放失败，等待用户交互:', error.message);
              this.autoPlayPending = true;
            });
          }
        }

        return true;
      }

      return false;
    } catch (error) {
      console.warn('Failed to restore music player state:', error);
      return false;
    }
  }
}

// 创建全局音乐播放器实例
let musicPlayer = null;

// 页面加载时初始化音乐播放器
document.addEventListener('DOMContentLoaded', () => {
  musicPlayer = new MusicPlayer();

  // 导出到全局（必须在初始化之后）
  window.MusicPlayer = MusicPlayer;
  window.musicPlayer = musicPlayer;
});