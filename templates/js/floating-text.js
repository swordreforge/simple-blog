// 飘字效果控制器 - 基于 Canvas 实现
class FloatingTextController {
  constructor() {
    this.canvas = null;
    this.ctx = null;
    this.isEnabled = false;
    this.texts = [];
    this.particles = [];
    this.colors = ['#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6'];
    this.animationId = null;
    this.init();
  }

  init() {
    // 创建 canvas
    this.createCanvas();

    // 从 localStorage 读取设置
    this.loadSettings();

    // 监听设置变化
    this.listenToSettings();

    // 监听窗口大小变化
    window.addEventListener('resize', () => this.handleResize());

    // 监听鼠标点击事件
    this.setupClickHandler();
  }

  createCanvas() {
    this.canvas = document.createElement('canvas');
    this.canvas.id = 'floating-text-canvas';
    // 移除 pointer-events: none，允许 canvas 接收点击事件
    this.canvas.style.cssText = 'position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;';
    this.ctx = this.canvas.getContext('2d');
    document.body.appendChild(this.canvas);
    this.handleResize();
  }

  handleResize() {
    if (this.canvas) {
      this.canvas.width = window.innerWidth;
      this.canvas.height = window.innerHeight;
    }
  }

  setupClickHandler() {
    // 监听整个文档的点击事件
    document.addEventListener('click', (e) => {
      if (!this.isEnabled) {
        return;
      }
      this.createParticle(e.clientX, e.clientY);
    });
  }

  loadSettings() {
    try {
      const settings = localStorage.getItem('appearanceSettings');

      if (settings) {
        const parsed = JSON.parse(settings);
        this.isEnabled = parsed.floating_text_enabled || false;

        // 从设置中读取自定义飘字文本
        if (parsed.floating_texts && Array.isArray(parsed.floating_texts) && parsed.floating_texts.length > 0) {
          this.texts = parsed.floating_texts;
        }
      }
    } catch (e) {
      // 忽略错误
    }

    if (this.isEnabled) {
      this.start();
    }
  }

  listenToSettings() {
    // 监听自定义事件
    window.addEventListener('appearanceSettingsChanged', (e) => {
      if (e.detail) {
        if (e.detail.floating_text_enabled !== undefined) {
          const prevEnabled = this.isEnabled;
          this.isEnabled = e.detail.floating_text_enabled;

          if (this.isEnabled) {
            this.start();
          } else {
            this.stop();
          }
        }

        // 更新飘字文本
        if (e.detail.floating_texts && Array.isArray(e.detail.floating_texts) && e.detail.floating_texts.length > 0) {
          this.texts = e.detail.floating_texts;
        }
      }
    });
  }

  start() {
    if (this.animationId) {
      return;
    }

    // 如果没有设置文本，使用默认值
    if (!this.texts || this.texts.length === 0) {
      this.texts = ['perfect', 'good', 'excellent', 'extraordinary', 'legend'];
    }

    // 开始动画循环
    this.animate();
  }

  stop() {
    // 停止动画循环
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }

    // 清除所有粒子
    this.particles = [];

    // 清除画布
    if (this.ctx && this.canvas) {
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
  }

  createParticle(x, y) {
    if (!this.isEnabled) {
      return;
    }

    const particle = {
      x: x,
      y: y,
      text: this.texts[Math.floor(Math.random() * this.texts.length)],
      color: this.colors[Math.floor(Math.random() * this.colors.length)],
      size: 18 + Math.random() * 6,
      speedY: -0.8 - Math.random() * 0.4, // 慢速向上移动
      life: 1,
      decay: 0.005 + Math.random() * 0.002 // 慢速淡出
    };

    this.particles.push(particle);
  }

  animate() {
    if (!this.ctx || !this.canvas) {
      return;
    }

    // 清除画布
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    // 更新和绘制所有粒子
    for (let i = this.particles.length - 1; i >= 0; i--) {
      const p = this.particles[i];

      // 更新位置 - 向上移动
      p.y += p.speedY;
      p.life -= p.decay;

      // 绘制粒子
      this.ctx.globalAlpha = p.life;
      this.ctx.fillStyle = p.color;
      this.ctx.font = `bold ${p.size}px Arial`;
      this.ctx.textAlign = 'center';
      this.ctx.textBaseline = 'middle';
      this.ctx.fillText(p.text, p.x, p.y);

      // 移除已淡出的粒子
      if (p.life <= 0) {
        this.particles.splice(i, 1);
      }
    }

    // 继续动画循环
    if (this.isEnabled || this.particles.length > 0) {
      this.animationId = requestAnimationFrame(() => this.animate());
    }
  }

  // 公共方法：更新设置
  updateSettings(settings) {
    if (settings.floating_text_enabled !== undefined) {
      this.isEnabled = settings.floating_text_enabled;

      if (this.isEnabled) {
        this.start();
      } else {
        this.stop();
      }
    }

    if (settings.floating_texts && Array.isArray(settings.floating_texts) && settings.floating_texts.length > 0) {
      this.texts = settings.floating_texts;
    }
  }

  // 公共方法：设置自定义文本
  setTexts(texts) {
    this.texts = texts;
  }

  // 公共方法：在指定位置创建飘字
  createAt(x, y) {
    this.createParticle(x, y);
  }
}

// 初始化
let floatingTextController = null;

document.addEventListener('DOMContentLoaded', function() {
  // 延迟初始化，等待外观设置加载
  setTimeout(() => {
    floatingTextController = new FloatingTextController();
  }, 1000);
});

// 导出全局方法
window.floatingTextController = floatingTextController;