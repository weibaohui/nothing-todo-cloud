# Nothing Todo Cloud 同步 API 文档

## 概述

ntd-cloud 提供多设备数据同步服务，采用用户级数据合并策略。

**Base URL**: `http://localhost:8089`

**认证方式**:
- JWT Token (登录用)
- 同步 Token (设备专用，格式 `ntd_xxx`)

**数据格式**: YAML (Content-Type: `text/yaml`)

---

## 认证流程

### 1. 注册

```
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "123456"
}
```

**响应**:
```json
{"success":true,"token":"eyJhbGci...","user_id":1}
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

### 3. 创建同步 Token (设备认证)

```
POST /api/tokens
Authorization: Bearer <JWT>
Content-Type: application/json

{
  "name": "我的手机"
}
```

**响应**:
```json
{
  "id": 1,
  "name": "我的手机",
  "token": "ntd_252f72e5-ee77-4f48-8f6a-32b9bae754cb",
  "last_used_at": null,
  "created_at": "2026-06-04T03:44:23.062710+00:00"
}
```

**重要**: `token` 字段仅在创建时返回，请妥善保存！

### 4. 同步 Token 列表

```
GET /api/tokens
Authorization: Bearer <JWT>
```

### 5. 撤销 Token

```
DELETE /api/tokens/:id
Authorization: Bearer <JWT>
```

---

## 同步 API

所有同步 API 使用**同步 Token** 认证，无需 JWT。

### Push (上传数据)

```
POST /api/v1/sync/push
Authorization: Bearer <同步Token>
Content-Type: text/yaml

data_type: todos
conflict_mode: rename
dry_run: false
data: |
  version: '1.0'
  todos:
    - title: 买菜
      status: pending
      prompt: 买菜
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
```

#### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| data_type | string | 是 | 数据类型：`todos`/`tags`/`skills` |
| data | string | 是 | YAML 格式的数据内容 |
| conflict_mode | string | 否 | 冲突解决模式，默认 `overwrite` |
| dry_run | bool | 否 | `true`=预览不执行 |

#### conflict_mode (冲突解决模式)

| 模式 | 说明 |
|------|------|
| `overwrite` | 客户端覆盖服务端 (默认) |
| `skip` | 保留服务端，忽略冲突项 |
| `rename` | 保留双方，冲突项重命名 |

#### 响应

```yaml
success: true
merged_data: |
  version: '1.0'
  created_at: 2026-06-04T03:44:23.076176+00:00
  todos:
    - title: 买菜
      ...
  tags: []
  skills: []
```

---

### Pull (下载数据)

```
GET /api/v1/sync/pull?data_type=todos
Authorization: Bearer <同步Token>
```

#### 响应

```yaml
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 买菜
      ...
  tags: []
  skills: []
updated_at: 2026-06-04T03:44:23.075848+00:00
```

---

### Status (同步状态)

```
GET /api/v1/sync/status?data_type=todos
Authorization: Bearer <同步Token>
```

#### 响应

```yaml
last_sync_at: "2026-06-04T03:44:23.075848+00:00"
```

---

## Dry Run 预览

使用 `dry_run: true` 预览合并结果，不实际执行。

```
POST /api/v1/sync/push
Authorization: Bearer <同步Token>
Content-Type: text/yaml

data_type: todos
conflict_mode: rename
dry_run: true
data: |
  version: '1.0'
  todos:
    - title: 买菜
      status: completed
```

#### 响应

```yaml
success: true
preview: true
conflict_mode: rename
merged_data: |
  ...
conflicts:
  - title: 买菜
    action: rename
    server_item:
      title: 买菜
      status: pending
      ...
    client_item:
      title: 买菜
      status: completed
      ...
    new_title: 买菜 (1)
summary:
  total_client_items: 1
  new_items: 0
  overwritten: 0
  skipped: 0
  renamed: 1
  final_total: 2
```

---

## 数据格式

### TodoItem

```yaml
title: string
prompt: string
status: pending|completed
executor: shell|atomcode
scheduler_enabled: bool
scheduler_config: "0 0 * * * *"
tag_names: [tag1, tag2]
workspace: /path/to/workspace
worktree: branch-name
done: null (废弃)
created_at: null (废弃)
updated_at: null (废弃)
```

### SyncData

```yaml
version: '1.0'
created_at: '2026-06-04T...'
todos: [TodoItem, ...]
tags: [string, ...]
skills: [string, ...]
```

---

## 冲突解决示例

服务器现有:
- 买菜 (pending)
- 做饭 (pending)

客户端提交:
- 买菜 (completed)
- 洗衣服 (新)

#### Overwrite → 买菜被覆盖

- 买菜 (completed)
- 做饭 (pending)
- 洗衣服 (新)

#### Skip → 保留服务端

- 买菜 (pending)
- 做饭 (pending)
- 洗衣服 (新)

#### Rename → 保留双方

- 买菜 (pending)
- 买菜 (1) (completed)
- 做饭 (pending)
- 洗衣服 (新)

---

## 健康检查

```
GET /health     → {"status":"ok","service":"nothing-todo-cloud"}
GET /livez     → OK (存活探针)
```

---

## 错误响应

```json
{"error":"Token 无效或已过期","success":false}
{"error":"缺少认证 Token","success":false}
{"error":"YAML解析失败: ...","success":false}
```
