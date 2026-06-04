#!/bin/bash
# 同步 API 测试脚本
# 测试: 注册/登录/创建Token/Push/Pull/DryRun

set -e
BASE_URL="http://localhost:8089"
EMAIL="test@example.com"
PASSWORD="123456"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

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
echo "  同步 API 测试"
echo "========================================"
echo ""

# 清理并启动服务器
echo -e "${YELLOW}0. 启动服务器${NC}"
pkill -f "ntd-cloud-server" 2>/dev/null || true
sleep 1
cd /Users/mac/projects/rust/nothing-todo-cloud/backend
rm -f ntd_cloud.db
touch ntd_cloud.db
cargo run > /tmp/ntd_server.log 2>&1 &
sleep 4
echo "   ✓ 服务器启动"
echo ""

# ========== 1. 注册 ==========
echo -e "${YELLOW}1. 注册用户${NC}"
REG=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
# 注册可能返回已存在，但只要有token就算成功
if echo "$REG" | grep -q '"token":"'; then
  echo -e "   ${GREEN}✓${NC} 注册成功"
  PASS=$((PASS + 1))
else
  echo -e "   ${RED}✗${NC} 注册失败: $REG"
  FAIL=$((FAIL + 1))
fi
JWT=$(echo "$REG" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "   JWT: ${JWT:0:40}..."
echo ""

# ========== 2. 创建设备 Token ==========
echo -e "${YELLOW}2. 创建同步 Token (设备A)${NC}"
TOKEN_RESP=$(curl -s -X POST "$BASE_URL/api/tokens" \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"name":"设备A"}')
assert_contains "$TOKEN_RESP" "ntd_" "Token 格式正确"
SYNC_TOKEN=$(echo "$TOKEN_RESP" | grep -o '"token":"ntd_[^"]*"' | cut -d'"' -f4)
echo "   Token: $SYNC_TOKEN"
echo ""

# ========== 3. Push ==========
echo -e "${YELLOW}3. Push 数据${NC}"
PUSH_RESP=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Authorization: Bearer $SYNC_TOKEN" \
  -H "Content-Type: text/yaml" \
  --data-binary @- << YAML
data_type: todos
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
    - title: 做饭
      status: pending
      prompt: 做饭
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$PUSH_RESP" "Push 成功"
assert_contains "$PUSH_RESP" "买菜" "包含 买菜"
assert_contains "$PUSH_RESP" "做饭" "包含 做饭"
echo ""

# ========== 4. Pull ==========
echo -e "${YELLOW}4. Pull 数据${NC}"
PULL_RESP=$(curl -s "$BASE_URL/api/v1/sync/pull?data_type=todos" \
  -H "Authorization: Bearer $SYNC_TOKEN")
assert_contains "$PULL_RESP" "买菜" "Pull 包含 买菜"
assert_contains "$PULL_RESP" "做饭" "包含 做饭"
echo ""

# ========== 5. Dry Run ==========
echo -e "${YELLOW}5. Dry Run 预览${NC}"
DRY_RESP=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Authorization: Bearer $SYNC_TOKEN" \
  -H "Content-Type: text/yaml" \
  --data-binary @- << YAML
data_type: todos
conflict_mode: rename
dry_run: true
data: |
  version: '1.0'
  todos:
    - title: 买菜
      status: completed
      prompt: 买菜
      executor: atomcode
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
    - title: 新任务
      status: pending
      prompt: 新任务
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$DRY_RESP" "Dry Run 成功"
assert_contains "$DRY_RESP" "preview: true" "preview 标识"
assert_contains "$DRY_RESP" "action: rename" "rename 动作"
assert_contains "$DRY_RESP" "买菜 (1)" "重命名结果"
echo ""

# ========== 6. 验证 Dry Run 不影响实际数据 ==========
echo -e "${YELLOW}6. 验证 Dry Run 不影响数据${NC}"
VERIFY=$(curl -s "$BASE_URL/api/v1/sync/pull?data_type=todos" \
  -H "Authorization: Bearer $SYNC_TOKEN")
if echo "$VERIFY" | grep -q "买菜 (1)"; then
  echo -e "   ${RED}✗${NC} Dry Run 不应修改数据"
  FAIL=$((FAIL + 1))
else
  echo -e "   ${GREEN}✓${NC} Dry Run 未修改数据"
  PASS=$((PASS + 1))
fi
echo ""

# ========== 7. 实际执行 rename ==========
echo -e "${YELLOW}7. 执行 rename Push${NC}"
RENAME_RESP=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Authorization: Bearer $SYNC_TOKEN" \
  -H "Content-Type: text/yaml" \
  --data-binary @- << YAML
data_type: todos
conflict_mode: rename
data: |
  version: '1.0'
  todos:
    - title: 买菜
      status: completed
      prompt: 买菜
      executor: atomcode
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
    - title: 新任务
      status: pending
      prompt: 新任务
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)
assert_success "$RENAME_RESP" "Rename Push 成功"
assert_contains "$RENAME_RESP" "买菜 (1)" "生成重命名项"
echo ""

# ========== 总结 ==========
echo "========================================"
echo "  测试完成"
echo "========================================"
echo ""
echo -e "${GREEN}通过: $PASS${NC}"
echo -e "${RED}失败: $FAIL${NC}"
echo ""

pkill -f "ntd-cloud-server" 2>/dev/null || true

if [ "$FAIL" -eq 0 ]; then
  echo -e "${GREEN}所有测试通过！${NC}"
  exit 0
else
  echo -e "${RED}有 $FAIL 个测试失败${NC}"
  exit 1
fi