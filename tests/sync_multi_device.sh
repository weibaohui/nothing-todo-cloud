#!/bin/bash
# 多设备同步测试脚本

BASE_URL="http://localhost:8089"
EMAIL="test$(date +%s)@example.com"
PASSWORD="123456"

echo "=== 多设备同步测试 ==="
echo ""

# 1. 注册用户
echo "1. 注册用户: $EMAIL"
REG=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
TOKEN=$(echo $REG | grep -o '"token":"[^"]*"' | sed 's/"token":"//;s/"$//')
echo "   Token: ${TOKEN:0:50}..."

# 2. 创建设备 A 和 B
echo ""
echo "2. 创建设备 A"
DEVICE_A=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备A"}')
DEVICE_A_ID=$(echo $DEVICE_A | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
echo "   设备A ID: $DEVICE_A_ID"

echo ""
echo "3. 创建设备 B"
DEVICE_B=$(curl -s -X POST "$BASE_URL/api/devices" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"device_name":"设备B"}')
DEVICE_B_ID=$(echo $DEVICE_B | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
echo "   设备B ID: $DEVICE_B_ID"

# 4. 设备 A push todos
echo ""
echo "4. 设备A push todos"
A_TODOS="todos:
  - title: 买菜
  - title: 做饭
  - title: 洗澡"
A_PUSH=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  -d "$A_TODOS")
echo "   A push 结果:"
echo "$A_PUSH" | head -10 | sed 's/^/   /'

# 5. 设备 B push todos（有重叠）
echo ""
echo "5. 设备B push todos（有重叠）
B_TODOS="todos:
  - title: 买菜
  - title: 洗衣服
  - title: 跑步"
B_PUSH=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  -d "$B_TODOS")
echo "   B push 结果:"
echo "$B_PUSH" | head -10 | sed 's/^/   /'

# 6. 设备 A pull（验证合并）
echo ""
echo "6. 设备A pull（验证合并结果）"
A_PULL=$(curl -s "$BASE_URL/api/v1/sync/pull?device_id=$DEVICE_A_ID&data_type=todos" \
  -H "Authorization: Bearer $TOKEN")
echo "   A pull 结果:"
echo "$A_PULL" | head -15 | sed 's/^/   /'
if echo "$A_PULL" | grep -q "跑步" && echo "$A_PULL" | grep -q "洗衣服"; then
    echo "   ✓ 合并成功！包含A和B的并集"
else
    echo "   ❌ 合并失败"
fi

# 7. 测试 tags 同步
echo ""
echo "7. 设备A push tags"
A_TAGS="tags:
  - 工作
  - 生活"
A_TAGS_PUSH=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  -d "$A_TAGS")
echo "   A push tags 结果:"
echo "$A_TAGS_PUSH" | head -10 | sed 's/^/   /'

echo ""
echo "8. 设备B push tags（有重叠）"
B_TAGS="tags:
  - 工作
  - 学习"
B_TAGS_PUSH=$(curl -s -X POST "$BASE_URL/api/v1/sync/push" \
  -H "Content-Type: text/yaml" \
  -H "Authorization: Bearer $TOKEN" \
  -d "$B_TAGS")
echo "   B push tags 结果:"
echo "$B_TAGS_PUSH" | head -10 | sed 's/^/   /'
TAGS_COUNT=$(echo "$B_TAGS_PUSH" | grep -c "^- " || echo "0")
if [ "$TAGS_COUNT" -ge 3 ]; then
    echo "   ✓ tags 合并成功！包含A和B的并集"
else
    echo "   ❌ tags 合并失败"
fi

echo ""
echo "=== 测试完成 ==="
