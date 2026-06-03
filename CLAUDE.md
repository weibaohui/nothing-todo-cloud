# CLAUDE.md

## 项目概述

ntd-cloud 是 nothing-todo 的云端同步服务器，实现多设备间的数据同步。

## 开发流程

**禁止直接在主分支 (main) 上写代码。所有代码改动必须先创建分支，在分支上完成开发后再通过 PR 合入 main。**

## 代码注释规范

**强制要求：所有新增/修改的代码必须带注释。**

- **逐行注释**：每一行代码都要写注释，解释「为什么这么写」（不是「写了什么」）。说明意图、设计取舍、边界条件、踩过的坑，而不是复述语法。
- **段落总览注释**：在大段代码（如函数实现、复杂逻辑块、状态机分支）之前，先用一段注释说明整体的处理思路、输入输出和关键步骤，让读者不用读代码就能理解做了什么。
- **避免无意义注释**：`// 自增 i` 这类复述代码本身的注释属于噪音，要写成「为什么需要自增」「自增的边界是什么」。
- **修改既有代码时**：如果改动了原有逻辑，要同步更新或新增注释，不能让注释与代码脱节。

### 示例

❌ 反例（注释复述了代码，没解释为什么）：
```rust
// 调用 push_data 函数
push_data(&db, device_id, version, data_type, data, checksum)?;
```

✅ 正例（注释解释了意图与取舍）：
```rust
// Push 前验证 checksum，防止传输损坏的数据；
// 版本号必须递增，防止旧数据覆盖新数据。
let new_version = push_data(&db, device_id, version, data_type, data, checksum)?;
```

✅ 段落总览示例：
```rust
// 将设备数据存储到云端。
// 输入：device_id, version, data_type, 原始数据
// 流程：验证 checksum → 检查版本号 → 压缩存储 → 记录 sync_log
// 输出：新版本号
pub async fn push_data(...) -> anyhow::Result<i64> { ... }
```

## 技术栈

与 nothing-todo 主项目保持一致：

- 后端: Rust (Axum 框架) + rust-embed（前端嵌入）
- 前端: React + Vite + Ant Design
- 数据库: SQLite + SeaORM

**注意：前端通过 `rust-embed` 嵌入后端二进制，部署后无需单独运行前端服务。**

## 目录结构

```
nothing-todo-cloud/
├── backend/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs           # 主入口
│   │   ├── config.rs         # 配置管理
│   │   ├── error.rs          # 统一错误处理
│   │   ├── db/               # 数据库相关
│   │   │   ├── mod.rs
│   │   │   └── schema.rs    # SeaORM 数据表
│   │   ├── handlers/         # API 端点
│   │   │   ├── auth.rs      # 认证
│   │   │   ├── token.rs     # Token 管理
│   │   │   ├── device.rs    # 设备管理
│   │   │   ├── sync.rs      # 同步核心
│   │   │   └── admin.rs     # 管理后台
│   │   ├── middleware/       # 中间件
│   │   │   └── auth.rs      # JWT 认证
│   │   └── services/        # 业务逻辑
│   │       ├── auth_service.rs
│   │       ├── device_service.rs
│   │       └── sync_service.rs
│   └── config.yaml           # 默认配置
│
├── frontend/                  # React 前端（管理后台）
│   ├── src/
│   │   ├── pages/           # 页面
│   │   ├── components/      # 组件
│   │   └── api/             # API 调用
│   └── vite.config.ts
│
├── docs/                      # 设计文档
│   └── SYNC_DESIGN.md       # 同步系统设计
│
├── Dockerfile
├── docker-compose.yaml
└── README.md
```

## 部署

### 默认端口

- 服务器默认端口：**8089**
- 用户可通过修改 `config.yaml` 或环境变量 `NTD_SERVER__PORT` 自行更改

### 启动方式

```bash
# Docker 部署
docker-compose up -d

# 手动运行后端
cd backend
cargo run

# 前端开发
cd frontend
npm run dev
```

### 配置文件

- 配置文件：`./config.yaml` 或 `/etc/ntd-cloud/config.yaml`
- 数据库：`./data/ntd_cloud.db`
- 日志：stdout（Docker 环境）/ `ntd-cloud.log`（直接运行）

## 前端测试验证

**重要：修改前端 UI 后，必须使用 Playwright 进行自动化验证，再通知用户。**

### Playwright 测试脚本位置

测试脚本位于 `/tmp/` 目录下，文件名格式为 `check_*.js`

**运行方式**：由于 playwright 依赖在 `frontend/node_modules/` 中，需要在 `frontend/` 目录下执行：
```bash
cd frontend && npx playwright test --reporter=list
```

### 验证流程

1. 修改前端代码后，重启开发服务
2. 使用 Playwright 编写测试脚本验证 UI 效果
3. 验证通过后再通知用户

## 相关文档

- [同步系统设计文档](./docs/SYNC_DESIGN.md) - 详细的 API、数据模型、同步流程设计
