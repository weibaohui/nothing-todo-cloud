#!/bin/bash
# Dry Run 功能测试：验证输出结构化和用户可读性

set -e
BASE_URL="http://localhost:8089"
EMAIL="dryrun_view_test@example.com"
PASSWORD="123456"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local msg="$3"
  if ! echo "$haystack" | grep -q "$needle"; then
    echo -e "   ${GREEN}✓${NC} $msg"
    PASS=$((PASS + 1))
  else
    echo -e "   ${RED}✗${NC} $msg"
    FAIL=$((FAIL + 1))
  fi
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local msg="$3"
  if [ "$actual" = "$expected" ]; then
    echo -e "   ${GREEN}✓${NC} $msg"
    PASS=$((PASS + 1))
  else
    echo -e "   ${RED}✗${NC} $msg (期望: $expected, 实际: $actual)"
    FAIL=$((FAIL + 1))
  fi
}

echo "========================================"
echo "  Dry Run 输出结构化测试"
echo "========================================"
echo ""

# 清理旧数据
echo -e "${YELLOW}0. 准备测试环境${NC}"
pkill -f "ntd-cloud-server" 2>/dev/null || true
sleep 1
rm -f /Users/mac/projects/rust/nothing-todo-cloud/backend/ntd_cloud.db
touch /Users/mac/projects/rust/nothing-todo-cloud/backend/ntd_cloud.db
cd /Users/mac/projects/rust/nothing-todo-cloud/backend
cargo run > /tmp/ntd_server.log 2>&1 &
cd /Users/mac/projects/rust/nothing-todo-cloud
sleep 4
echo "   ✓ 服务器启动"
echo ""

# 注册用户
TOKEN=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

# 创建设备
DEV=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备A"}')
DEVICE_ID=$(echo "$DEV" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
echo "   设备ID: $DEVICE_ID"
echo ""

# ========== 准备初始数据 ==========
echo -e "${YELLOW}1. 准备初始数据${NC}"
curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  --data-binary @- << YAML > /dev/null
device_id: $DEVICE_ID
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
    - title: 洗衣服
      status: completed
      prompt: 洗衣服
      executor: atomcode
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
echo "   ✓ 服务器现有 3 条 todos"
echo ""

# ========== 测试1: 结构完整性 ==========
echo "========================================"
echo "  测试1: 结构完整性检查"
echo "========================================"
echo ""

DRYRUN_RESULT=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  --data-binary @- << YAML
device_id: $DEVICE_ID
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
    - title: 新任务A
      status: pending
      prompt: 新任务A
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
    - title: 新任务B
      status: pending
      prompt: 新任务B
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML
)

echo "【1.1】检查顶层字段"
assert_contains "$DRYRUN_RESULT" "success: true" "包含 success 字段"
assert_contains "$DRYRUN_RESULT" "preview: true" "包含 preview 标识"
assert_contains "$DRYRUN_RESULT" "conflict_mode: rename" "包含 conflict_mode"
assert_contains "$DRYRUN_RESULT" "merged_data:" "包含 merged_data 预览"
assert_contains "$DRYRUN_RESULT" "conflicts:" "包含 conflicts 列表"
assert_contains "$DRYRUN_RESULT" "summary:" "包含 summary 统计"
echo ""

echo "【1.2】检查 conflicts 结构"
assert_contains "$DRYRUN_RESULT" "conflicts:" "conflicts 数组存在"
assert_contains "$DRYRUN_RESULT" "  - title:" "冲突项包含 title"
assert_contains "$DRYRUN_RESULT" "    action:" "冲突项包含 action"
assert_contains "$DRYRUN_RESULT" "    server_item:" "冲突项包含 server_item"
assert_contains "$DRYRUN_RESULT" "    client_item:" "冲突项包含 client_item"
echo ""

echo "【1.3】检查 summary 结构"
assert_contains "$DRYRUN_RESULT" "summary:" "summary 对象存在"
assert_contains "$DRYRUN_RESULT" "total_client_items:" "包含 total_client_items"
assert_contains "$DRYRUN_RESULT" "new_items:" "包含 new_items"
assert_contains "$DRYRUN_RESULT" "overwritten:" "包含 overwritten"
assert_contains "$DRYRUN_RESULT" "skipped:" "包含 skipped"
assert_contains "$DRYRUN_RESULT" "renamed:" "包含 renamed"
assert_contains "$DRYRUN_RESULT" "final_total:" "包含 final_total"
echo ""

# ========== 测试2: 内容准确性 ==========
echo "========================================"
echo "  测试2: 内容准确性检查"
echo "========================================"
echo ""

echo "【2.1】检查冲突项识别"
assert_contains "$DRYRUN_RESULT" "title: 买菜" "识别出冲突项: 买菜"
assert_contains "$DRYRUN_RESULT" "action: rename" "识别出 action: rename"
assert_contains "$DRYRUN_RESULT" "new_title: 买菜 (1)" "生成新标题: 买菜 (1)"
echo ""

echo "【2.2】检查新增项识别"
# 买菜 冲突后 rename，所以 new_items 应该是 2 (新任务A + 新任务B)
assert_eq "$(echo "$DRYRUN_RESULT" | grep -A1 'new_items:' | tail -1 | tr -d ' ')" "new_items: 2" "新增项数量正确"
echo ""

echo "【2.3】检查最终总数"
# 服务器原有 3 条，客户端 3 条，rename 后 3 + 2 = 5
assert_eq "$(echo "$DRYRUN_RESULT" | grep -A1 'final_total:' | tail -1 | tr -d ' ')" "final_total: 5" "最终总数正确 (5)"
echo ""

echo "【2.4】检查 rename 数量"
assert_eq "$(echo "$DRYRUN_RESULT" | grep -A1 'renamed:' | tail -1 | tr -d ' ')" "renamed: 1" "重命名数量正确 (1)"
echo ""

# ========== 测试3: 用户可读性 ==========
echo "========================================"
echo "  测试3: 用户可读性检查"
echo "========================================"
echo ""

echo "【3.1】检查 action 可读性"
assert_contains "$DRYRUN_RESULT" "action: overwrite" "overwrite 标识可读"
assert_contains "$DRYRUN_RESULT" "action: skip" "skip 标识可读"
assert_contains "$DRYRUN_RESULT" "action: rename" "rename 标识可读"
echo ""

echo "【3.2】检查 server/client 区分"
# server_item 和 client_item 都应该有 title 字段
SERVER_TITLE_COUNT=$(echo "$DRYRUN_RESULT" | grep -A20 "server_item:" | grep -c "title:" || echo 0)
CLIENT_TITLE_COUNT=$(echo "$DRYRUN_RESULT" | grep -A20 "client_item:" | grep -c "title:" || echo 0)
assert_eq "$SERVER_TITLE_COUNT" "1" "server_item 包含 title"
assert_eq "$CLIENT_TITLE_COUNT" "1" "client_item 包含 title"
echo ""

echo "【3.3】检查 status 变化显示"
assert_contains "$DRYRUN_RESULT" "status: pending" "保留原 status"
assert_contains "$DRYRUN_RESULT" "status: completed" "显示新 status"
echo ""

# ========== 测试4: 实际执行验证 ==========
echo "========================================"
echo "  测试4: Dry Run 不影响实际数据"
echo "========================================"
echo ""

# Dry run 之前的状态
BEFORE=$(curl -s "$BASE_URL/api/v1/sync/pull?device_id=$DEVICE_ID&data_type=todos" \
  -H "Authorization: Bearer $TOKEN")
BEFORE_COUNT=$(echo "$BEFORE" | grep -c "^  - title:")

echo "   Dry run 前数据库状态: $BEFORE_COUNT 条"

# 执行一次真正的 push
curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  --data-binary @- << YAML > /dev/null
device_id: $DEVICE_ID
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
    - title: 新任务A
      status: pending
      prompt: 新任务A
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
    - title: 新任务B
      status: pending
      prompt: 新任务B
      executor: shell
      scheduler_enabled: false
      tag_names: []
      workspace: null
      worktree: null
YAML

# Dry run 之后的状态
AFTER=$(curl -s "$BASE_URL/api/v1/sync/pull?device_id=$DEVICE_ID&data_type=todos" \
  -H "Authorization: Bearer $TOKEN")
AFTER_COUNT=$(echo "$AFTER" | grep -c "^  - title:")

echo "   真正 push 后数据库状态: $AFTER_COUNT 条"
assert_eq "$AFTER_COUNT" "5" "执行后数据正确更新"
assert_not_contains "$BEFORE" "买菜 (1)" "Dry run 前无重命名项"
assert_contains "$AFTER" "买菜 (1)" "执行后出现重命名项"
echo ""

# ========== 测试5: 三种模式对比 ==========
echo "========================================"
echo "  测试5: 三种模式预览对比"
echo "========================================"
echo ""

# Skip 模式
SKIP_RESULT=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  --data-binary @- << YAML
device_id: $DEVICE_ID
data_type: todos
conflict_mode: skip
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
YAML
)

echo "【5.1】Skip 模式识别"
assert_contains "$SKIP_RESULT" "action: skip" "识别 skip 动作"
assert_eq "$(echo "$SKIP_RESULT" | grep -A1 'skipped:' | tail -1 | tr -d ' ')" "skipped: 1" "跳过数量为 1"
assert_eq "$(echo "$SKIP_RESULT" | grep -A1 'final_total:' | tail -1 | tr -d ' ')" "final_total: 5" "skip 后总数不变 (5)"
echo ""

# Overwrite 模式
OVERWRITE_RESULT=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  --data-binary @- << YAML
device_id: $DEVICE_ID
data_type: todos
conflict_mode: overwrite
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
YAML
)

echo "【5.2】Overwrite 模式识别"
assert_contains "$OVERWRITE_RESULT" "action: overwrite" "识别 overwrite 动作"
assert_eq "$(echo "$OVERWRITE_RESULT" | grep -A1 'overwritten:' | tail -1 | tr -d ' ')" "overwritten: 1" "覆盖数量为 1"
# overwrite 后买菜被替换，所以总数仍为 5
assert_eq "$(echo "$OVERWRITE_RESULT" | grep -A1 'final_total:' | tail -1 | tr -d ' ')" "final_total: 5" "overwrite 后总数不变 (5)"
echo ""

# ========== 测试6: YAML 美观度 ==========
echo "========================================"
echo "  测试6: YAML 输出格式检查"
echo "========================================"
echo ""

echo "【6.1】检查缩进格式"
assert_contains "$DRYRUN_RESULT" "    - title:" "YAML 列表项缩进正确"
assert_contains "$DRYRUN_RESULT" "      status:" "YAML 字段缩进正确"
echo ""

echo "【6.2】检查 null 值处理"
assert_contains "$DRYRUN_RESULT" "null" "null 值被正确序列化"
echo ""

# ========== 总结 ==========
echo "========================================"
echo "  测试完成"
echo "========================================"
echo ""
echo -e "${GREEN}通过: $PASS${NC}"
echo -e "${RED}失败: $FAIL${NC}"
echo ""

if [ "$FAIL" -eq 0 ]; then
  echo -e "${GREEN}所有测试通过！${NC}"
  echo ""
  echo "========================================"
  echo "  Dry Run 输出示例"
  echo "========================================"
  echo "$DRYRUN_RESULT"
else
  echo -e "${RED}有 $FAIL 个测试失败${NC}"
fi

# 清理
pkill -f "ntd-cloud-server" 2>/dev/null || true
exit $FAIL