/**
 * ntd-cloud Todo 管理页面
 *
 * 设计特点：
 * - 桌面端：左侧筛选 + 右侧列表 分栏布局
 * - 移动端：顶部筛选折叠 + 卡片列表
 * - 状态标签多彩设计
 * - 详情抽屉支持展开查看完整信息
 */
import React, { useEffect, useState, useMemo } from 'react';
import {
  Card, Tag, Typography, Modal, Form, Input, Select, Switch, Button,
  message, Space, Drawer, Radio, List, Flex, Badge, Empty, Tooltip
} from 'antd';
import {
  DeleteOutlined, EditOutlined, PlusOutlined, UploadOutlined,
  ScheduleOutlined, CheckCircleOutlined, ClockCircleOutlined, RobotOutlined,
  FilterOutlined, UnorderedListOutlined, AppstoreOutlined, CloseOutlined,
  CopyOutlined,
} from '@ant-design/icons';
import { todos } from '../api/client';

const { Text, Paragraph } = Typography;
const { TextArea } = Input;

/** 使用窗口尺寸判断是否为移动端 */
const useIsMobile = () => {
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);
  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener('resize', handleResize, { passive: true });
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  return isMobile;
};

/** Todo 条目接口 */
interface Todo {
  id: number;
  user_id: number;
  title: string;
  prompt?: string;
  status?: string;
  executor?: string;
  scheduler_enabled?: boolean;
  scheduler_config?: string;
  tag_names: string[];
  workspace?: string;
  worktree?: string;
  created_at: string;
  updated_at?: string;
}

/** 状态选项 */
const STATUS_OPTIONS = [
  { value: 'pending', label: '待处理', color: '#94A3B8', bg: '#F1F5F9' },
  { value: 'running', label: '执行中', color: '#3B82F6', bg: '#EFF6FF' },
  { value: 'completed', label: '已完成', color: '#10B981', bg: '#ECFDF5' },
  { value: 'failed', label: '失败', color: '#EF4444', bg: '#FEF2F2' },
];

const getStatusConfig = (status?: string) =>
  STATUS_OPTIONS.find(o => o.value === status) || STATUS_OPTIONS[0];

/** 状态图标组件 */
const StatusIcon: React.FC<{ status?: string; size?: number }> = ({ status, size = 18 }) => {
  const config = getStatusConfig(status);
  if (status === 'completed') return <CheckCircleOutlined style={{ color: config.color, fontSize: size }} />;
  if (status === 'running') return <ClockCircleOutlined style={{ color: config.color, fontSize: size }} />;
  return <ClockCircleOutlined style={{ color: '#D1D5DB', fontSize: size }} />;
};

/** 标签颜色池 */
const TAG_COLORS = ['blue', 'cyan', 'green', 'orange', 'purple', 'magenta', 'red', 'geekblue'];
const getTagColor = (tag: string) => TAG_COLORS[tag.charCodeAt(0) % TAG_COLORS.length];

const Todos: React.FC = () => {
  const [allTodos, setAllTodos] = useState<Todo[]>([]);
  const [loading, setLoading] = useState(true);
  const [editModal, setEditModal] = useState<Todo | null>(null);
  const [createModal, setCreateModal] = useState(false);
  const [detailDrawer, setDetailDrawer] = useState<Todo | null>(null);
  const [importModal, setImportModal] = useState(false);
  const [importYaml, setImportYaml] = useState('');
  const [importMode, setImportMode] = useState<'merge' | 'replace'>('merge');
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);
  const [form] = Form.useForm();
  const [filterVisible, setFilterVisible] = useState(false);
  const [filterTag, setFilterTag] = useState<string | null>(null);
  const [filterStatus, setFilterStatus] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'list' | 'card'>('list');
  const isMobile = useIsMobile();

  // 提取所有标签
  const allTags = useMemo(() => {
    const tagSet = new Set<string>();
    allTodos.forEach(t => t.tag_names?.forEach(tag => tagSet.add(tag)));
    return Array.from(tagSet).sort();
  }, [allTodos]);

  // 加载数据
  const loadData = () => {
    setLoading(true);
    todos.list()
      .then(res => setAllTodos(res.data.data || []))
      .catch(err => message.error('加载失败: ' + err.message))
      .finally(() => setLoading(false));
  };

  useEffect(() => { loadData(); }, []);

  // 过滤后的 Todos
  const filteredTodos = allTodos.filter(t => {
    if (filterTag && !t.tag_names?.includes(filterTag)) return false;
    if (filterStatus && t.status !== filterStatus) return false;
    return true;
  });

  // 打开编辑弹窗
  const openEdit = (todo: Todo) => {
    setDetailDrawer(null);
    setEditModal(todo);
    form.setFieldsValue({
      title: todo.title,
      prompt: todo.prompt,
      status: todo.status,
      executor: todo.executor,
      scheduler_enabled: todo.scheduler_enabled,
      scheduler_config: todo.scheduler_config,
      tag_names: todo.tag_names,
      workspace: todo.workspace,
      worktree: todo.worktree,
    });
  };

  // 保存编辑
  const handleSave = () => {
    if (!editModal) return;
    const values = form.getFieldsValue();
    todos.update(editModal.id, values)
      .then(() => {
        message.success('更新成功');
        setEditModal(null);
        form.resetFields();
        loadData();
      })
      .catch(err => message.error('更新失败: ' + err.message));
  };

  // 创建 Todo
  const handleCreate = () => {
    const values = form.getFieldsValue();
    todos.create(values)
      .then(() => {
        message.success('创建成功');
        setCreateModal(false);
        form.resetFields();
        loadData();
      })
      .catch(err => message.error('创建失败: ' + err.message));
  };

  // 删除 Todo
  const handleDelete = (id: number) => {
    todos.delete(id)
      .then(() => { message.success('删除成功'); loadData(); })
      .catch(err => message.error('删除失败: ' + err.message));
  };

  // 导入 YAML
  const handleImport = () => {
    if (!importYaml.trim()) { message.error('请输入 YAML 数据'); return; }
    todos.import(importYaml, importMode)
      .then((res: any) => {
        const data = res.data?.data || res.data || {};
        message.success(`导入完成: ${data.todos_imported || 0} 条 Todo`);
        setImportModal(false);
        setImportYaml('');
        loadData();
      })
      .catch(err => message.error('导入失败: ' + (err.message || '解析错误')));
  };

  // 清除筛选
  const clearFilters = () => {
    setFilterStatus(null);
    setFilterTag(null);
  };

  // 渲染状态标签
  const renderStatusTag = (status?: string) => {
    const config = getStatusConfig(status);
    return (
      <Tag
        style={{
          background: config.bg,
          color: config.color,
          border: 'none',
          borderRadius: 'var(--radius-sm)',
          fontWeight: 500,
          margin: 0,
        }}
      >
        {config.label}
      </Tag>
    );
  };

  // ========== 桌面端列表渲染 ==========
  const renderDesktopListItem = (todo: Todo) => {
    return (
      <List.Item
        style={{
          padding: '16px 20px',
          transition: 'background var(--transition-fast)',
          cursor: 'pointer',
        }}
        onClick={() => setDetailDrawer(todo)}
        actions={[
          <Button
            key="edit"
            type="text"
            size="small"
            icon={<EditOutlined />}
            onClick={(e) => { e.stopPropagation(); openEdit(todo); }}
            style={{ borderRadius: 'var(--radius-sm)' }}
          >
            编辑
          </Button>,
          <Button
            key="delete"
            type="text"
            size="small"
            danger
            icon={<DeleteOutlined />}
            onClick={(e) => { e.stopPropagation(); setDeleteConfirm(todo.id); }}
            style={{ borderRadius: 'var(--radius-sm)' }}
          >
            删除
          </Button>,
        ]}
      >
        <List.Item.Meta
          avatar={
            <div style={{
              width: 40,
              height: 40,
              borderRadius: 'var(--radius-md)',
              background: getStatusConfig(todo.status).bg,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}>
              <StatusIcon status={todo.status} size={20} />
            </div>
          }
          title={
            <Flex justify="space-between" align="center" gap={8}>
              <Text strong style={{ fontSize: 15, flex: 1 }} ellipsis={{ tooltip: todo.title }}>
                {todo.title}
              </Text>
              {renderStatusTag(todo.status)}
            </Flex>
          }
          description={
            <Flex gap={6} wrap align="center" style={{ marginTop: 6 }}>
              {todo.scheduler_enabled && (
                <Tag
                  icon={<ScheduleOutlined />}
                  style={{
                    background: '#EFF6FF',
                    color: '#3B82F6',
                    border: 'none',
                    borderRadius: 'var(--radius-sm)',
                    margin: 0,
                  }}
                >
                  定时
                </Tag>
              )}
              {todo.executor && (
                <Tag
                  icon={<RobotOutlined />}
                  style={{
                    background: '#F5F5F5',
                    color: '#666',
                    border: 'none',
                    borderRadius: 'var(--radius-sm)',
                    margin: 0,
                  }}
                >
                  {todo.executor}
                </Tag>
              )}
              {todo.tag_names.slice(0, 3).map(tag => (
                <Tag
                  key={tag}
                  color={getTagColor(tag)}
                  style={{ borderRadius: 'var(--radius-sm)', margin: 0 }}
                >
                  {tag}
                </Tag>
              ))}
              {todo.tag_names.length > 3 && (
                <Text type="secondary" style={{ fontSize: 12 }}>
                  +{todo.tag_names.length - 3}
                </Text>
              )}
            </Flex>
          }
        />
      </List.Item>
    );
  };

  // ========== 移动端卡片渲染 ==========
  const renderMobileCard = (todo: Todo) => {
    return (
      <Card
        key={todo.id}
        size="small"
        style={{
          marginBottom: 10,
          borderRadius: 'var(--radius-lg)',
          border: '1px solid var(--color-border-light)',
        }}
        bodyStyle={{ padding: 14 }}
        onClick={() => setDetailDrawer(todo)}
        hoverable
      >
        <Flex justify="space-between" align="flex-start" gap={10}>
          <Flex gap={10} align="flex-start" style={{ flex: 1, minWidth: 0 }}>
            <div style={{
              width: 36,
              height: 36,
              borderRadius: 'var(--radius-sm)',
              background: getStatusConfig(todo.status).bg,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}>
              <StatusIcon status={todo.status} size={18} />
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <Text strong style={{ fontSize: 14, display: 'block', marginBottom: 4 }} ellipsis={{ tooltip: todo.title }}>
                {todo.title}
              </Text>
              <Flex gap={4} wrap>
                {renderStatusTag(todo.status)}
                {todo.scheduler_enabled && (
                  <Tag icon={<ScheduleOutlined />} style={{ background: '#EFF6FF', color: '#3B82F6', border: 'none', borderRadius: 'var(--radius-sm)', margin: 0, fontSize: 11 }}>
                    定时
                  </Tag>
                )}
              </Flex>
            </div>
          </Flex>
          {/* 移动端操作按钮 - 增大触摸目标 */}
          <Flex gap={8}>
            <Button
              type="text"
              size="middle"
              icon={<EditOutlined />}
              onClick={(e) => { e.stopPropagation(); openEdit(todo); }}
              style={{ minWidth: 44, minHeight: 44, borderRadius: 'var(--radius-md)' }}
            />
            <Button
              type="text"
              size="middle"
              danger
              icon={<DeleteOutlined />}
              onClick={(e) => { e.stopPropagation(); setDeleteConfirm(todo.id); }}
              style={{ minWidth: 44, minHeight: 44, borderRadius: 'var(--radius-md)' }}
            />
          </Flex>
        </Flex>
        {todo.tag_names.length > 0 && (
          <Flex gap={4} wrap style={{ marginTop: 8 }}>
            {todo.tag_names.slice(0, 4).map(tag => (
              <Tag key={tag} color={getTagColor(tag)} style={{ borderRadius: 'var(--radius-sm)', margin: 0, fontSize: 11 }}>
                {tag}
              </Tag>
            ))}
            {todo.tag_names.length > 4 && (
              <Text type="secondary" style={{ fontSize: 11 }}>+{todo.tag_names.length - 4}</Text>
            )}
          </Flex>
      )}
      </Card>
    );
  };

  // ========== 详情抽屉内容 ==========
  const renderDetailContent = (todo: Todo) => {
    return (
      <Space direction="vertical" size="middle" style={{ width: '100%' }}>
        {/* 标题和状态 */}
        <div>
          <Flex justify="space-between" align="center">
            <Text type="secondary" style={{ fontSize: 12 }}>标题</Text>
            {renderStatusTag(todo.status)}
          </Flex>
          <div style={{ fontSize: 18, fontWeight: 600, marginTop: 4 }}>{todo.title}</div>
        </div>

        {/* 执行器信息 */}
        {(todo.executor || todo.scheduler_enabled) && (
          <Flex gap={8} wrap>
            {todo.executor && (
              <Tag icon={<RobotOutlined />} style={{ borderRadius: 'var(--radius-sm)' }}>
                执行器: {todo.executor}
              </Tag>
            )}
            {todo.scheduler_enabled && (
              <Tag icon={<ScheduleOutlined />} color="blue" style={{ borderRadius: 'var(--radius-sm)' }}>
                定时: {todo.scheduler_config || '已启用'}
              </Tag>
            )}
          </Flex>
        )}

        {/* 标签 */}
        {todo.tag_names.length > 0 && (
          <div>
            <Text type="secondary" style={{ fontSize: 12 }}>标签</Text>
            <Flex gap={4} wrap style={{ marginTop: 6 }}>
              {todo.tag_names.map(tag => (
                <Tag key={tag} color={getTagColor(tag)} style={{ borderRadius: 'var(--radius-sm)' }}>
                  {tag}
                </Tag>
              ))}
            </Flex>
          </div>
        )}

        {/* Prompt */}
        {todo.prompt && (
          <div>
            <Text type="secondary" style={{ fontSize: 12 }}>Prompt</Text>
            <Paragraph
              copyable
              style={{
                marginTop: 6,
                fontFamily: "'SF Mono', Monaco, monospace",
                fontSize: 12,
                background: 'var(--color-bg)',
                padding: 12,
                borderRadius: 'var(--radius-md)',
                whiteSpace: 'pre-wrap',
              }}
            >
              {todo.prompt}
            </Paragraph>
          </div>
        )}

        {/* 工作空间 */}
        {(todo.workspace || todo.worktree) && (
          <div>
            <Text type="secondary" style={{ fontSize: 12 }}>环境</Text>
            <Flex gap={8} wrap style={{ marginTop: 6 }}>
              {todo.workspace && (
                <Tag icon={<CopyOutlined />} style={{ borderRadius: 'var(--radius-sm)' }}>
                  {todo.workspace}
                </Tag>
              )}
              {todo.worktree && (
                <Tag style={{ borderRadius: 'var(--radius-sm)' }}>
                  {todo.worktree}
                </Tag>
              )}
            </Flex>
          </div>
        )}

        {/* 时间信息 */}
        <div style={{ borderTop: '1px solid var(--color-border-light)', paddingTop: 12 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            创建于 {new Date(todo.created_at).toLocaleString('zh-CN')}
          </Text>
          {todo.updated_at && todo.updated_at !== todo.created_at && (
            <Text type="secondary" style={{ fontSize: 12, display: 'block', marginTop: 2 }}>
              更新于 {new Date(todo.updated_at).toLocaleString('zh-CN')}
            </Text>
          )}
        </div>

        {/* 操作按钮 */}
        <Flex gap={8}>
          <Button
            type="primary"
            icon={<EditOutlined />}
            onClick={() => openEdit(todo)}
            style={{ borderRadius: 'var(--radius-md)' }}
          >
            编辑
          </Button>
          <Button
            danger
            icon={<DeleteOutlined />}
            onClick={() => { setDetailDrawer(null); setDeleteConfirm(todo.id); }}
            style={{ borderRadius: 'var(--radius-md)' }}
          >
            删除
          </Button>
        </Flex>
      </Space>
    );
  };

  // ========== 筛选栏 ==========
  const renderFilterBar = () => (
    <Flex gap={8} wrap align="center">
      <Select
        style={{ minWidth: 100 }}
        placeholder="状态"
        allowClear
        value={filterStatus}
        onChange={v => setFilterStatus(v || null)}
        size="middle"
      >
        {STATUS_OPTIONS.map(opt => (
          <Select.Option key={opt.value} value={opt.value}>
            <Flex gap={6} align="center">
              <div style={{ width: 8, height: 8, borderRadius: '50%', background: opt.color }} />
              {opt.label}
            </Flex>
          </Select.Option>
        ))}
      </Select>
      <Select
        style={{ minWidth: 120 }}
        placeholder="标签"
        allowClear
        value={filterTag}
        onChange={v => setFilterTag(v || null)}
        size="middle"
      >
        {allTags.map(tag => (
          <Select.Option key={tag} value={tag}>
            <Tag color={getTagColor(tag)} style={{ borderRadius: 'var(--radius-sm)', margin: 0 }}>
              {tag}
            </Tag>
          </Select.Option>
        ))}
      </Select>
      {(filterStatus || filterTag) && (
        <Button
          size="middle"
          icon={<CloseOutlined />}
          onClick={clearFilters}
          style={{ borderRadius: 'var(--radius-md)' }}
        >
          清除
        </Button>
      )}
    </Flex>
  );

  // ========== 列表头部 ==========
  const renderListHeader = () => (
    <Flex justify="space-between" align="center" wrap gap={12}>
      <Flex gap={8} align="center">
        <Text strong style={{ fontSize: 16 }}>Todo 列表</Text>
        <Badge
          count={filteredTodos.length}
          style={{ backgroundColor: 'var(--color-primary)' }}
        />
      </Flex>
      <Flex gap={8} align="center">
        {/* 视图切换 - 桌面端 */}
        {!isMobile && (
          <Space size={4}>
            <Tooltip title="列表视图">
              <Button
                type={viewMode === 'list' ? 'primary' : 'text'}
                icon={<UnorderedListOutlined />}
                size="middle"
                onClick={() => setViewMode('list')}
                style={{ borderRadius: 'var(--radius-sm)' }}
              />
            </Tooltip>
            <Tooltip title="卡片视图">
              <Button
                type={viewMode === 'card' ? 'primary' : 'text'}
                icon={<AppstoreOutlined />}
                size="middle"
                onClick={() => setViewMode('card')}
                style={{ borderRadius: 'var(--radius-sm)' }}
              />
            </Tooltip>
          </Space>
        )}
        <Button
          icon={<FilterOutlined />}
          onClick={() => setFilterVisible(!filterVisible)}
          type={filterVisible ? 'primary' : 'default'}
          style={{ borderRadius: 'var(--radius-md)' }}
        >
          筛选
        </Button>
        <Button
          icon={<UploadOutlined />}
          onClick={() => setImportModal(true)}
          style={{ borderRadius: 'var(--radius-md)' }}
        >
          导入
        </Button>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setCreateModal(true)}
          style={{ borderRadius: 'var(--radius-md)' }}
        >
          {isMobile ? '新建' : '新建 Todo'}
        </Button>
      </Flex>
    </Flex>
  );

  return (
    <div className="animate-fade-in" style={{ padding: isMobile ? 0 : 0 }}>
      {/* 移动端顶部栏 */}
      {isMobile && (
        <Flex
          justify="space-between"
          align="center"
          style={{
            padding: '12px 16px',
            background: '#fff',
            marginBottom: 8,
            borderRadius: 'var(--radius-lg)',
          }}
        >
          <Flex gap={8} align="center">
            <Text strong style={{ fontSize: 16 }}>Todo</Text>
            <Badge count={filteredTodos.length} style={{ backgroundColor: 'var(--color-primary)' }} />
          </Flex>
          <Space>
            <Button
              size="small"
              icon={<FilterOutlined />}
              onClick={() => setFilterVisible(!filterVisible)}
            />
            <Button
              type="primary"
              size="small"
              icon={<PlusOutlined />}
              onClick={() => setCreateModal(true)}
            />
          </Space>
        </Flex>
      )}

      {/* 筛选区域 */}
      {(filterVisible || !isMobile) && (
        <Card
          size="small"
          style={{
            marginBottom: 12,
            borderRadius: isMobile ? 'var(--radius-lg)' : 'var(--radius-lg)',
          }}
          bodyStyle={{ padding: isMobile ? 12 : 16 }}
        >
          <Flex gap={12} align={isMobile ? 'stretch' : 'center'} style={{ flexDirection: isMobile ? 'column' : 'row' }}>
            {!isMobile && renderListHeader()}
            {renderFilterBar()}
          </Flex>
        </Card>
      )}

      {/* Todo 列表 */}
      {isMobile ? (
        // 移动端：卡片列表
        <div style={{ padding: '0 16px' }}>
          {filteredTodos.length === 0 && !loading ? (
            <Empty description="暂无 Todo" style={{ padding: '40px 0' }} />
          ) : (
            <List
              dataSource={filteredTodos}
              renderItem={renderMobileCard}
              loading={loading}
              locale={{ emptyText: '暂无数据' }}
            />
          )}
        </div>
      ) : viewMode === 'list' ? (
        // 桌面端：列表视图
        <Card style={{ borderRadius: 'var(--radius-lg)' }} bodyStyle={{ padding: 0 }}>
          {renderListHeader()}
          <List
            dataSource={filteredTodos}
            renderItem={renderDesktopListItem}
            loading={loading}
            locale={{ emptyText: '暂无 Todo，请创建一个' }}
            pagination={{
              pageSize: 15,
              showSizeChanger: false,
              showTotal: (total) => `共 ${total} 条`,
            }}
          />
        </Card>
      ) : (
        // 桌面端：卡片网格视图
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 12 }}>
          {filteredTodos.map(todo => (
            <Card
              key={todo.id}
              style={{
                borderRadius: 'var(--radius-lg)',
                border: '1px solid var(--color-border-light)',
                cursor: 'pointer',
              }}
              hoverable
              onClick={() => setDetailDrawer(todo)}
              bodyStyle={{ padding: 16 }}
            >
              <Flex justify="space-between" align="flex-start" gap={10}>
                <div style={{
                  width: 40,
                  height: 40,
                  borderRadius: 'var(--radius-md)',
                  background: getStatusConfig(todo.status).bg,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}>
                  <StatusIcon status={todo.status} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <Text strong ellipsis={{ tooltip: todo.title }} style={{ display: 'block', marginBottom: 6 }}>
                    {todo.title}
                  </Text>
                  {renderStatusTag(todo.status)}
                </div>
              </Flex>
              {todo.tag_names.length > 0 && (
                <Flex gap={4} wrap style={{ marginTop: 10 }}>
                  {todo.tag_names.slice(0, 3).map(tag => (
                    <Tag key={tag} color={getTagColor(tag)} style={{ borderRadius: 'var(--radius-sm)', margin: 0, fontSize: 11 }}>
                      {tag}
                    </Tag>
                  ))}
                  {todo.tag_names.length > 3 && (
                    <Text type="secondary" style={{ fontSize: 11 }}>+{todo.tag_names.length - 3}</Text>
                  )}
                </Flex>
              )}
              {/* 桌面端卡片操作按钮 */}
              <Flex gap={4} style={{ marginTop: 12 }}>
                <Button
                  type="text"
                  size="small"
                  icon={<EditOutlined />}
                  onClick={(e) => { e.stopPropagation(); openEdit(todo); }}
                  style={{ borderRadius: 'var(--radius-sm)' }}
                >
                  编辑
                </Button>
                <Button
                  type="text"
                  size="small"
                  danger
                  icon={<DeleteOutlined />}
                  onClick={(e) => { e.stopPropagation(); setDeleteConfirm(todo.id); }}
                  style={{ borderRadius: 'var(--radius-sm)' }}
                >
                  删除
                </Button>
              </Flex>
            </Card>
          ))}
        </div>
      )}

      {/* 创建/编辑弹窗 */}
      <Modal
        title={
          <Space size={8}>
            <UnorderedListOutlined style={{ color: 'var(--color-primary)' }} />
            <span>{editModal ? '编辑 Todo' : '新建 Todo'}</span>
          </Space>
        }
        open={createModal || !!editModal}
        onCancel={() => { setCreateModal(false); setEditModal(null); form.resetFields(); }}
        onOk={editModal ? handleSave : handleCreate}
        okText={editModal ? '保存' : '创建'}
        cancelText="取消"
        width={isMobile ? '95vw' : 520}
        destroyOnClose
        styles={{ body: { paddingTop: 16 } }}
      >
        <Form form={form} layout="vertical" size="middle">
          <Form.Item name="title" label="标题" rules={[{ required: true, message: '请输入标题' }]}>
            <Input placeholder="Todo 标题" />
          </Form.Item>
          <Form.Item name="prompt" label="Prompt">
            <TextArea rows={3} placeholder="Prompt 内容" />
          </Form.Item>
          <Form.Item name="status" label="状态" initialValue="pending">
            <Select>
              {STATUS_OPTIONS.map(opt => (
                <Select.Option key={opt.value} value={opt.value}>
                  <Flex gap={6} align="center">
                    <div style={{ width: 8, height: 8, borderRadius: '50%', background: opt.color }} />
                    {opt.label}
                  </Flex>
                </Select.Option>
              ))}
            </Select>
          </Form.Item>
          <Form.Item name="executor" label="执行器">
            <Input placeholder="如: claude-code" />
          </Form.Item>
          <Form.Item name="scheduler_enabled" label="启用定时" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item name="scheduler_config" label="定时配置" rules={[
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (getFieldValue('scheduler_enabled') && !value) {
                  return Promise.reject('请输入定时配置');
                }
                return Promise.resolve();
              },
            }),
          ]}>
            <Input placeholder="如: 0 6 * * *" />
          </Form.Item>
          <Form.Item name="tag_names" label="标签">
            <Select mode="tags" placeholder="输入标签后按回车">
              {allTags.map(tag => (
                <Select.Option key={tag} value={tag}>
                  <Tag color={getTagColor(tag)} style={{ borderRadius: 'var(--radius-sm)', margin: 0 }}>
                    {tag}
                  </Tag>
                </Select.Option>
              ))}
            </Select>
          </Form.Item>
          <Form.Item name="workspace" label="工作空间">
            <Input placeholder="工作目录" />
          </Form.Item>
          <Form.Item name="worktree" label="Worktree">
            <Input placeholder="Git worktree" />
          </Form.Item>
        </Form>
      </Modal>

      {/* 导入弹窗 */}
      <Modal
        title={
          <Space size={8}>
            <UploadOutlined style={{ color: 'var(--color-primary)' }} />
            <span>导入 YAML</span>
          </Space>
        }
        open={importModal}
        onCancel={() => { setImportModal(false); setImportYaml(''); }}
        onOk={handleImport}
        okText="导入"
        cancelText="取消"
        width={isMobile ? '95vw' : 600}
        destroyOnClose
      >
        <Space direction="vertical" style={{ width: '100%' }} size="middle">
          <Radio.Group value={importMode} onChange={e => setImportMode(e.target.value)}>
            <Radio value="merge">合并模式</Radio>
            <Radio value="replace">替换模式</Radio>
          </Radio.Group>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {importMode === 'merge' ? '新增 Todo 与现有数据合并' : '导入将替换所有现有 Todo'}
          </Text>
          <TextArea
            rows={10}
            placeholder="粘贴 YAML 数据..."
            value={importYaml}
            onChange={e => setImportYaml(e.target.value)}
            style={{
              fontFamily: "'SF Mono', Monaco, monospace",
              fontSize: 12,
            }}
          />
        </Space>
      </Modal>

      {/* 删除确认弹窗 */}
      <Modal
        title="确定删除？"
        open={deleteConfirm !== null}
        onCancel={() => setDeleteConfirm(null)}
        onOk={() => {
          if (deleteConfirm !== null) {
            handleDelete(deleteConfirm);
            setDeleteConfirm(null);
          }
        }}
        okText="删除"
        okType="danger"
        cancelText="取消"
      >
        <p>删除后无法恢复，确定要删除吗？</p>
      </Modal>

      {/* 详情抽屉 */}
      <Drawer
        title={
          <Space size={8}>
            <UnorderedListOutlined style={{ color: 'var(--color-primary)' }} />
            <span>Todo 详情</span>
          </Space>
        }
        placement="right"
        width={isMobile ? '100vw' : 420}
        onClose={() => setDetailDrawer(null)}
        open={!!detailDrawer}
        styles={{ body: { padding: 20 } }}
      >
        {detailDrawer && renderDetailContent(detailDrawer)}
      </Drawer>
    </div>
  );
};

export default Todos;
