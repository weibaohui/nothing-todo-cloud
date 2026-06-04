import React, { useEffect, useState } from 'react';
import { Card, Row, Col, Statistic, Table, Tag, Typography, Modal, Descriptions } from 'antd';
import { UserOutlined, SwapOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
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

const actionLabels: Record<string, string> = {
  push: '上传 (Push)',
  pull: '下载 (Pull)',
  merge: '合并',
  sync: '同步',
};

const Dashboard: React.FC = () => {
  const [stats, setStats] = useState({ total_users: 0, total_syncs: 0 });
  const [logs, setLogs] = useState<SyncLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [detailModal, setDetailModal] = useState<SyncLog | null>(null);

  useEffect(() => {
    Promise.all([
      admin.stats(),
      admin.syncLogs(),
    ])
      .then(([statsRes, logsRes]) => {
        setStats(statsRes.data);
        setLogs(logsRes.data);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const columns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 60,
    },
    {
      title: '操作',
      dataIndex: 'action',
      key: 'action',
      width: 120,
      render: (action: string) => actionLabels[action] || action,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 80,
      render: (status: string) =>
        status === 'success' ? (
          <Tag icon={<CheckCircleOutlined />} color="success">成功</Tag>
        ) : (
          <Tag icon={<CloseCircleOutlined />} color="error">失败</Tag>
        ),
    },
    {
      title: '详情',
      dataIndex: 'details',
      key: 'details',
      render: (details: string | null, record: SyncLog) => (
        details ? (
          <Text
            ellipsis={{ tooltip: { title: details, placement: 'topLeft' } }}
            style={{ maxWidth: 400, display: 'inline-block', cursor: 'pointer', color: '#1677ff' }}
            onClick={() => setDetailModal(record)}
          >
            {details}
          </Text>
        ) : (
          <Text type="secondary">-</Text>
        )
      ),
    },
    {
      title: '时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
      render: (t: string) => new Date(t).toLocaleString('zh-CN'),
    },
  ];

  return (
    <div>
      {/* 统计卡片 */}
      <Row gutter={16}>
        <Col span={12}>
          <Card>
            <Statistic
              title="总用户数"
              value={stats.total_users}
              prefix={<UserOutlined />}
            />
          </Card>
        </Col>
        <Col span={12}>
          <Card>
            <Statistic
              title="同步次数"
              value={stats.total_syncs}
              prefix={<SwapOutlined />}
            />
          </Card>
        </Col>
      </Row>

      {/* 同步记录列表 */}
      <Card
        title="同步记录"
        style={{ marginTop: 24 }}
      >
        <Table
          columns={columns}
          dataSource={logs}
          rowKey="id"
          loading={loading}
          size="small"
          pagination={{ pageSize: 15, showSizeChanger: false }}
        />
      </Card>

      {/* 详情弹窗 */}
      <Modal
        title={`同步记录 #${detailModal?.id || ''}`}
        open={!!detailModal}
        onCancel={() => setDetailModal(null)}
        footer={null}
        width={600}
      >
        {detailModal && (
          <Descriptions column={1} bordered size="small">
            <Descriptions.Item label="ID">{detailModal.id}</Descriptions.Item>
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
              <Paragraph copyable style={{ whiteSpace: 'pre-wrap', margin: 0 }}>
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
