# nothing-todo-cloud

ntd (Nothing Todo) 的云端同步服务器，支持多设备间同步 Todos、Tags 和 Skills。

## 功能特性

- **多设备同步** - 支持多台主机间的数据同步
- **Token 认证** - 安全的 API Token 认证机制
- **冲突解决** - 多种冲突处理策略供选择
- **前后端分离** - 提供完整的管理后台

## 快速部署

### Docker 部署（推荐）

```bash
git clone https://github.com/your-username/nothing-todo-cloud.git
cd nothing-todo-cloud
docker-compose up -d
```

访问 `http://your-server:8080` 创建管理员账户。

### 手动部署

```bash
# 构建后端
cd backend
cargo build --release
./target/release/ntd-cloud-server

# 构建前端
cd frontend
npm install
npm run build
```

## 客户端配置

在 `~/.ntd/config.yaml` 中添加：

```yaml
sync:
  enabled: true
  server: "http://your-server:8080"
  token: "your-api-token"
  auto_sync: true
  sync_on_startup: true
```

## 技术栈

- 后端: Rust + Axum + SeaORM + SQLite
- 前端: React + Vite + Ant Design

## 项目结构

```
nothing-todo-cloud/
├── backend/           # Rust 后端
├── frontend/          # React 前端（管理后台）
├── Dockerfile
├── docker-compose.yaml
└── README.md
```

## 许可证

MIT
