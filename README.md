# nothing-todo-cloud

ntd-cloud 是 **nothing-todo** 的云端同步服务器，提供 Todo 数据的云端存储和多设备同步功能。

## 功能特性

### 核心功能

- **Todo 管理** - 创建、编辑、删除、导入 Todo 条目
- **多设备同步** - 支持多台设备间的 Todo 数据实时同步
- **Token 认证** - 安全的 API Token 认证机制
- **管理后台** - Web 控制台，支持用户管理、同步日志查看

### 管理后台页面

| 页面 | 功能 |
|------|------|
| **Dashboard** | 系统统计（用户数、Token 数、Todo 数、同步记录） |
| **Todo 管理** | Todo 的增删改查、状态管理、标签筛选、YAML 导入 |
| **Token 管理** | API Token 的创建、复制、撤销 |
| **设置** | 用户信息修改、密码更新 |

### 同步 API

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/v1/sync/status` | GET | 获取同步状态 |
| `/api/v1/sync/push` | POST | 上传本地数据到云端 |
| `/api/v1/sync/pull` | GET | 从云端拉取数据 |

## 快速部署

### Docker 部署（推荐）

```bash
docker-compose up -d
```

服务默认运行在 `http://localhost:8089`

### 手动部署

```bash
# 1. 构建前端
cd frontend && npm install && npm run build && cd ..

# 2. 构建后端
cd backend
cargo build --release

# 3. 运行
./target/release/ntd-cloud-server
```

## 客户端配置

在 `~/.ntd/config.yaml` 中配置同步：

```yaml
sync:
  enabled: true
  server: "http://your-server:8089"
  token: "your-api-token"
  auto_sync: true
  sync_on_startup: true
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust + Axum + SeaORM + SQLite |
| 前端 | React + Vite + Ant Design |
| 嵌入 | rust-embed（前端资源直接打包进二进制） |

## 目录结构

```
nothing-todo-cloud/
├── backend/
│   ├── src/
│   │   ├── main.rs          # 主入口、路由配置
│   │   ├── config.rs         # 配置管理
│   │   ├── db/               # 数据库相关
│   │   │   ├── mod.rs        # 数据库初始化
│   │   │   └── schema.rs     # 数据表定义
│   │   ├── handlers/          # API 处理器
│   │   │   ├── auth.rs       # 认证（注册/登录）
│   │   │   ├── token.rs       # Token 管理
│   │   │   ├── sync.rs        # 同步核心
│   │   │   ├── todo.rs        # Todo CRUD
│   │   │   └── admin.rs       # 管理后台
│   │   ├── middleware/         # 中间件
│   │   │   └── auth.rs        # JWT 认证
│   │   └── services/          # 业务逻辑
│   └── Cargo.toml
│
├── frontend/
│   ├── src/
│   │   ├── pages/            # 页面组件
│   │   │   ├── Dashboard.tsx # 控制台
│   │   │   ├── Todos.tsx     # Todo 管理
│   │   │   ├── Tokens.tsx    # Token 管理
│   │   │   ├── Login.tsx     # 登录
│   │   │   └── Settings.tsx   # 设置
│   │   ├── api/              # API 调用
│   │   ├── App.tsx           # 应用入口
│   │   └── index.css         # 全局样式
│   └── package.json
│
├── docs/                     # 设计文档
├── Dockerfile
├── docker-compose.yaml
└── Makefile
```

## 二进制下载

从 GitHub Releases 下载预编译的二进制文件：

- `ntd-cloud-server-linux-x64` - Linux x64
- `ntd-cloud-server-linux-arm64` - Linux ARM64
- `ntd-cloud-server-darwin-arm64` - macOS Apple Silicon
- `ntd-cloud-server-darwin-x64` - macOS Intel
- `ntd-cloud-server.exe` - Windows

## 开发

```bash
# 安装依赖
make setup

# 前端开发
cd frontend && npm run dev

# 后端开发
make dev

# 构建发布版本
make build
```

## 配置

配置文件位于 `./config.yaml` 或 `/etc/ntd-cloud/config.yaml`：

```yaml
database:
  url: "sqlite://data/ntd_cloud.db?mode=rwc"

server:
  port: 8089
```

## 许可证

MIT
