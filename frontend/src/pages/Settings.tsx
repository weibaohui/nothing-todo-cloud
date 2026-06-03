import React from 'react';
import { Card, Descriptions, Button, message } from 'antd';

const Settings: React.FC = () => {
  const handleExportData = () => {
    message.info('数据导出功能开发中...');
  };

  return (
    <Card title="系统设置">
      <Descriptions column={1} bordered>
        <Descriptions.Item label="服务器版本">v0.1.0</Descriptions.Item>
        <Descriptions.Item label="数据库">SQLite</Descriptions.Item>
        <Descriptions.Item label="认证方式">JWT Token</Descriptions.Item>
      </Descriptions>

      <Card title="数据管理" style={{ marginTop: 24 }}>
        <Button onClick={handleExportData}>导出所有数据</Button>
      </Card>

      <Card title="关于" style={{ marginTop: 24 }}>
        <p>ntd-cloud 是 nothing-todo 的云端同步服务器。</p>
        <p>支持多设备间的 Todos、Tags 和 Skills 同步。</p>
      </Card>
    </Card>
  );
};

export default Settings;
