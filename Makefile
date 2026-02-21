.PHONY: all build build-glibc build-musl clean release release-glibc release-musl test

# 默认构建glibc版本
all: build-glibc

# 构建调试版本
build: build-glibc

build-glibc:
	cargo build

build-musl:
	cargo build --target x86_64-unknown-linux-musl

# 清理
clean:
	cargo clean

# 发布版本
release: release-glibc

release-glibc:
	cargo build --release
	@echo "压缩二进制文件..."
	upx --best --lzma target/release/staticwallpaper
	@echo "✅ glibc版本构建完成: target/release/staticwallpaper"
	@ls -lh target/release/staticwallpaper

release-musl:
	cargo build --release --target x86_64-unknown-linux-musl
	@echo "压缩二进制文件..."
	upx --best --lzma target/x86_64-unknown-linux-musl/release/staticwallpaper
	@echo "✅ musl版本构建完成: target/x86_64-unknown-linux-musl/release/staticwallpaper"
	@ls -lh target/x86_64-unknown-linux-musl/release/staticwallpaper

# 构建所有版本
release-all: release-glibc release-musl
	@echo ""
	@echo "=== 构建摘要 ==="
	@echo "glibc版本: $(ls -lh target/release/staticwallpaper | awk '{print $$5}')"
	@echo "musl版本: $(ls -lh target/x86_64-unknown-linux-musl/release/staticwallpaper | awk '{print $$5}')"

# 测试
test:
	cargo test

# 检查
check:
	cargo check

# 格式化
fmt:
	cargo fmt

# 安装依赖
deps:
	cargo fetch

# 显示帮助
help:
	@echo "可用的命令:"
	@echo "  make              - 构建glibc调试版本"
	@echo "  make build        - 构建glibc调试版本"
	@echo "  make build-glibc  - 构建glibc调试版本"
	@echo "  make build-musl   - 构建musl调试版本"
	@echo "  make release      - 构建glibc发布版本(带UPX压缩)"
	@echo "  make release-glibc- 构建glibc发布版本(带UPX压缩)"
	@echo "  make release-musl  - 构建musl发布版本(带UPX压缩)"
	@echo "  make release-all   - 构建所有发布版本"
	@echo "  make clean        - 清理构建文件"
	@echo "  make test         - 运行测试"
	@echo "  make check        - 检查代码"
	@echo "  make fmt          - 格式化代码"
	@echo "  make deps         - 下载依赖"
	@echo "  make help         - 显示此帮助"