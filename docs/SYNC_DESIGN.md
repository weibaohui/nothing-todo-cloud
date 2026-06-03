# nothing-todo-cloud 同步系统设计文档

## 1. 概述

### 1.1 项目定位

`nothing-todo-cloud` 是 `nothing-todo` (ntd) 的云端同步服务器，实现多设备间的数据同步。

### 1.2 架构图

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│ 主机 A  │     │ 主机 B  │     │ 主机 C  │     │ 主机 D  │
│ (本地)  │     │ (本地)  │     │ (本地)  │     │ (本地)  │
└────┬────┘     └────┬────┘     └────┬────┘     └────┬────┘
     │               │               │               │
     └───────────────┴───────────────┴───────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  nothing-todo-cloud │
              │    (中转服务器)      │
              │                     │
              │  - 用户认证         │
              │  - 设备管理         │
              │  - 数据存储         │
              │  - 冲突处理         │
              └─────────────────────┘
```

### 1.3 技术栈

| 层级 | 技术选型 | 理由 |
|------|---------|------|
| 后端 | Rust + Axum | 与主项目一致，代码复用方便 |
| 前端 | React + Vite + Ant Design | 与主项目一致 |
| 数据库 | SQLite + SeaORM | 轻量级，用户自建无需额外依赖 |
| 认证 | JWT Token | 简单可靠，支持 stateless |

---

## 2. 数据库设计

### 2.1 ER 图

```
┌─────────────┐       ┌─────────────┐       ┌─────────────────┐
│   users     │       │ api_tokens  │       │  device_snapshots│
├─────────────┤       ├─────────────┤       ├─────────────────┤
│ id          │───┐   │ id          │       │ id               │
│ email       │   │   │ user_id     │──────→│ device_id        │
│ password_hash│  │   │ name        │       │ version          │
│ created_at  │   │   │ token_hash  │       │ data_type        │
│ plan        │   │   │ last_used  │       │ data_payload     │
└─────────────┘   │   │ created_at │       │ checksum         │
                  │   └─────────────┘       │ created_at       │
                  │                         │ metadata         │
                  │   ┌─────────────┐       └─────────────────┘
                  │   │  devices    │               ▲
                  └──→│ id          │               │
                      │ user_id     │───────┐       │
                      │ device_name │       │       │
                      │ device_key  │       │       │
                      │ last_seen   │       │       │
                      │ created_at  │       │       │
                      └─────────────┘       │       │
                                            │       │
                          ┌─────────────────┘       │
                          ▼                         │
                   ┌─────────────┐                  │
                   │  sync_logs  │                  │
                   ├─────────────┤                  │
                   │ id          │                  │
                   │ device_id   │──────────────────┘
                   │ action      │
                   │ status      │
                   │ details     │
                   │ created_at  │
                   └─────────────┘
```

### 2.2 表结构

#### users - 用户表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGINT PK | 用户 ID |
| email | VARCHAR(255) UNIQUE | 邮箱 |
| password_hash | VARCHAR(255) | bcrypt 哈希后的密码 |
| created_at | DATETIME | 创建时间 |
| plan | VARCHAR(50) | 套餐：free/pro |

#### devices - 设备表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGINT PK | 设备 ID |
| user_id | BIGINT FK | 所属用户 |
| device_name | VARCHAR(255) | 设备名称，如 "MacBook Pro" |
| device_key | TEXT NULL | 设备公钥（未来扩展） |
| last_seen_at | DATETIME | 最后访问时间 |
| created_at | DATETIME | 注册时间 |

#### api_tokens - API Token 表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGINT PK | Token ID |
| user_id | BIGINT FK | 所属用户 |
| name | VARCHAR(255) | Token 名称，如 "Home Server" |
| token_hash | VARCHAR(255) | Token 哈希（不存明文） |
| last_used_at | DATETIME NULL | 最后使用时间 |
| created_at | DATETIME | 创建时间 |

#### device_snapshots - 设备数据快照表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGINT PK | 快照 ID |
| device_id | BIGINT FK | 所属设备 |
| version | BIGINT | 递增版本号 |
| data_type | VARCHAR(50) | 数据类型：todos/tags/skills/all |
| data_payload | TEXT | gzip 压缩 + base64 编码后的数据 |
| checksum | VARCHAR(64) | SHA256 校验和 |
| created_at | DATETIME | 创建时间 |
| metadata | TEXT NULL | 额外元数据（JSON） |

#### sync_logs - 同步日志表

| 字段 | 类型 | 说明 |
|------|------|------|
| id | BIGINT PK | 日志 ID |
| device_id | BIGINT FK | 所属设备 |
| action | VARCHAR(50) | 操作类型：push/pull/merge |
| status | VARCHAR(50) | 状态：success/failed |
| details | TEXT NULL | 详情 |
| created_at | DATETIME | 时间 |

---

## 3. API 设计

### 3.1 认证相关

#### POST /api/auth/register - 用户注册

**请求：**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**响应：**
```json
{
  "success": true,
  "token": "eyJhbGc...",
  "user_id": 1
}
```

#### POST /api/auth/login - 用户登录

**请求：**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**响应：**
```json
{
  "success": true,
  "token": "eyJhbGc...",
  "user_id": 1
}
```

#### POST /api/auth/logout - 用户登出

**响应：**
```json
{
  "success": true,
  "token": null,
  "user_id": null
}
```

### 3.2 Token 管理

#### GET /api/tokens - 列出所有 Token

**响应：**
```json
[
  {
    "id": 1,
    "name": "Home Server",
    "last_used_at": "2026-01-01T00:00:00Z",
    "created_at": "2026-01-01T00:00:00Z"
  }
]
```

#### POST /api/tokens - 创建新 Token

**请求：**
```json
{
  "name": "Home Server"
}
```

**响应：**
```json
{
  "id": 1,
  "name": "Home Server",
  "token": "ntd_cloud_abc123...",  // 仅创建时返回明文
  "last_used_at": null,
  "created_at": "2026-01-01T00:00:00Z"
}
```

#### DELETE /api/tokens/:id - 撤销 Token

**响应：**
```json
{
  "success": true
}
```

### 3.3 设备管理

#### GET /api/devices - 列出所有设备

**响应：**
```json
[
  {
    "id": 1,
    "device_name": "MacBook Pro",
    "last_seen_at": "2026-01-01T00:00:00Z",
    "created_at": "2026-01-01T00:00:00Z"
  }
]
```

#### POST /api/devices - 注册新设备

**请求：**
```json
{
  "name": "MacBook Pro"
}
```

**响应：**
```json
{
  "id": 1,
  "device_name": "MacBook Pro",
  "last_seen_at": "2026-01-01T00:00:00Z",
  "created_at": "2026-01-01T00:00:00Z"
}
```

#### DELETE /api/devices/:id - 删除设备

**响应：**
```json
{
  "success": true
}
```

### 3.4 同步核心 API (v1)

#### GET /api/v1/sync/status - 获取同步状态

**查询参数：**
- `device_id`: 设备 ID

**响应：**
```json
{
  "device_id": 1,
  "version": 10,
  "last_sync_at": "2026-01-01T00:00:00Z",
  "has_conflict": false
}
```

#### POST /api/v1/sync/push - 上传本地数据

**请求：**
```json
{
  "device_id": 1,
  "version": 10,
  "data_type": "all",
  "data": "H4sIAAAAAAAAA...",  // gzip + base64
  "checksum": "sha256:abc123..."
}
```

**响应：**
```json
{
  "success": true,
  "new_version": 11
}
```

#### GET /api/v1/sync/pull - 拉取远端数据

**查询参数：**
- `device_id`: 设备 ID
- `data_type`: 数据类型（可选，默认 all）

**响应：**
```json
{
  "device_id": 1,
  "version": 11,
  "data_type": "all",
  "data": "H4sIAAAAAAAAA...",
  "checksum": "sha256:abc123...",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

#### POST /api/v1/sync/resolve - 解决同步冲突

**请求：**
```json
{
  "device_id": 1,
  "strategy": "overwrite_local",
  "merged_data": null
}
```

**策略选项：**
- `overwrite_local`: 用云端数据覆盖本地
- `overwrite_remote`: 用本地数据覆盖云端
- `merge`: 合并数据

**响应：**
```json
{
  "success": true,
  "new_version": 12
}
```

### 3.5 管理后台 API

#### GET /api/admin/stats - 系统统计

**响应：**
```json
{
  "total_users": 10,
  "total_devices": 25,
  "total_syncs": 1000
}
```

#### GET /api/admin/users - 用户列表

**响应：**
```json
[
  {
    "id": 1,
    "email": "user@example.com",
    "plan": "pro",
    "created_at": "2026-01-01T00:00:00Z"
  }
]
```

---

## 4. 同步流程

### 4.1 Push 流程（本地 → 云端）

```
┌──────────┐                              ┌──────────────┐
│  本地    │                              │  Cloud Server │
│  客户端  │                              │              │
└────┬─────┘                              └──────┬───────┘
     │                                           │
     │  1. 压缩本地数据（gzip）                  │
     │  2. 计算 checksum（SHA256）               │
     │  3. 生成版本号                           │
     │                                           │
     │─── POST /api/v1/sync/push ──────────────→│
     │    {                                     │
     │      device_id: 1,                      │
     │      version: 10,                       │
     │      data_type: "all",                  │
     │      data: "H4sI...",                  │
     │      checksum: "sha256:..."             │
     │    }                                    │
     │                                           │
     │                      4. 验证 checksum    │
     │                      5. 检查 version     │
     │                      6. 存储快照         │
     │                      7. 递增 version     │
     │                                           │
     │←── { success: true, new_version: 11 } ─│
     │                                           │
     │  8. 更新本地 version                     │
     ▼                                           ▼
```

### 4.2 Pull 流程（云端 → 本地）

```
┌──────────┐                              ┌──────────────┐
│  本地    │                              │  Cloud Server │
│  客户端  │                              │              │
└────┬─────┘                              └──────┬───────┘
     │                                           │
     │─── GET /api/v1/sync/pull?device_id=1 ──→│
     │                                           │
     │                      1. 查询设备最新快照  │
     │                      2. 返回数据          │
     │                                           │
     │←── {                                     │──→ 返回数据
     │      device_id: 1,                       │
     │      version: 11,                        │
     │      data_type: "all",                   │
     │      data: "H4sI...",                    │
     │      checksum: "sha256:...",             │
     │      updated_at: "..."                    │
     │    }                                     │
     │                                           │
     │  3. 验证 checksum                        │
     │  4. 解压数据（gunzip）                   │
     │  5. 与本地数据合并/覆盖                  │
     │  6. 更新本地 version                     │
     ▼                                           ▼
```

### 4.3 冲突检测与解决

```
┌─────────────────────────────────────────────────────────────┐
│  冲突场景                                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  本地 version: 10                                           │
│  云端 version: 12  ──→ 有新数据 ──→ 检测冲突               │
│                                                             │
│  冲突类型：同一 todo 在两端都有修改                         │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  主机 A: "完成报告"  → updated_at: 10:00            │   │
│  │  主机 B: "完成报告"  → updated_at: 10:05            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**解决策略：**

| 策略 | 说明 | 适用场景 |
|------|------|---------|
| `overwrite_local` | 直接用云端数据覆盖本地 | 云端为权威数据源 |
| `overwrite_remote` | 用本地数据覆盖云端 | 本地为权威数据源 |
| `merge` | 智能合并 | 两边都有有效修改 |

---

## 5. 客户端配置

### 5.1 配置项

在 `~/.ntd/config.yaml` 中添加：

```yaml
sync:
  enabled: true
  server: "http://your-server:8080"
  token: "ntd_cloud_xxx"
  auto_sync: true        # 自动同步
  sync_interval: 5      # 同步间隔（分钟）
  sync_on_startup: true # 启动时同步
  conflict_strategy: "ask"  # 冲突策略：overwrite_local / overwrite_remote / merge / ask
```

### 5.2 数据打包格式

```json
{
  "version": "1.0",
  "type": "all",  // all / todos / tags / skills
  "timestamp": "2026-01-01T00:00:00Z",
  "data": {
    "todos": [...],
    "tags": [...],
    "skills": "base64_encoded_zip_of_skills_dir"
  }
}
```

---

## 6. 安全设计

### 6.1 认证流程

```
┌─────────────────────────────────────────────────────────────┐
│  首次配置流程                                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. 用户在 Cloud 服务器创建账户                              │
│                                                             │
│  2. 生成 API Token                                          │
│     → 管理员后台: API Tokens → 创建 Token                  │
│     → Token 仅在创建时显示一次，需用户复制保存              │
│                                                             │
│  3. 配置本地客户端                                          │
│     → 在 config.yaml 配置 server + token                  │
│                                                             │
│  4. 客户端请求带上 Token                                    │
│     → Header: Authorization: Bearer <token>               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Token 安全

- Token 使用 SHA256 哈希存储，不存明文
- 支持多 Token，可为不同设备生成不同 Token
- 可随时撤销单个 Token
- Token 使用 HTTPS 传输

### 6.3 数据安全

- 全链路 HTTPS 加密
- 数据 payload 使用 gzip 压缩 + base64 编码
- 传输时携带 SHA256 checksum 校验完整性

---

## 7. 待实现功能

### 7.1 必须实现

- [ ] 用户注册/登录（bcrypt + JWT）
- [ ] API Token CRUD
- [ ] 设备注册/管理
- [ ] Push 数据存储（版本管理）
- [ ] Pull 数据拉取
- [ ] 冲突检测
- [ ] 冲突解决策略
- [ ] 同步日志记录

### 7.2 未来扩展

- [ ] WebSocket 实时推送
- [ ] 增量同步（只同步变化的字段）
- [ ] Skills 目录同步
- [ ] 多用户协作（分享 todo 给其他用户）
- [ ] 同步历史查看
- [ ] 冲突可视化对比

---

## 8. 部署方式

### 8.1 Docker 部署（推荐）

```bash
git clone https://github.com/xxx/nothing-todo-cloud.git
cd nothing-todo-cloud
docker-compose up -d
```

### 8.2 手动部署

```bash
# 后端
cd backend
cargo build --release
./target/release/ntd-cloud-server

# 前端
cd frontend
npm install
npm run build
# 静态文件用 nginx 托管
```

### 8.3 环境变量覆盖

```bash
export NTD_SERVER__PORT=8080
export NTD_DATABASE__URL=sqlite:ntd_cloud.db
export NTD_JWT__SECRET=your-secret-key
```
