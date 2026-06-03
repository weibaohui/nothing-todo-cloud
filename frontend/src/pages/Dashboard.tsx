import React, { useEffect, useState } from 'react';
import { Card, Row, Col, Statistic, Table } from 'antd';
import { UserOutlined, LaptopOutlined, SwapOutlined } from '@ant-design/icons';
import { admin } from '../api/client';

const Dashboard: React.FC = () => {
  const [stats, setStats] = useState({ total_users: 0, total_devices: 0, total_syncs: 0 });

  useEffect(() => {
    admin.stats().then((res) => setStats(res.data)).catch(console.error);
  }, []);

  return (
    <div>
      <Row gutter={16}>
        <Col span={8}>
          <Card>
            <Statistic
              title="总用户数"
              value={stats.total_users}
              prefix={<UserOutlined />}
            />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic
              title="注册设备数"
              value={stats.total_devices}
              prefix={<LaptopOutlined />}
            />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic
              title="同步次数"
              value={stats.total_syncs}
              prefix={<SwapOutlined />}
            />
          </Card>
        </Col>
      </Row>

      <Card title="系统概览" style={{ marginTop: 24 }}>
        <p>欢迎使用 ntd-cloud 管理后台。</p>
        <p>在这里您可以管理用户账户、API Token 和设备。</p>
      </Card>
    </div>
  );
};

export default Dashboard;
