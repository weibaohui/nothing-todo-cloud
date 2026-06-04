# Nothing Todo Cloud 同步 API 文档

## 概述

ntd-cloud 是 nothing-todo 的云端同步服务器，提供多设备间的数据同步功能。

**Base URL**: `http://localhost:8089`（默认端口）

**认证方式**: JWT Bearer Token

**数据格式**: YAML (Content-Type: `text/yaml`)

---

## 认证流程

### 1. 注册用户

```
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "123456"
}
```

**响应**:
```yaml
success: true
token: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
user_id: 1
```

### 2. 登录

```
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "123456"
}
```

### 3. 创建设备

```
POST /api/devices
Authorization: Bearer <token>
Content-Type: application/json

{
  "device_name": "设备A"
}
```

**响应**:
```yaml
id: 1
device_name: "设备A"
last_seen_at: "2026-06-04T00:00:00.000000+00:00"
created_at: "2026-06-04T00:00:00.000000+00:00"
```

---

## 同步 API

### Push (向上同步/上传)

客户端将本地数据推送到服务器，支持冲突解决模式和 Dry Run 预览。

```
POST /api/v1/sync/push
Authorization: Bearer <token>
Content-Type: text/yaml

device_id: 1
data_type: todos
conflict_mode: rename    # 可选：overwrite|skip|rename
dry_run: true            # 可选：true=预览不执行
data: |
  version: '1.0'
  todos:
    - title: 买菜
      prompt: 买菜
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
```

#### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| device_id | i64 | 是 | 设备ID |
| data_type | string | 是 | 数据类型：`todos`/`tags`/`skills` |
| data | string | 是 | YAML 格式的数据内容 |
| conflict_mode | string | 否 | 冲突解决模式，默认 `overwrite` |
| dry_run | bool | 否 | `true`=预览不执行 |

#### conflict_mode (冲突解决模式)

| 模式 | 说明 | 示例 |
|------|------|------|
| `overwrite` | 客户端数据覆盖服务端数据 | 冲突时用客户端的 |
| `skip` | 保留服务端数据，忽略客户端冲突项 | 冲突时保留服务端的 |
| `rename` | 保留双方，客户端冲突项重命名 | 冲突时生成 `标题 (1)` |

#### 正常响应 (非 Dry Run)

```yaml
success: true
merged_data: |
  version: '1.0'
  created_at: 2026-06-04T00:00:00.000000+00:00
  todos:
    - title: 买菜
      status: pending
      ...
  tags: []
  skills: []
```

#### Dry Run 响应

```yaml
success: true
preview: true                     # 预览标识
conflict_mode: rename             # 当前模式
merged_data: |                    # 预览合并结果
  version: '1.0'
  todos:
    - title: 买菜
    - title: 买菜 (1)
conflicts:                        # 冲突详情列表
  - title: 买菜
    action: rename                # 本次操作
    server_item:                 # 服务端原始数据
      title: 买菜
      status: pending
      prompt: 买菜
      executor: shell
      ...
    client_item:                 # 客户端提交数据
      title: 买菜
      status: completed
      prompt: 买菜
      executor: atomcode
      ...
    new_title: 买菜 (1)          # rename 后新标题
summary:                          # 统计摘要
  total_client_items: 2            # 客户端提交总数
  new_items: 1                    # 新增项（无冲突）
  overwritten: 0                  # 被覆盖数
  skipped: 0                      # 被跳过数
  renamed: 1                      # 被重命名数
  final_total: 3                  # 最终总数
```

---

### Pull (向下同步/拉取)

客户端从服务器拉取同步数据。

```
GET /api/v1/sync/pull?device_id=1&data_type=todos
Authorization: Bearer <token>
```

#### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| device_id | i64 | 是 | 设备ID |
| data_type | string | 否 | 数据类型，默认 `todos` |

#### 响应

```yaml
device_id: 1
data_type: todos
data: |
  version: '1.0'
  created_at: 2026-06-04T00:00:00.000000+00:00
  todos:
    - title: 买菜
      status: pending
      prompt: 买菜
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
  tags: []
  skills: []
updated_at: 2026-06-04T00:00:00.000000+00:00
```

---

### Status (同步状态)

获取设备最后同步时间。

```
GET /api/v1/sync/status?device_id=1&data_type=todos
Authorization: Bearer <token>
```

#### 响应

```yaml
device_id: 1
last_sync_at: "2026-06-04T00:00:00.000000+00:00"
```

---

## 数据格式

### TodoItem 结构

```yaml
title: string                    # 标题（唯一标识）
prompt: string                    # 触发提示词
status: string                   # 状态: pending|completed
executor: string                 # 执行器: shell|atomcode
scheduler_enabled: bool         # 是否启用定时
scheduler_config: string          # cron 表达式
tag_names: []string              # 关联标签
workspace: string|null           # 工作空间路径
worktree: string|null            # git worktree
done: null                       # 兼容字段（废弃）
created_at: null                 # 兼容字段（废弃）
updated_at: null                 # 兼容字段（废弃）
```

### SyncData 结构

```yaml
version: '1.0'                   # 备份格式版本
created_at: '2026-06-04T...'     # 创建时间
todos: []TodoItem                # Todo 列表
tags: []string                   # 标签列表
skills: []string                 # 技能列表
```

---

## 冲突解决策略

### 场景示例

服务器现有：
- 买菜 (status: pending)
- 做饭 (status: pending)

客户端提交：
- 买菜 (status: completed)
- 洗衣服 (新)

#### Overwrite 模式

结果：
- 买菜 (status: completed) ← 被覆盖
- 做饭 (status: pending)
- 洗衣服 (新)

#### Skip 模式

结果：
- 买菜 (status: pending) ← 保留原值
- 做饭 (status: pending)
- 洗衣服 (新)

#### Rename 模式

结果：
- 买菜 (status: pending) ← 保留原值
- 买菜 (1) (status: completed) ← 重命名保留
- 做饭 (status: pending)
- 洗衣服 (新)

---

## Dry Run 预览使用场景

1. **用户同步前预览**：用户想看同步后的结果，不确定会不会丢数据
2. **冲突检查**：同名标题是否会被覆盖/跳过/重命名
3. **决策辅助**：用户根据预览结果选择合适的 conflict_mode

### 使用流程

```bash
# 1. Dry Run 预览（不执行）
curl -X POST /api/v1/sync/push \
  -H "Authorization: Bearer <token>" \
  -d 'device_id: 1
       data_type: todos
       conflict_mode: rename
       dry_run: true
       data: | ...'

# 2. 根据预览结果决定是否执行
curl -X POST /api/v1/sync/push \
  -H "Authorization: Bearer <token>" \
  -d 'device_id: 1
       data_type: todos
       conflict_mode: rename
       dry_run: false
       data: | ...'
```

---

## 错误响应

```yaml
{"error":"设备不存在","success":false}
{"error":"Token 无效或已过期","success":false}
{"error":"YAML解析失败: ...","success":false}
{"error":"缺少认证 Token","success":false}
```

---

## 健康检查

```
GET /health
```

响应：
```json
{"service":"nothing-todo-cloud","status":"ok"}
```
