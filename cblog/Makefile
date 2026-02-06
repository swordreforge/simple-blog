# Makefile - RustBlog C语言重构版本
# 使用 BearSSL + select 模型，目标 250-500KB
CC = gcc
AR = ar
STRIP = strip

# 优化选项
CFLAGS = -Os -fdata-sections -ffunction-sections -fno-stack-protector \
         -fno-unwind-tables -fno-asynchronous-unwind-tables -flto \
         -Wall -Wextra -Wno-unused-parameter

LDFLAGS = -Wl,--gc-sections -Wl,--strip-all -static -lc -lm -lpthread

# BearSSL 配置
BEARSSL_DIR = bearssl
BEARSSL_CONFIG = -DBR_USE_UNIX_TIME=1 \
                 -DBR_HAVE_GETENTROPY=1 \
                 -DBR_SSL_BR_ONLY=1 \
                 -DBR_SHA256_SMALL=1 \
                 -DBR_ARM_SHA2_NEON=0

# BearSSL 静态库
BEARSSL_LIB = $(BEARSSL_DIR)/build/libbearssl.a

# SQLite 精简配置
SQLITE_FLAGS = -DSQLITE_THREADSAFE=0 \
               -DSQLITE_OMIT_AUTHORIZATION \
               -DSQLITE_OMIT_DECLTYPE \
               -DSQLITE_OMIT_DEPRECATED \
               -DSQLITE_OMIT_EXPLAIN \
               -DSQLITE_OMIT_GET_TABLE \
               -DSQLITE_OMIT_INCRBLOB \
               -DSQLITE_OMIT_LOAD_EXTENSION \
               -DSQLITE_OMIT_PROGRESS_CALLBACK \
               -DSQLITE_OMIT_SHARED_CACHE \
               -DSQLITE_OMIT_TRACE \
               -DSQLITE_OMIT_UTF16 \
               -DSQLITE_OMIT_WAL \
               -DSQLITE_DEFAULT_MEMSTATUS=0 \
               -DSQLITE_MAX_ATTACHED=2 \
               -DSQLITE_DEFAULT_CACHE_SIZE=2000 \
               -DSQLITE_DEFAULT_PAGE_SIZE=1024 \
               -DSQLITE_MAX_COLUMN=32 \
               -DSQLITE_MAX_COMPOUND_SELECT=4 \
               -DSQLITE_MAX_FUNCTION_ARG=8 \
               -DSQLITE_MAX_LENGTH=8192 \
               -DSQLITE_MAX_SQL_LENGTH=4096 \
               -DSQLITE_MAX_VARIABLE_NUMBER=20

# 项目结构
SRC_DIR = src
BUILD_DIR = build
BIN_DIR = bin

# 源文件
SOURCES = $(SRC_DIR)/main.c \
          $(SRC_DIR)/server.c \
          $(SRC_DIR)/ssl.c \
          $(SRC_DIR)/http.c \
          $(SRC_DIR)/router.c \
          $(SRC_DIR)/database.c \
          $(SRC_DIR)/template.c \
          $(SRC_DIR)/crypto.c \
          $(SRC_DIR)/jwt.c \
          $(SRC_DIR)/common.c \
          $(SRC_DIR)/handlers/passage.c \
          $(SRC_DIR)/handlers/user.c \
          $(SRC_DIR)/handlers/auth.c \
          $(SRC_DIR)/handlers/music.c \
          $(SRC_DIR)/handlers/comment.c \
          $(SRC_DIR)/handlers/file.c \
          $(SRC_DIR)/utils/string.c \
          $(SRC_DIR)/utils/file.c \
          $(SRC_DIR)/utils/markdown.c

# BearSSL 源文件（使用 BearSSL 提供的构建脚本）
BEARSSL_SOURCES = $(wildcard $(BEARSSL_DIR)/src/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/ssl/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/hash/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/rand/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/aead/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/symmetric/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/rsa/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/ec/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/x509/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/codec/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/encoding/*.c) \
                  $(wildcard $(BEARSSL_DIR)/src/objects/*.c)

# 目标文件
OBJECTS = $(patsubst $(SRC_DIR)/%.c,$(BUILD_DIR)/%.o,$(SOURCES))

# 可执行文件
TARGET = $(BIN_DIR)/cblog

# SQLite 合并源文件
SQLITE_AMALGAMATION = sqlite3.c

# 默认目标
all: dirs $(TARGET)

# 创建必要的目录
dirs:
	@mkdir -p $(BUILD_DIR)/handlers $(BUILD_DIR)/utils $(BUILD_DIR)/bearssl
	@mkdir -p $(BIN_DIR)
	@mkdir -p data
	@mkdir -p templates
	@mkdir -p static

# 编译 SQLite
sqlite3.o: sqlite3.c
	@echo "编译 SQLite..."
	@$(CC) $(CFLAGS) $(SQLITE_FLAGS) -c sqlite3.c -o sqlite3.o

# 编译主程序
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
	@echo "编译 $<..."
	@$(CC) $(CFLAGS) $(SQLITE_FLAGS) $(BEARSSL_CONFIG) \
		-I$(SRC_DIR)/include \
		-I$(BEARSSL_DIR)/inc \
		-I$(BEARSSL_DIR)/src \
		-c $< -o $@

# 编译 BearSSL 库（如果不存在）
$(BEARSSL_LIB):
	@echo "编译 BearSSL 库..."
	@$(MAKE) -C $(BEARSSL_DIR) CFLAGS="-Os -fPIC"

# 链接
$(TARGET): $(OBJECTS) sqlite3.o $(BEARSSL_LIB)
	@echo "链接 $@..."
	@$(CC) $(OBJECTS) sqlite3.o \
		$(LDFLAGS) $(BEARSSL_LIB) -o $@
	@$(STRIP) --strip-all $@
	@echo "✅ 编译完成！"
	@echo "二进制大小: $$(stat -c%s $@ | numfmt --to=si) ($$(stat -c%s $@) bytes)"
	@echo "预期大小: 250-500KB"

# 克隆 BearSSL
bearssl:
	@echo "克隆 BearSSL..."
	@if [ ! -d "bearssl" ]; then \
		git clone --depth 1 https://www.bearssl.org/git/BearSSL bearssl; \
	fi

# 生成测试证书
certs:
	@echo "生成自签名证书..."
	@openssl req -x509 -newkey rsa:2048 -keyout data/key.pem -out data/cert.pem \
		-days 365 -nodes -subj "/C=US/ST=State/L=City/O=RustBlog/CN=localhost"
	@openssl x509 -in data/cert.pem -outform der -out data/cert.der
	@openssl rsa -in data/key.pem -outform der -out data/key.der

# 初始化数据库
init-db: $(TARGET)
	@echo "初始化数据库..."
	@$(TARGET) --init-db

# 清理
clean:
	@echo "清理构建文件..."
	@rm -rf $(BUILD_DIR) $(BIN_DIR) sqlite3.c

# 深度清理
distclean: clean
	@rm -rf bearssl data/*.db data/*.der data/*.pem

# 运行
run: $(TARGET)
	@echo "启动服务器..."
	@$(TARGET)

# 调试模式
debug: CFLAGS += -g -DDEBUG
debug: clean $(TARGET)

# 分析大小
size: $(TARGET)
	@echo "=== 二进制大小分析 ==="
	@echo "总大小: $$(stat -c%s $@ | numfmt --to=si)"
	@echo ""
	@echo "=== 段大小 ==="
	@size $@
	@echo ""
	@echo "=== 符号表 ==="
	@nm --size-sort $@ | head -20

.PHONY: all dirs sqlite3.c certs init-db clean distclean run debug size bearssl