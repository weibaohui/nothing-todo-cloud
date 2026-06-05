/**
 * ntd-cloud 控制台页面
 *
 * 设计特点：
 * - 统计卡片网格布局，桌面端 2 列
 * - 渐变色图标，数字跳动效果
 * - 同步记录表格，圆角设计
 */
import React, { useEffect, useState } from 'react';
import {
  Card, Row, Col, Table, Tag, Typography, Modal, Descriptions,
  Space, Empty
} from 'antd';
import {
  UserOutlined,
  SwapOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  CloudOutlined,
  HistoryOutlined,
} from '@ant-design/icons';
import { admin } from '../api/client';

const { Text, Paragraph } = Typography;

/** 同步日志条目 */
interface SyncLog {
  id: number;
  user_id: number;
  action: string;
  status: string;
  details: string | null;
  created_at: string;
}

/** 操作类型标签映射 */
const actionLabels: Record<string, string> = {
  push: '上传 (Push)',
  pull: '下载 (Pull)',
  merge: '合并',
  sync: '同步',
};

/** 统计卡片属性 */
interface StatCardProps {
  title: string;
  value: number;
  icon: React.ReactNode;
  color: string;
  suffix?: string;
}

/** 统计卡片组件 */
const StatCard: React.FC<StatCardProps> = ({ title, value, icon, color, suffix }) => (
  <Card
    style={{
      borderRadius: 'var(--radius-lg)',
      border: '1px solid var(--color-border-light)',
    }}
    bodyStyle={{ padding: '20px 24px' }}
  >
    <Space size={16} align="start">
      {/* 渐变背景图标 */}
      <div style={{
        width: 52,
        height: 52,
        borderRadius: 'var(--radius-md)',
        background: `linear-gradient(135deg, ${color}15 0%, ${color}25 100%)`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexShrink: 0,
      }}>
        <div style={{ color }}>{icon}</div>
      </div>
      {/* 数值 */}
      <div>
        <Text type="secondary" style={{ fontSize: 13, display: 'block', marginBottom: 2 }}>
          {title}
        </Text>
        <div style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, color: 'var(--color-text)' }}>
          {value.toLocaleString()}{suffix && <span style={{ fontSize: 14, fontWeight: 400, marginLeft: 4, color: 'var(--color-text-secondary)' }}>{suffix}</span>}
        </div>
      </div>
    </Space>
  </Card>
);

const Dashboard: React.FC = () => {
  const [stats, setStats] = useState({ total_users: 0, total_syncs: 0 });
  const [logs, setLogs] = useState<SyncLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [detailModal, setDetailModal] = useState<SyncLog | null>(null);

  // 加载数据
  useEffect(() => {
    Promise.all([
      admin.stats(),
      admin.syncLogs(),
    ])
      .then(([statsRes, logsRes]) => {
        setStats(statsRes.data);
        setLogs(logsRes.data || []);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  // 表格列配置
  const columns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 70,
      render: (id: number) => <Text type="secondary" style={{ fontSize: 12 }}>#{id}</Text>,
    },
    {
      title: '操作',
      dataIndex: 'action',
      key: 'action',
      width: 110,
      render: (action: string) => (
        <Tag
          style={{
            borderRadius: 'var(--radius-sm)',
            background: 'var(--color-primary-bg)',
            color: 'var(--color-primary)',
            border: 'none',
            fontWeight: 500,
          }}
        >
          {actionLabels[action] || action}
        </Tag>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 80,
      render: (status: string) =>
        status === 'success' ? (
          <Tag icon={<CheckCircleOutlined />} color="success" style={{ borderRadius: 'var(--radius-sm)' }}>
            成功
          </Tag>
        ) : (
          <Tag icon={<CloseCircleOutlined />} color="error" style={{ borderRadius: 'var(--radius-sm)' }}>
            失败
          </Tag>
        ),
    },
    {
      title: '详情',
      dataIndex: 'details',
      key: 'details',
      ellipsis: true,
      render: (details: string | null, record: SyncLog) =>
        details ? (
          <Text
            ellipsis={{ tooltip: { title: details, placement: 'topLeft' } }}
            style={{
              maxWidth: 300,
              display: 'inline-block',
              cursor: 'pointer',
              color: 'var(--color-primary)',
              fontWeight: 500,
            }}
            onClick={() => setDetailModal(record)}
          >
            {details.length > 40 ? details.substring(0, 40) + '...' : details}
          </Text>
        ) : (
          <Text type="secondary">-</Text>
        ),
    },
    {
      title: '时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 160,
      render: (t: string) => (
        <Text type="secondary" style={{ fontSize: 12 }}>
          {new Date(t).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })}
        </Text>
      ),
    },
  ];

  return (
    <div className="animate-fade-in">
      {/* 页面标题 */}
      <div style={{ marginBottom: 24 }}>
        <h2 style={{ fontSize: 22, fontWeight: 600, marginBottom: 4 }}>控制台概览</h2>
        <Text type="secondary">实时查看系统运行状态</Text>
      </div>

      {/* 统计卡片网格 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} sm={12}>
          <StatCard
            title="总用户数"
            value={stats.total_users}
            icon={<UserOutlined style={{ fontSize: 24 }} />}
            color="#0D9488"
          />
        </Col>
        <Col xs={24} sm={12}>
          <StatCard
            title="同步次数"
            value={stats.total_syncs}
            icon={<SwapOutlined style={{ fontSize: 24 }} />}
            color="#F97316"
          />
        </Col>
      </Row>

      {/* 同步记录列表 */}
      <Card
        title={
          <Space size={8}>
            <HistoryOutlined style={{ color: 'var(--color-primary)' }} />
            <span>同步记录</span>
          </Space>
        }
        style={{ borderRadius: 'var(--radius-lg)' }}
        bodyStyle={{ padding: 0 }}
        extra={
          <Text type="secondary" style={{ fontSize: 13 }}>
            共 {logs.length} 条记录
          </Text>
        }
      >
        {logs.length === 0 && !loading ? (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="暂无同步记录"
            style={{ padding: '40px 0' }}
          />
        ) : (
          <Table
            columns={columns}
            dataSource={logs}
            rowKey="id"
            loading={loading}
            size="middle"
            pagination={{
              pageSize: 10,
              showSizeChanger: false,
              showTotal: (total) => `共 ${total} 条`,
            }}
            style={{ borderRadius: 'var(--radius-lg)' }}
          />
        )}
      </Card>

      {/* 详情弹窗 */}
      <Modal
        title={
          <Space size={8}>
            <CloudOutlined style={{ color: 'var(--color-primary)' }} />
            <span>同步详情 #{detailModal?.id}</span>
          </Space>
        }
        open={!!detailModal}
        onCancel={() => setDetailModal(null)}
        footer={null}
        width={520}
        styles={{ body: { paddingTop: 16 } }}
      >
        {detailModal && (
          <Descriptions column={1} bordered size="small">
            <Descriptions.Item label="操作">
              {actionLabels[detailModal.action] || detailModal.action}
            </Descriptions.Item>
            <Descriptions.Item label="状态">
              {detailModal.status === 'success' ? (
                <Tag icon={<CheckCircleOutlined />} color="success">成功</Tag>
              ) : (
                <Tag icon={<CloseCircleOutlined />} color="error">失败</Tag>
              )}
            </Descriptions.Item>
            <Descriptions.Item label="用户 ID">{detailModal.user_id}</Descriptions.Item>
            <Descriptions.Item label="时间">
              {new Date(detailModal.created_at).toLocaleString('zh-CN')}
            </Descriptions.Item>
            <Descriptions.Item label="详情">
              <Paragraph
                copyable
                style={{
                  whiteSpace: 'pre-wrap',
                  margin: 0,
                  fontFamily: "'SF Mono', Monaco, monospace",
                  fontSize: 12,
                  background: 'var(--color-bg)',
                  padding: 12,
                  borderRadius: 'var(--radius-sm)',
                }}
              >
                {detailModal.details || '-'}
              </Paragraph>
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>
    </div>
  );
};

export default Dashboard;
