# UI 问题修复记录

## 问题：移动端 Popconfirm 触摸点击无反应

### 症状
- 在 iPhone Safari 浏览器上，点击删除/撤销等按钮时，Popconfirm 弹出框无法触发
- 触摸事件没有响应，按钮看起来可以点击但实际无效果
- Playwright 自动化测试能通过（使用 mouse.click），但真实手机触摸无法工作

### 根本原因
Ant Design 的 `Popconfirm` 组件在移动端 Safari 上存在触摸事件穿透问题。Popconfirm 的触发依赖于鼠标事件而非触摸事件，导致：
- 触摸点击按钮时，Popconfirm 无法正确捕获触摸事件
- 触摸事件被识别为"点击"但不触发 Popconfirm 的显示逻辑

### 解决方案
**用 Modal 确认框替代 Popconfirm**

移动端使用 Modal 组件代替 Popconfirm，Modal 在移动端有更好的触摸事件支持：

```tsx
// ❌ 错误 - Popconfirm 在移动端触摸有问题
<Popconfirm
  title="确定删除？"
  onConfirm={() => handleDelete(id)}
>
  <Button icon={<DeleteOutlined />}>删除</Button>
</Popconfirm>

// ✅ 正确 - 使用 Modal
const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);

<Button onClick={() => setDeleteConfirm(id)}>删除</Button>

<Modal
  title="确定删除？"
  open={deleteConfirm !== null}
  onOk={() => { handleDelete(deleteConfirm); setDeleteConfirm(null); }}
  onCancel={() => setDeleteConfirm(null)}
  okText="删除"
  okType="danger"
>
  <p>删除后无法恢复，确定要删除吗？</p>
</Modal>
```

### 涉及的页面
- `frontend/src/pages/Todos.tsx` - Todo 删除功能
- `frontend/src/pages/Tokens.tsx` - Token 撤销功能

### 注意事项
1. **所有移动端交互**：如果使用 Ant Design 的 Popconfirm、Tooltip 等依赖鼠标事件的组件，在移动端都可能有问题
2. **触摸目标大小**：移动端按钮最小尺寸建议 44x44 像素（符合 Apple HIG 标准）
3. **测试**：移动端功能必须用真实设备测试，不能只靠自动化测试

### 相关文档
- Apple Human Interface Guidelines: [Touch Bar and Gestures](https://developer.apple.com/design/human-interface-guidelines/touch-bar-and-gestures)
- Ant Design Mobile: [手势和触摸事件处理](https://ant.design/mobile)
