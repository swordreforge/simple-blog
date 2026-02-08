/* database.c - SQLite 数据库封装 */
#include "include/database.h"
#include "include/common.h"
#include <sqlite3.h>
#include <string.h>

/* === 初始化数据库 === */

int database_init(sqlite3 **db, const char *path) {
    int rc = sqlite3_open(path, db);
    if (rc != SQLITE_OK) {
        LOG_ERROR("打开数据库失败: %s", sqlite3_errmsg(*db));
        return -1;
    }
    
    /* 设置缓存大小 */
    sqlite3_exec(*db, "PRAGMA cache_size = 2000;", NULL, NULL, NULL);
    sqlite3_exec(*db, "PRAGMA page_size = 1024;", NULL, NULL, NULL);
    sqlite3_exec(*db, "PRAGMA synchronous = NORMAL;", NULL, NULL, NULL);
    
    LOG_INFO("数据库初始化完成: %s", path);
    return 0;
}

/* === 关闭数据库 === */

void database_close(sqlite3 *db) {
    if (db) {
        sqlite3_close(db);
        LOG_INFO("数据库已关闭");
    }
}

/* === 创建数据库表 === */

int database_create_tables(sqlite3 *db) {
    const char *sqls[] = {
        /* 用户表 */
        "CREATE TABLE IF NOT EXISTS users ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "username TEXT UNIQUE NOT NULL,"
        "password_hash TEXT NOT NULL,"
        "email TEXT,"
        "role TEXT DEFAULT 'user',"
        "status TEXT DEFAULT 'active',"
        "created_at INTEGER NOT NULL,"
        "updated_at INTEGER NOT NULL"
        ");",
        
        /* 文章表 */
        "CREATE TABLE IF NOT EXISTS passages ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "uuid TEXT UNIQUE NOT NULL,"
        "title TEXT NOT NULL,"
        "content TEXT NOT NULL,"
        "original_content TEXT,"
        "summary TEXT,"
        "author TEXT DEFAULT 'Anonymous',"
        "tags TEXT DEFAULT '[]',"
        "category TEXT DEFAULT '未分类',"
        "status TEXT DEFAULT 'draft',"
        "file_path TEXT,"
        "visibility TEXT DEFAULT 'public',"
        "is_scheduled INTEGER DEFAULT 0,"
        "published_at INTEGER,"
        "cover_image TEXT,"
        "created_at INTEGER NOT NULL,"
        "updated_at INTEGER NOT NULL"
        ");",
        
        /* 评论表 */
        "CREATE TABLE IF NOT EXISTS comments ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "username TEXT NOT NULL,"
        "content TEXT NOT NULL,"
        "passage_uuid TEXT NOT NULL,"
        "created_at INTEGER NOT NULL,"
        "FOREIGN KEY(passage_uuid) REFERENCES passages(uuid)"
        ");",
        
        /* 分类表 */
        "CREATE TABLE IF NOT EXISTS categories ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "name TEXT UNIQUE NOT NULL,"
        "description TEXT,"
        "icon TEXT DEFAULT '📁',"
        "sort_order INTEGER DEFAULT 0,"
        "is_enabled INTEGER DEFAULT 1,"
        "created_at INTEGER NOT NULL,"
        "updated_at INTEGER NOT NULL"
        ");",
        
        /* 标签表 */
        "CREATE TABLE IF NOT EXISTS tags ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "name TEXT UNIQUE NOT NULL,"
        "description TEXT,"
        "color TEXT DEFAULT '#007bff',"
        "category_id INTEGER DEFAULT 0,"
        "sort_order INTEGER DEFAULT 0,"
        "is_enabled INTEGER DEFAULT 1,"
        "created_at INTEGER NOT NULL,"
        "updated_at INTEGER NOT NULL"
        ");",
        
        /* 友链表 */
        "CREATE TABLE IF NOT EXISTS friend_links ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "nickname TEXT NOT NULL,"
        "link_url TEXT NOT NULL,"
        "avatar_url TEXT,"
        "motto TEXT,"
        "sort_order INTEGER DEFAULT 0,"
        "is_enabled INTEGER DEFAULT 1,"
        "created_at INTEGER NOT NULL,"
        "updated_at INTEGER NOT NULL"
        ");",
        
        /* 音乐表 */
        "CREATE TABLE IF NOT EXISTS music ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "title TEXT NOT NULL,"
        "artist TEXT,"
        "file_path TEXT NOT NULL,"
        "file_name TEXT NOT NULL,"
        "duration TEXT,"
        "cover_image TEXT,"
        "created_at INTEGER NOT NULL"
        ");",
        
        /* 设置表 */
        "CREATE TABLE IF NOT EXISTS settings ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "key TEXT UNIQUE NOT NULL,"
        "value TEXT NOT NULL,"
        "type TEXT DEFAULT 'string',"
        "description TEXT,"
        "category TEXT DEFAULT 'general',"
        "created_at INTEGER NOT NULL,"
        "updated_at INTEGER NOT NULL"
        ");",
        
        NULL
    };
    
    for (int i = 0; sqls[i]; i++) {
        char *err_msg = NULL;
        int rc = sqlite3_exec(db, sqls[i], NULL, NULL, &err_msg);
        if (rc != SQLITE_OK) {
            LOG_ERROR("创建表失败: %s", err_msg);
            sqlite3_free(err_msg);
            return -1;
        }
    }
    
    LOG_INFO("数据库表创建完成");
    return 0;
}

/* === 插入默认数据 === */

int database_insert_default_data(sqlite3 *db) {
    /* 检查是否已有管理员用户 */
    sqlite3_stmt *stmt;
    const char *sql = "SELECT COUNT(*) FROM users WHERE role = 'admin';";
    
    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        LOG_ERROR("准备查询失败: %s", sqlite3_errmsg(db));
        return -1;
    }
    
    int admin_count = 0;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        admin_count = sqlite3_column_int(stmt, 0);
    }
    sqlite3_finalize(stmt);
    
    /* 如果没有管理员，创建默认管理员 */
    if (admin_count == 0) {
        LOG_INFO("创建默认管理员用户...");
        const char *insert_sql = "INSERT INTO users (username, password_hash, email, role, created_at, updated_at) "
                                 "VALUES ('admin', '$argon2id$v=19$m=65536,t=3,p=4$...', "
                                 "'admin@rustblog.local', 'admin', ?, ?);";
        
        int64_t now = NOW();
        if (sqlite3_prepare_v2(db, insert_sql, -1, &stmt, NULL) != SQLITE_OK) {
            LOG_ERROR("准备插入失败: %s", sqlite3_errmsg(db));
            return -1;
        }
        
        sqlite3_bind_int64(stmt, 1, now);
        sqlite3_bind_int64(stmt, 2, now);
        
        if (sqlite3_step(stmt) != SQLITE_DONE) {
            LOG_ERROR("插入管理员失败: %s", sqlite3_errmsg(db));
            sqlite3_finalize(stmt);
            return -1;
        }
        sqlite3_finalize(stmt);
        
        LOG_INFO("默认管理员已创建 (用户名: admin, 密码: 请手动设置)");
    }
    
    /* 创建默认分类 */
    const char *default_categories[] = {
        "INSERT OR IGNORE INTO categories (name, description, icon, created_at, updated_at) "
        "VALUES ('技术', '技术相关文章', '💻', ?, ?);",
        "INSERT OR IGNORE INTO categories (name, description, icon, created_at, updated_at) "
        "VALUES ('生活', '生活随笔', '🌸', ?, ?);",
        "INSERT OR IGNORE INTO categories (name, description, icon, created_at, updated_at) "
        "VALUES ('笔记', '学习笔记', '📝', ?, ?);",
        NULL
    };
    
    int64_t now = NOW();
    for (int i = 0; default_categories[i]; i++) {
        char *err_msg = NULL;
        if (sqlite3_prepare_v2(db, default_categories[i], -1, &stmt, NULL) != SQLITE_OK) {
            continue;
        }
        sqlite3_bind_int64(stmt, 1, now);
        sqlite3_bind_int64(stmt, 2, now);
        sqlite3_step(stmt);
        sqlite3_finalize(stmt);
    }
    
    LOG_INFO("默认数据插入完成");
    return 0;
}

/* === Passage 操作（简化实现） === */

int passage_create(sqlite3 *db, const Passage *passage, int *id) {
    const char *sql = "INSERT INTO passages (uuid, title, content, original_content, summary, "
                     "author, tags, category, status, file_path, visibility, is_scheduled, "
                     "published_at, cover_image, created_at, updated_at) "
                     "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);";
    
    sqlite3_stmt *stmt;
    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        return -1;
    }
    
    sqlite3_bind_text(stmt, 1, passage->uuid, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 2, passage->title, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 3, passage->content, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 4, passage->html_content, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 5, passage->summary, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 6, passage->author, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 7, passage->tags, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 8, passage->category, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 9, passage->status, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 10, passage->file_path, -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 11, passage->visibility, -1, SQLITE_STATIC);
    sqlite3_bind_int(stmt, 12, passage->is_scheduled ? 1 : 0);
    sqlite3_bind_int64(stmt, 13, passage->published_at);
    sqlite3_bind_text(stmt, 14, passage->cover_image, -1, SQLITE_STATIC);
    sqlite3_bind_int64(stmt, 15, passage->created_at);
    sqlite3_bind_int64(stmt, 16, passage->updated_at);
    
    int rc = sqlite3_step(stmt);
    sqlite3_finalize(stmt);
    
    if (rc == SQLITE_DONE) {
        *id = (int)sqlite3_last_insert_rowid(db);
        return 0;
    }
    return -1;
}

/* === User 操作（简化实现） === */

int user_verify_password(sqlite3 *db, const char *username, const char *password, User *user) {
    /* TODO: 实现密码验证 */
    return -1;
}

/* === 其他操作（占位实现） === */

int passage_get_by_id(sqlite3 *db, int id, Passage *passage) { return -1; }
int passage_get_by_uuid(sqlite3 *db, const char *uuid, Passage *passage) { return -1; }
int passage_get_list(sqlite3 *db, Passage **passages, int *count, int limit, int offset) { return -1; }

int passage_get_published(sqlite3 *db, Passage **passages, int *count, int limit, int offset) {
    if (!db || !passages || !count) return -1;

    const char *sql = "SELECT id, uuid, title, content, html_content, summary, author, tags, "
                      "category, status, file_path, visibility, is_scheduled, published_at, "
                      "cover_image, created_at, updated_at "
                      "FROM passages WHERE status = 'published' AND visibility = 'public' "
                      "ORDER BY published_at DESC LIMIT ? OFFSET ?";

    sqlite3_stmt *stmt;
    int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        LOG_ERROR("准备查询失败: %s", sqlite3_errmsg(db));
        return -1;
    }

    sqlite3_bind_int(stmt, 1, limit);
    sqlite3_bind_int(stmt, 2, offset);

    /* 先查询结果数量 */
    int result_count = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        result_count++;
    }
    sqlite3_reset(stmt);

    if (result_count == 0) {
        *passages = NULL;
        *count = 0;
        sqlite3_finalize(stmt);
        return 0;
    }

    /* 分配内存 */
    *passages = (Passage*)calloc(result_count, sizeof(Passage));
    if (!*passages) {
        LOG_ERROR("内存分配失败");
        sqlite3_finalize(stmt);
        return -1;
    }

    /* 重新执行查询获取数据 */
    int idx = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW && idx < result_count) {
        Passage *p = &(*passages)[idx];
        memset(p, 0, sizeof(Passage));
        p->id = sqlite3_column_int(stmt, 0);

        const char *text = (const char*)sqlite3_column_text(stmt, 1);
        if (text) strncpy(p->uuid, text, sizeof(p->uuid) - 1);

        text = (const char*)sqlite3_column_text(stmt, 2);
        if (text) strncpy(p->title, text, sizeof(p->title) - 1);

        text = (const char*)sqlite3_column_text(stmt, 3);
        if (text) strncpy(p->content, text, sizeof(p->content) - 1);

        text = (const char*)sqlite3_column_text(stmt, 4);
        if (text) strncpy(p->html_content, text, sizeof(p->html_content) - 1);

        text = (const char*)sqlite3_column_text(stmt, 5);
        if (text) strncpy(p->summary, text, sizeof(p->summary) - 1);

        text = (const char*)sqlite3_column_text(stmt, 6);
        if (text) strncpy(p->author, text, sizeof(p->author) - 1);

        text = (const char*)sqlite3_column_text(stmt, 7);
        if (text) strncpy(p->tags, text, sizeof(p->tags) - 1);

        text = (const char*)sqlite3_column_text(stmt, 8);
        if (text) strncpy(p->category, text, sizeof(p->category) - 1);

        text = (const char*)sqlite3_column_text(stmt, 9);
        if (text) strncpy(p->status, text, sizeof(p->status) - 1);

        text = (const char*)sqlite3_column_text(stmt, 10);
        if (text) strncpy(p->file_path, text, sizeof(p->file_path) - 1);

        text = (const char*)sqlite3_column_text(stmt, 11);
        if (text) strncpy(p->visibility, text, sizeof(p->visibility) - 1);

        p->is_scheduled = sqlite3_column_int(stmt, 12) != 0;
        p->published_at = sqlite3_column_int64(stmt, 13);

        text = (const char*)sqlite3_column_text(stmt, 14);
        if (text) strncpy(p->cover_image, text, sizeof(p->cover_image) - 1);

        p->created_at = sqlite3_column_int64(stmt, 15);
        p->updated_at = sqlite3_column_int64(stmt, 16);
        idx++;
    }

    *count = idx;
    sqlite3_finalize(stmt);
    return 0;
}

int passage_update(sqlite3 *db, const Passage *passage) { return -1; }
int passage_delete_by_id(sqlite3 *db, int id) { return -1; }
int passage_delete_by_uuid(sqlite3 *db, const char *uuid) { return -1; }
int passage_delete_batch(sqlite3 *db, const int *ids, int count) { return -1; }
int passage_count(sqlite3 *db, int *count) { return -1; }

int passage_count_published(sqlite3 *db, int *count) {
    if (!db || !count) return -1;

    const char *sql = "SELECT COUNT(*) FROM passages WHERE status = 'published' AND visibility = 'public'";

    sqlite3_stmt *stmt;
    int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        LOG_ERROR("准备查询失败: %s", sqlite3_errmsg(db));
        return -1;
    }

    if (sqlite3_step(stmt) == SQLITE_ROW) {
        *count = sqlite3_column_int(stmt, 0);
    } else {
        *count = 0;
    }

    sqlite3_finalize(stmt);
    return 0;
}

int user_create(sqlite3 *db, const User *user, int *id) { return -1; }
int user_get_by_id(sqlite3 *db, int id, User *user) { return -1; }
int user_get_by_username(sqlite3 *db, const char *username, User *user) { return -1; }
int user_get_list(sqlite3 *db, User **users, int *count, int limit, int offset) { return -1; }
int user_update(sqlite3 *db, const User *user) { return -1; }
int user_delete_by_id(sqlite3 *db, int id) { return -1; }
int user_update_password(sqlite3 *db, int user_id, const char *new_password) { return -1; }

int comment_create(sqlite3 *db, const Comment *comment, int *id) { return -1; }
int comment_get_by_id(sqlite3 *db, int id, Comment *comment) { return -1; }
int comment_get_by_passage_uuid(sqlite3 *db, const char *passage_uuid, Comment **comments, int *count) { return -1; }
int comment_get_list(sqlite3 *db, Comment **comments, int *count, int limit, int offset) { return -1; }
int comment_delete_by_id(sqlite3 *db, int id) { return -1; }

int category_create(sqlite3 *db, const Category *category, int *id) { return -1; }
int category_get_by_id(sqlite3 *db, int id, Category *category) { return -1; }
int category_get_by_name(sqlite3 *db, const char *name, Category *category) { return -1; }
int category_get_all(sqlite3 *db, Category **categories, int *count) { return -1; }
int category_update(sqlite3 *db, const Category *category) { return -1; }
int category_delete_by_id(sqlite3 *db, int id) { return -1; }

int tag_create(sqlite3 *db, const Tag *tag, int *id) { return -1; }
int tag_get_by_id(sqlite3 *db, int id, Tag *tag) { return -1; }
int tag_get_by_name(sqlite3 *db, const char *name, Tag *tag) { return -1; }
int tag_get_all(sqlite3 *db, Tag **tags, int *count) { return -1; }
int tag_update(sqlite3 *db, const Tag *tag) { return -1; }
int tag_delete_by_id(sqlite3 *db, int id) { return -1; }

int friend_link_create(sqlite3 *db, const FriendLink *link, int *id) { return -1; }
int friend_link_get_by_id(sqlite3 *db, int id, FriendLink *link) { return -1; }
int friend_link_get_all(sqlite3 *db, FriendLink **links, int *count) { return -1; }
int friend_link_update(sqlite3 *db, const FriendLink *link) { return -1; }
int friend_link_delete_by_id(sqlite3 *db, int id) { return -1; }

int music_create(sqlite3 *db, const MusicTrack *track, int *id) { return -1; }
int music_get_by_id(sqlite3 *db, int id, MusicTrack *track) { return -1; }
int music_get_all(sqlite3 *db, MusicTrack **tracks, int *count) { return -1; }
int music_update(sqlite3 *db, const MusicTrack *track) { return -1; }
int music_delete_by_id(sqlite3 *db, int id) { return -1; }

int database_begin_transaction(sqlite3 *db) { return -1; }
int database_commit(sqlite3 *db) { return -1; }
int database_rollback(sqlite3 *db) { return -1; }
int database_execute(sqlite3 *db, const char *sql) { return -1; }
int64_t database_last_insert_id(sqlite3 *db) { return 0; }