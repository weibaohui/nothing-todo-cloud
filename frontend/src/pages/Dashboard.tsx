import React, { useEffect, useState } from 'react';
import { Card, Row, Col, Statistic, Table, Tag, Typography } from 'antd';
import { UserOutlined, SwapOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import { admin } from '../api/client';

const { Text } = Typography;

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
      width: 140,
      render: (action: string) => actionLabels[action] || action,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
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
      render: (details: string | null) => (
        <Text ellipsis style={{ maxWidth: 300, display: 'inline-block' }}>
          {details || '-'}
        </Text>
      ),
    },
    {
      title: '用户 ID',
      dataIndex: 'user_id',
      key: 'user_id',
      width: 80,
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
    </div>
  );
};

export default Dashboard;
