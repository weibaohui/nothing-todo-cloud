/**
 * ntd-cloud Token 管理页面
 *
 * 设计特点：
 * - 桌面端：表格布局
 * - 移动端：卡片列表，操作按钮可靠近点击
 * - 复制/撤销功能正常可用
 */
import React, { useEffect, useState } from 'react';
import {
  Card, Table, Button, Modal, Form, Input, message, Space, Typography, Alert, Empty, Flex, Tooltip
} from 'antd';
import {
  PlusOutlined,
  CopyOutlined,
  DeleteOutlined,
  KeyOutlined,
  CheckCircleOutlined,
} from '@ant-design/icons';
import { tokens } from '../api/client';

const { Text } = Typography;

interface Token {
  id: number;
  name: string;
  token?: string;
  last_used_at: string | null;
  created_at: string;
}

/** 判断是否为移动端 */
const useIsMobile = () => {
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);
  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener('resize', handleResize, { passive: true });
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  return isMobile;
};

const Tokens: React.FC = () => {
  const [data, setData] = useState<Token[]>([]);
  const [modalVisible, setModalVisible] = useState(false);
  const [newToken, setNewToken] = useState<string | null>(null);
  const [revokeConfirm, setRevokeConfirm] = useState<number | null>(null);
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const isMobile = useIsMobile();

  // 加载 Token 列表
  const loadTokens = async () => {
    try {
      const res = await tokens.list();
      setData(res.data || []);
    } catch (err) {
      console.error(err);
      message.error('加载 Token 失败');
    }
  };

  useEffect(() => { loadTokens(); }, []);

  // 创建新 Token
  const handleCreate = async () => {
    try {
      await form.validateFields();
      setLoading(true);
      const res = await tokens.create(form.getFieldValue('name'));
      setNewToken(res.data.token);
      message.success('Token 创建成功，请及时复制保存');
      form.resetFields();
      loadTokens();
    } catch (err) {
      console.error(err);
      message.error('创建失败');
    } finally {
      setLoading(false);
    }
  };

  // 撤销 Token
  const handleRevoke = async (id: number) => {
    try {
      await tokens.revoke(id);
      message.success('Token 已撤销');
      loadTokens();
    } catch (err) {
      console.error(err);
      message.error('撤销失败');
    }
  };

  // 复制 Token - 使用兼容方法
  const copyToken = (token: string) => {
    // 优先使用现代 Clipboard API
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(token).then(() => {
        message.success('已复制到剪贴板');
      }).catch(() => {
        // Fallback: 使用 execCommand
        fallbackCopy(token);
      });
    } else {
      // Fallback: 使用 execCommand
      fallbackCopy(token);
    }
  };

  // execCommand 备用复制方法
  const fallbackCopy = (token: string) => {
    const textarea = document.createElement('textarea');
    textarea.value = token;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    try {
      document.execCommand('copy');
      message.success('已复制到剪贴板');
    } catch (err) {
      message.error('复制失败，请手动复制');
    }
    document.body.removeChild(textarea);
  };

  // 关闭弹窗时重置状态
  const handleModalClose = () => {
    setModalVisible(false);
    setNewToken(null);
    form.resetFields();
  };

  // 桌面端表格列配置
  const columns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 60,
      render: (id: number) => <Text type="secondary">#{id}</Text>,
    },
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      render: (name: string) => (
        <Space size={4}>
          <KeyOutlined style={{ color: 'var(--color-primary)' }} />
          <Text strong>{name}</Text>
        </Space>
      ),
    },
    {
      title: '最后使用',
      dataIndex: 'last_used_at',
      key: 'last_used_at',
      width: 150,
      render: (v: string | null) => (
        <Text type="secondary" style={{ fontSize: 12 }}>
          {v ? new Date(v).toLocaleString('zh-CN') : '从未使用'}
        </Text>
      ),
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 150,
      render: (t: string) => (
        <Text type="secondary" style={{ fontSize: 12 }}>
          {new Date(t).toLocaleString('zh-CN')}
        </Text>
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: 180,
      render: (_: any, record: Token) => (
        <Space size={4}>
          <Tooltip title={record.token ? '复制 Token' : 'Token 仅创建时显示，请重新创建'}>
            <Button
              icon={<CopyOutlined />}
              onClick={() => copyToken(record.token || `ntd_${record.id}`)}
              size="small"
              style={{ borderRadius: 'var(--radius-sm)' }}
            >
              复制
            </Button>
          </Tooltip>
          <Button
            icon={<DeleteOutlined />}
            danger
            size="small"
            onClick={() => setRevokeConfirm(record.id)}
            style={{ borderRadius: 'var(--radius-sm)' }}
          >
            撤销
          </Button>
        </Space>
      ),
    },
  ];

  // 移动端卡片渲染
  const renderMobileCard = (record: Token) => (
    <Card
      key={record.id}
      size="small"
      style={{
        marginBottom: 12,
        borderRadius: 'var(--radius-lg)',
        border: '1px solid var(--color-border-light)',
      }}
      bodyStyle={{ padding: 16 }}
    >
      <Space direction="vertical" size="middle" style={{ width: '100%' }}>
        {/* 标题行 */}
        <Flex justify="space-between" align="center">
          <Space size={8}>
            <KeyOutlined style={{ color: 'var(--color-primary)' }} />
            <Text strong>{record.name}</Text>
          </Space>
          <Text type="secondary" style={{ fontSize: 12 }}>#{record.id}</Text>
        </Flex>

        {/* 时间信息 */}
        <Space direction="vertical" size={4} style={{ width: '100%' }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            最后使用: {record.last_used_at ? new Date(record.last_used_at).toLocaleString('zh-CN') : '从未使用'}
          </Text>
          <Text type="secondary" style={{ fontSize: 12 }}>
            创建: {new Date(record.created_at).toLocaleString('zh-CN')}
          </Text>
        </Space>

        {/* 操作按钮 */}
        <Space size={8}>
          <Button
            type="primary"
            icon={<CopyOutlined />}
            onClick={() => copyToken(record.token || `ntd_${record.id}`)}
            size="small"
            style={{ borderRadius: 'var(--radius-md)' }}
          >
            复制 Token
          </Button>
          <Button
            icon={<DeleteOutlined />}
            danger
            size="small"
            onClick={() => setRevokeConfirm(record.id)}
            style={{ borderRadius: 'var(--radius-md)' }}
          >
            撤销
          </Button>
        </Space>
      </Space>
    </Card>
  );

  return (
    <div className="animate-fade-in">
      {/* 页面标题 */}
      <div style={{ marginBottom: 24 }}>
        <h2 style={{ fontSize: 22, fontWeight: 600, marginBottom: 4 }}>API Token 管理</h2>
        <Text type="secondary">管理用于访问 API 的 Token 凭证</Text>
      </div>

      {/* 移动端：卡片列表 */}
      {isMobile ? (
        <div>
          {/* 移动端顶部操作栏 */}
          <Card
            size="small"
            style={{ marginBottom: 12, borderRadius: 'var(--radius-lg)' }}
            bodyStyle={{ padding: '12px 16px' }}
          >
            <Flex justify="space-between" align="center">
              <Text strong>Token 列表</Text>
              <Button
                type="primary"
                icon={<PlusOutlined />}
                size="small"
                onClick={() => { setModalVisible(true); setNewToken(null); }}
                style={{ borderRadius: 'var(--radius-md)' }}
              >
                新建
              </Button>
            </Flex>
          </Card>

          {/* 卡片列表 */}
          {data.length === 0 ? (
            <Empty description="暂无 Token" />
          ) : (
            data.map(renderMobileCard)
          )}
        </div>
      ) : (
        /* 桌面端：表格 */
        <Card
          title={
            <Space size={8}>
              <KeyOutlined style={{ color: 'var(--color-primary)' }} />
              <span>Token 列表</span>
            </Space>
          }
          extra={
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => { setModalVisible(true); setNewToken(null); }}
              style={{ borderRadius: 'var(--radius-md)' }}
            >
              创建 Token
            </Button>
          }
          style={{ borderRadius: 'var(--radius-lg)' }}
          bodyStyle={{ padding: 0 }}
        >
          <Table
            columns={columns}
            dataSource={data}
            rowKey="id"
            tableLayout="fixed"
            pagination={{
              pageSize: 10,
              showSizeChanger: false,
              showTotal: (total) => `共 ${total} 个 Token`,
            }}
            locale={{ emptyText: '暂无 Token，请创建一个' }}
          />
        </Card>
      )}

      {/* 创建 Token 弹窗 */}
      <Modal
        title={
          <Space size={8}>
            <PlusOutlined style={{ color: 'var(--color-primary)' }} />
            <span>创建新 Token</span>
          </Space>
        }
        open={modalVisible}
        onOk={newToken ? handleModalClose : handleCreate}
        onCancel={handleModalClose}
        okText={newToken ? '完成' : '创建'}
        cancelText="取消"
        loading={loading}
        width={isMobile ? '95vw' : 480}
      >
        {newToken ? (
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Alert
              message="Token 创建成功"
              description="请立即复制并保存 Token，关闭此窗口后将无法再次查看完整内容"
              type="success"
              showIcon
              icon={<CheckCircleOutlined />}
              style={{ borderRadius: 'var(--radius-md)' }}
            />
            <Input
              value={newToken}
              readOnly
              size="large"
              style={{
                fontFamily: "'SF Mono', Monaco, monospace",
                background: 'var(--color-bg)',
              }}
              addonAfter={
                <Button onClick={() => copyToken(newToken)} size="small" style={{ margin: -8 }}>
                  复制
                </Button>
              }
            />
          </Space>
        ) : (
          <Form form={form} layout="vertical" style={{ marginTop: 16 }}>
            <Form.Item
              name="name"
              label="Token 名称"
              rules={[{ required: true, message: '请输入 Token 名称' }]}
            >
              <Input placeholder="如：Home Server、Docker Compose" />
            </Form.Item>
            <Text type="secondary" style={{ fontSize: 12 }}>
              创建后请妥善保存 Token，它只会显示一次
            </Text>
          </Form>
        )}
      </Modal>

      {/* 撤销 Token 确认弹窗 */}
      <Modal
        title="确定撤销 Token？"
        open={revokeConfirm !== null}
        onCancel={() => setRevokeConfirm(null)}
        onOk={() => {
          if (revokeConfirm !== null) {
            handleRevoke(revokeConfirm);
            setRevokeConfirm(null);
          }
        }}
        okText="撤销"
        okType="danger"
        cancelText="取消"
      >
        <p>撤销后该 Token 将立即失效，无法恢复。</p>
      </Modal>
    </div>
  );
};

export default Tokens;
