#ifndef DATABASE_H
#define DATABASE_H

#include "types.h"

/* SQLite 前向声明 */
typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

/* === 数据库初始化 === */

/**
 * 初始化数据库
 */
int database_init(sqlite3 **db, const char *path);

/**
 * 关闭数据库
 */
void database_close(sqlite3 *db);

/**
 * 初始化数据库表结构
 */
int database_create_tables(sqlite3 *db);

/**
 * 初始化默认数据
 */
int database_insert_default_data(sqlite3 *db);

/* === Passage 操作 === */

/**
 * 创建文章
 */
int passage_create(sqlite3 *db, const Passage *passage, int *id);

/**
 * 根据 ID 获取文章
 */
int passage_get_by_id(sqlite3 *db, int id, Passage *passage);

/**
 * 根据 UUID 获取文章
 */
int passage_get_by_uuid(sqlite3 *db, const char *uuid, Passage *passage);

/**
 * 获取文章列表
 */
int passage_get_list(sqlite3 *db, Passage **passages, int *count, int limit, int offset);

/**
 * 获取已发布文章列表
 */
int passage_get_published(sqlite3 *db, Passage **passages, int *count, int limit, int offset);

/**
 * 更新文章
 */
int passage_update(sqlite3 *db, const Passage *passage);

/**
 * 删除文章（根据 ID）
 */
int passage_delete_by_id(sqlite3 *db, int id);

/**
 * 删除文章（根据 UUID）
 */
int passage_delete_by_uuid(sqlite3 *db, const char *uuid);

/**
 * 批量删除文章
 */
int passage_delete_batch(sqlite3 *db, const int *ids, int count);

/**
 * 获取文章总数
 */
int passage_count(sqlite3 *db, int *count);

/**
 * 获取已发布文章总数
 */
int passage_count_published(sqlite3 *db, int *count);

/* === User 操作 === */

/**
 * 创建用户
 */
int user_create(sqlite3 *db, const User *user, int *id);

/**
 * 根据 ID 获取用户
 */
int user_get_by_id(sqlite3 *db, int id, User *user);

/**
 * 根据用户名获取用户
 */
int user_get_by_username(sqlite3 *db, const char *username, User *user);

/**
 * 获取用户列表
 */
int user_get_list(sqlite3 *db, User **users, int *count, int limit, int offset);

/**
 * 更新用户
 */
int user_update(sqlite3 *db, const User *user);

/**
 * 删除用户
 */
int user_delete_by_id(sqlite3 *db, int id);

/**
 * 验证用户密码
 */
int user_verify_password(sqlite3 *db, const char *username, const char *password, User *user);

/**
 * 更新用户密码
 */
int user_update_password(sqlite3 *db, int user_id, const char *new_password);

/* === Comment 操作 === */

/**
 * 创建评论
 */
int comment_create(sqlite3 *db, const Comment *comment, int *id);

/**
 * 根据 ID 获取评论
 */
int comment_get_by_id(sqlite3 *db, int id, Comment *comment);

/**
 * 获取文章的评论列表
 */
int comment_get_by_passage_uuid(sqlite3 *db, const char *passage_uuid, Comment **comments, int *count);

/**
 * 获取所有评论列表
 */
int comment_get_list(sqlite3 *db, Comment **comments, int *count, int limit, int offset);

/**
 * 删除评论
 */
int comment_delete_by_id(sqlite3 *db, int id);

/* === Category 操作 === */

/**
 * 创建分类
 */
int category_create(sqlite3 *db, const Category *category, int *id);

/**
 * 根据 ID 获取分类
 */
int category_get_by_id(sqlite3 *db, int id, Category *category);

/**
 * 根据名称获取分类
 */
int category_get_by_name(sqlite3 *db, const char *name, Category *category);

/**
 * 获取所有分类
 */
int category_get_all(sqlite3 *db, Category **categories, int *count);

/**
 * 更新分类
 */
int category_update(sqlite3 *db, const Category *category);

/**
 * 删除分类
 */
int category_delete_by_id(sqlite3 *db, int id);

/* === Tag 操作 === */

/**
 * 创建标签
 */
int tag_create(sqlite3 *db, const Tag *tag, int *id);

/**
 * 根据 ID 获取标签
 */
int tag_get_by_id(sqlite3 *db, int id, Tag *tag);

/**
 * 根据名称获取标签
 */
int tag_get_by_name(sqlite3 *db, const char *name, Tag *tag);

/**
 * 获取所有标签
 */
int tag_get_all(sqlite3 *db, Tag **tags, int *count);

/**
 * 更新标签
 */
int tag_update(sqlite3 *db, const Tag *tag);

/**
 * 删除标签
 */
int tag_delete_by_id(sqlite3 *db, int id);

/* === FriendLink 操作 === */

/**
 * 创建友链
 */
int friend_link_create(sqlite3 *db, const FriendLink *link, int *id);

/**
 * 根据 ID 获取友链
 */
int friend_link_get_by_id(sqlite3 *db, int id, FriendLink *link);

/**
 * 获取所有友链
 */
int friend_link_get_all(sqlite3 *db, FriendLink **links, int *count);

/**
 * 更新友链
 */
int friend_link_update(sqlite3 *db, const FriendLink *link);

/**
 * 删除友链
 */
int friend_link_delete_by_id(sqlite3 *db, int id);

/* === Music 操作 === */

/**
 * 创建音乐
 */
int music_create(sqlite3 *db, const MusicTrack *track, int *id);

/**
 * 根据 ID 获取音乐
 */
int music_get_by_id(sqlite3 *db, int id, MusicTrack *track);

/**
 * 获取所有音乐
 */
int music_get_all(sqlite3 *db, MusicTrack **tracks, int *count);

/**
 * 更新音乐
 */
int music_update(sqlite3 *db, const MusicTrack *track);

/**
 * 删除音乐
 */
int music_delete_by_id(sqlite3 *db, int id);

/* === 工具函数 === */

/**
 * 开始事务
 */
int database_begin_transaction(sqlite3 *db);

/**
 * 提交事务
 */
int database_commit(sqlite3 *db);

/**
 * 回滚事务
 */
int database_rollback(sqlite3 *db);

/**
 * 执行 SQL 语句
 */
int database_execute(sqlite3 *db, const char *sql);

/**
 * 获取最后插入的 ID
 */
int64_t database_last_insert_id(sqlite3 *db);

#endif /* DATABASE_H */