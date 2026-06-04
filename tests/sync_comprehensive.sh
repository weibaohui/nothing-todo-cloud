#!/bin/bash
# 全面的多设备同步测试
# 测试场景：向上(Push)/向下(Pull)同步，多设备合并，冲突解决，新字段完整性

set -e
BASE_URL="http://localhost:8089"
EMAIL="sync_test@example.com"
PASSWORD="123456"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 测试计数器
PASS=0
FAIL=0

# 断言函数
assert_contains() {
  local haystack="$1"
  local needle="$2"
  local msg="$3"
  if echo "$haystack" | grep -q "$needle"; then
    echo -e "   ${GREEN}✓${NC} $msg"
    PASS=$((PASS + 1))
  else
    echo -e "   ${RED}✗${NC} $msg"
    FAIL=$((FAIL + 1))
  fi
}

assert_count() {
  local count=$1
  local expected=$2
  local msg="$3"
  if [ "$count" -eq "$expected" ]; then
    echo -e "   ${GREEN}✓${NC} $msg"
    PASS=$((PASS + 1))
  else
    echo -e "   ${RED}✗${NC} $msg (期望: $expected, 实际: $count)"
    FAIL=$((FAIL + 1))
  fi
}

# 计算 todos 数量（只计算顶层的 title: 行）
count_todos() {
  local response="$1"
  echo "$response" | grep -c "^  - title:" || echo "0"
}

assert_success() {
  local response="$1"
  local msg="$2"
  if echo "$response" | grep -q "success: true"; then
    echo -e "   ${GREEN}✓${NC} $msg"
    PASS=$((PASS + 1))
  else
    echo -e "   ${RED}✗${NC} $msg"
    FAIL=$((FAIL + 1))
  fi
}

echo "========================================"
echo "  全面的多设备同步测试"
echo "========================================"
echo ""

# 清理数据库
echo -e "${YELLOW}0. 清理旧数据${NC}"
pkill -f "ntd-cloud-server" 2>/dev/null || true
sleep 1
rm -f /Users/mac/projects/rust/nothing-todo-cloud/backend/ntd_cloud.db
touch /Users/mac/projects/rust/nothing-todo-cloud/backend/ntd_cloud.db
echo "   ✓ 清理完成"
echo ""

# 启动服务器
echo -e "${YELLOW}1. 启动服务器...${NC}"
cd /Users/mac/projects/rust/nothing-todo-cloud/backend
cargo run > /tmp/ntd_server.log 2>&1 &
sleep 4
if curl -s "$BASE_URL/health" | grep -q "ok"; then
  echo "   ✓ 服务器启动成功"
else
  echo "   ✗ 服务器启动失败"
  cat /tmp/ntd_server.log
  exit 1
fi
echo ""

# 注册/登录
echo -e "${YELLOW}2. 注册用户${NC}"
REG=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
TOKEN=$(echo "$REG" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "   ✓ Token: ${TOKEN:0:40}..."
echo ""

# 创建设备
echo -e "${YELLOW}3. 创建设备 A、B、C${NC}"
DEV_A=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备A"}')
DEVICE_A_ID=$(echo "$DEV_A" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)

DEV_B=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备B"}')
DEVICE_B_ID=$(echo "$DEV_B" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)

DEV_C=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备C"}')
DEVICE_C_ID=$(echo "$DEV_C" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)

echo "   设备A ID: $DEVICE_A_ID"
echo "   设备B ID: $DEVICE_B_ID"
echo "   设备C ID: $DEVICE_C_ID"
echo ""

# ============ 测试函数 ============

# 使用文件推送，避免 bash 字符串换行问题
TMP_DIR="/tmp/ntd_sync_test_$$"
mkdir -p "$TMP_DIR"

push_yaml() {
  local DEVICE_ID=$1
  local DATA_TYPE=$2
  local YAML_FILE="$TMP_DIR/push_${DEVICE_ID}_${DATA_TYPE}_$(date +%s).yaml"

  # 从 stdin 读取 YAML 内容
  cat > "$YAML_FILE"

  curl -s -X POST "$BASE_URL/api/v1/sync/push" \
    -H "Content-Type: text/yaml" \
    -H "Authorization: Bearer $TOKEN" \
    --data-binary @"$YAML_FILE"
}

pull_yaml() {
  local DEVICE_ID=$1
  local DATA_TYPE=$2
  curl -s "$BASE_URL/api/v1/sync/pull?device_id=$DEVICE_ID&data_type=$DATA_TYPE" \
    -H "Authorization: Bearer $TOKEN"
}

# ============ 场景1: 新格式基本 Push/Pull ============
echo "========================================"
echo "  场景1: 新格式基本 Push/Pull"
echo "========================================"
echo ""

echo "【1.1】设备A Push Todos (完整新格式)"
PUSH_A1=$(push_yaml $DEVICE_A_ID "todos" << YAML
device_id: $DEVICE_A_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 讲个笑话
      prompt: 讲个笑话
      status: completed
      executor: atomcode
      scheduler_enabled: true
      scheduler_config: '0 0 * * * *'
      tag_names: []
      workspace: null
      worktree: null
    - title: 写代码
      prompt: 写代码
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names:
        - work
      workspace: /projects
      worktree: feature-branch
YAML
)
assert_success "$PUSH_A1" "Push 成功"
assert_contains "$PUSH_A1" "讲个笑话" "包含 讲个笑话"
assert_contains "$PUSH_A1" "atomcode" "包含 executor"
assert_contains "$PUSH_A1" "scheduler_enabled: true" "包含 scheduler 配置"
assert_contains "$PUSH_A1" "workspace: /projects" "包含 workspace"
assert_contains "$PUSH_A1" "worktree: feature-branch" "包含 worktree"
echo ""

echo "【1.2】设备A Pull 验证"
PULL_A1=$(pull_yaml $DEVICE_A_ID "todos")
assert_contains "$PULL_A1" "讲个笑话" "Pull 包含 讲个笑话"
assert_contains "$PULL_A1" "写代码" "Pull 包含 写代码"
assert_contains "$PULL_A1" "status: completed" "包含 status completed"
assert_contains "$PULL_A1" "status: pending" "包含 status pending"
assert_contains "$PULL_A1" "tag_names:" "包含 tag_names 字段"
echo ""

# ============ 场景2: 多设备数据合并 ============
echo "========================================"
echo "  场景2: 多设备数据合并 (3设备)"
echo "========================================"
echo ""

echo "【2.1】设备B Push Todos (2条，无重叠)"
PUSH_B1=$(cat << YAML | sed "s/DEVICE_ID_PLACEHOLDER/$DEVICE_B_ID/"
device_id: DEVICE_ID_PLACEHOLDER
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 洗衣服
      prompt: 洗衣服
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names:
        - life
      workspace: null
      worktree: null
    - title: 跑步
      prompt: 跑步
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
push_yaml $DEVICE_B_ID "todos" << YAML
device_id: $DEVICE_B_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 洗衣服
      prompt: 洗衣服
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names:
        - life
      workspace: null
      worktree: null
    - title: 跑步
      prompt: 跑步
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_B1" "设备B Push 成功"
assert_count "$(count_todos "$PUSH_B1")" 4 "合并后共4条 todos (A:2 + B:2)"
echo ""

echo "【2.2】设备C Push Todos (1条，无重叠)"
PUSH_C1=$(push_yaml $DEVICE_C_ID "todos" << YAML
device_id: $DEVICE_C_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 做饭
      prompt: 做饭
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names:
        - life
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_C1" "设备C Push 成功"
assert_count "$(count_todos "$PUSH_C1")" 5 "合并后共5条 todos (A:2 + B:2 + C:1)"
echo ""

echo "【2.3】设备A Pull 验证3设备合并"
PULL_A2=$(pull_yaml $DEVICE_A_ID "todos")
assert_contains "$PULL_A2" "讲个笑话" "包含 讲个笑话 (A)"
assert_contains "$PULL_A2" "写代码" "包含 写代码 (A)"
assert_contains "$PULL_A2" "洗衣服" "包含 洗衣服 (B)"
assert_contains "$PULL_A2" "跑步" "包含 跑步 (B)"
assert_contains "$PULL_A2" "做饭" "包含 做饭 (C)"
assert_count "$(count_todos "$PULL_A2")" 5 "共5条 todos"
echo ""

# ============ 场景3: 冲突解决 (同标题不同内容) ============
echo "========================================"
echo "  场景3: 冲突解决 (同标题不同内容)"
echo "========================================"
echo ""

echo "【3.1】设备A Push 相同标题 (pending)"
PUSH_A3=$(push_yaml $DEVICE_A_ID "todos" << YAML
device_id: $DEVICE_A_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 项目X
      prompt: 项目X
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_A3" "设备A Push 项目X"
echo ""

echo "【3.2】设备B Push 相同标题 (completed, 不同executor)"
PUSH_B3=$(push_yaml $DEVICE_B_ID "todos" << YAML
device_id: $DEVICE_B_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 项目X
      prompt: 项目X
      status: completed
      executor: atomcode
      scheduler_enabled: true
      scheduler_config: '0 0 * * * *'
      tag_names:
        - important
      workspace: /workspace
      worktree: null
YAML
)
assert_success "$PUSH_B3" "设备B Push 项目X (不同内容)"
echo ""

echo "【3.3】验证冲突解决策略"
PULL_A3=$(pull_yaml $DEVICE_A_ID "todos")
assert_contains "$PULL_A3" "项目X" "包含 项目X"
COUNT=$(echo "$PULL_A3" | grep -c "title: 项目X")
assert_count "$COUNT" 1 "项目X 只有1条（去重成功）"
echo ""

# ============ 场景9: 冲突模式测试 (overwrite/skip/rename) ============
echo "========================================"
echo "  场景9: 冲突模式测试"
echo "========================================"
echo ""

echo "【9.1】设备A Push 初始数据 (status: pending, title: 买水果)"
PUSH_A9=$(push_yaml $DEVICE_A_ID "todos" << YAML
device_id: $DEVICE_A_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 买水果
      prompt: 买水果
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_A9" "设备A Push 买水果 (pending)"
echo ""

echo "【9.2】设备B Push 同标题不同内容 (status: completed) 使用 overwrite 模式"
PUSH_B9=$(push_yaml $DEVICE_B_ID "todos" << YAML
device_id: $DEVICE_B_ID
data_type: todos
conflict_mode: overwrite
data: |
  version: '1.0'
  todos:
    - title: 买水果
      prompt: 买水果
      status: completed
      executor: atomcode
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_B9" "设备B overwrite Push 买水果"
echo ""

echo "【9.3】验证 overwrite 模式 (B 的 completed 覆盖 A 的 pending)"
PULL_A9=$(pull_yaml $DEVICE_A_ID "todos")
assert_contains "$PULL_A9" "买水果" "包含 买水果"
assert_contains "$PULL_A9" "status: completed" "overwrite 后 status 变为 completed"
echo ""

echo "【9.4】设备A 修改为 pending"
push_yaml $DEVICE_A_ID "todos" << YAML > /dev/null
device_id: $DEVICE_A_ID
data_type: todos
conflict_mode: overwrite
data: |
  version: '1.0'
  todos:
    - title: 买水果
      prompt: 买水果
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
echo "   ✓ 设备A 修改买水果为 pending"
echo ""

echo "【9.5】设备B Push skip 模式 (应保留 server 的 pending)"
PUSH_B9S=$(push_yaml $DEVICE_B_ID "todos" << YAML
device_id: $DEVICE_B_ID
data_type: todos
conflict_mode: skip
data: |
  version: '1.0'
  todos:
    - title: 买水果
      prompt: 买水果
      status: completed
      executor: atomcode
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_B9S" "设备B skip Push 买水果"
assert_contains "$PUSH_B9S" "status: pending" "skip 后保留 server 的 pending"
echo ""

echo "【9.6】设备A 修改为 pending"
push_yaml $DEVICE_A_ID "todos" << YAML > /dev/null
device_id: $DEVICE_A_ID
data_type: todos
conflict_mode: overwrite
data: |
  version: '1.0'
  todos:
    - title: 买水果
      prompt: 买水果
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
echo "   ✓ 设备A 修改买水果为 pending"
echo ""

echo "【9.7】设备B Push rename 模式 (应保留双方，客户端重命名)"
PUSH_B9R=$(push_yaml $DEVICE_B_ID "todos" << YAML
device_id: $DEVICE_B_ID
data_type: todos
conflict_mode: rename
data: |
  version: '1.0'
  todos:
    - title: 买水果
      prompt: 买水果
      status: completed
      executor: atomcode
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_B9R" "设备B rename Push 买水果"
assert_contains "$PUSH_B9R" "买水果" "保留原标题"
assert_contains "$PUSH_B9R" "买水果 (1)" "冲突项重命名为 买水果 (1)"
echo ""

echo "【9.8】验证最终状态 (买水果 pending, 买水果 (1) completed)"
PULL_A9R=$(pull_yaml $DEVICE_A_ID "todos")
assert_contains "$PULL_A9R" "买水果" "包含 买水果"
assert_contains "$PULL_A9R" "买水果 (1)" "包含重命名项 买水果 (1)"
COUNT9=$(echo "$PULL_A9R" | grep -c "title: 买水果")
assert_count "$COUNT9" 2 "共有2条买水果相关记录"
echo ""

# ============ 场景4: Tags 并集合并 ============
echo "========================================"
echo "  场景4: Tags 并集合并"
echo "========================================"
echo ""

echo "【4.1】设备A Push Tags"
PUSH_A4=$(push_yaml $DEVICE_A_ID "tags" << YAML
device_id: $DEVICE_A_ID
data_type: tags
data: |
  version: '1.0'
  tags:
    - 工作
    - 生活
    - Python
YAML
)
assert_success "$PUSH_A4" "设备A Push Tags"
echo ""

echo "【4.2】设备B Push Tags (有重叠: 工作)"
PUSH_B4=$(push_yaml $DEVICE_B_ID "tags" << YAML
device_id: $DEVICE_B_ID
data_type: tags
data: |
  version: '1.0'
  tags:
    - 工作
    - 学习
    - Rust
YAML
)
assert_success "$PUSH_B4" "设备B Push Tags"
echo ""

echo "【4.3】设备C Push Tags"
PUSH_C4=$(push_yaml $DEVICE_C_ID "tags" << YAML
device_id: $DEVICE_C_ID
data_type: tags
data: |
  version: '1.0'
  tags:
    - 运动
    - 休息
YAML
)
assert_success "$PUSH_C4" "设备C Push Tags"
echo ""

echo "【4.4】验证 Tags 合并"
PULL_A4=$(pull_yaml $DEVICE_A_ID "tags")
for tag in 工作 生活 Python 学习 Rust 运动 休息; do
  assert_contains "$PULL_A4" "  - $tag" "包含 tag: $tag"
done
echo ""

# ============ 场景5: Skills 并集合并 ============
echo "========================================"
echo "  场景5: Skills 并集合并"
echo "========================================"
echo ""

echo "【5.1】设备A Push Skills"
PUSH_A5=$(push_yaml $DEVICE_A_ID "skills" << YAML
device_id: $DEVICE_A_ID
data_type: skills
data: |
  version: '1.0'
  skills:
    - Rust
    - Python
YAML
)
assert_success "$PUSH_A5" "设备A Push Skills"
echo ""

echo "【5.2】设备B Push Skills"
PUSH_B5=$(push_yaml $DEVICE_B_ID "skills" << YAML
device_id: $DEVICE_B_ID
data_type: skills
data: |
  version: '1.0'
  skills:
    - Rust
    - Go
    - JavaScript
YAML
)
assert_success "$PUSH_B5" "设备B Push Skills"
echo ""

echo "【5.3】验证 Skills 合并"
PULL_A5=$(pull_yaml $DEVICE_A_ID "skills")
for skill in Rust Python Go JavaScript; do
  assert_contains "$PULL_A5" "  - $skill" "包含 skill: $skill"
done
echo ""

# ============ 场景6: 全量覆盖/删除场景 ============
echo "========================================"
echo "  场景6: 全量覆盖场景"
echo "========================================"
echo ""

echo "【6.1】设备B Push 覆盖（只保留: 买菜, 新任务）"
PUSH_B6=$(push_yaml $DEVICE_B_ID "todos" << YAML
device_id: $DEVICE_B_ID
data_type: todos
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
    - title: 新任务
      prompt: 新任务
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_B6" "设备B Push 覆盖"
echo ""

echo "【6.2】验证覆盖结果"
PULL_B6=$(pull_yaml $DEVICE_B_ID "todos")
assert_contains "$PULL_B6" "买菜" "包含 买菜"
assert_contains "$PULL_B6" "新任务" "包含 新任务"
echo ""

# ============ 场景7: 空数据处理 ============
echo "========================================"
echo "  场景7: 空数据处理"
echo "========================================"
echo ""

echo "【7.1】创建设备D并Push空todos"
DEV_D=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备D"}')
DEVICE_D_ID=$(echo "$DEV_D" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
echo "   设备D ID: $DEVICE_D_ID"

PUSH_D7=$(push_yaml $DEVICE_D_ID "todos" << YAML
device_id: $DEVICE_D_ID
data_type: todos
data: |
  version: '1.0'
  todos: []
YAML
)
assert_success "$PUSH_D7" "设备D Push 空 todos"
echo ""

echo "【7.2】设备D Pull 验证"
PULL_D7=$(pull_yaml $DEVICE_D_ID "todos")
PULL_D7_COUNT=$(count_todos "$PULL_D7")
echo "   设备D Pull 获得: $PULL_D7_COUNT 条 todos"
# 验证新设备能获取到服务器数据（数量 >= 1 即可）
if [ "$PULL_D7_COUNT" -ge 1 ]; then
  echo -e "   ${GREEN}✓${NC} 新设备能看到服务器已有数据 ($PULL_D7_COUNT 条)"
  PASS=$((PASS + 1))
else
  echo -e "   ${RED}✗${NC} 新设备能看到服务器已有数据 (期望: >=1, 实际: $PULL_D7_COUNT)"
  FAIL=$((FAIL + 1))
fi
echo ""

# ============ 场景8: 版本号和 created_at ============
echo "========================================"
echo "  场景8: 版本号和 created_at"
echo "========================================"
echo ""

echo "【8.1】验证 Push 响应包含 version 和 created_at"
PUSH_A8=$(push_yaml $DEVICE_A_ID "todos" << YAML
device_id: $DEVICE_A_ID
data_type: todos
data: |
  version: '1.0'
  todos:
    - title: 测试版本
      prompt: 测试
      status: pending
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_contains "$PUSH_A8" "version:" "包含 version 字段"
assert_contains "$PUSH_A8" "created_at:" "包含 created_at 字段"
echo ""

# ============ 总结 ============
echo "========================================"
echo "  测试完成"
echo "========================================"
echo ""
echo -e "${GREEN}通过: $PASS${NC}"
echo -e "${RED}失败: $FAIL${NC}"
echo ""

# 清理
rm -rf "$TMP_DIR"
pkill -f "ntd-cloud-server" 2>/dev/null || true

if [ "$FAIL" -eq 0 ]; then
  echo -e "${GREEN}所有测试通过！${NC}"
  exit 0
else
  echo -e "${RED}有 $FAIL 个测试失败${NC}"
  exit 1
fi