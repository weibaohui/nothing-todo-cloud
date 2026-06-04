import React, { useState } from 'react';
import { Card, Descriptions, Button, message, Input, Typography } from 'antd';
import { CopyOutlined, EyeOutlined, EyeInvisibleOutlined } from '@ant-design/icons';

const { Text } = Typography;

const Settings: React.FC = () => {
  const [showToken, setShowToken] = useState(false);

  // 从 localStorage 读取当前登录的 JWT Token
  const token = localStorage.getItem('ntd_cloud_token') || '';

  const handleCopyToken = () => {
    if (token) {
      navigator.clipboard.writeText(token);
      message.success('Token 已复制到剪贴板');
    }
  };

  const handleExportData = () => {
    message.info('数据导出功能开发中...');
  };

  return (
    <Card title="系统设置">
      <Descriptions column={1} bordered>
        <Descriptions.Item label="服务器版本">v0.1.0</Descriptions.Item>
        <Descriptions.Item label="数据库">SQLite</Descriptions.Item>
        <Descriptions.Item label="认证方式">JWT Token</Descriptions.Item>
        <Descriptions.Item label="当前登录 Token">
          {token ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Input
                type={showToken ? 'text' : 'password'}
                value={token}
                readOnly
                style={{ width: 320, fontFamily: 'monospace', fontSize: 12 }}
              />
              <Button
                icon={showToken ? <EyeInvisibleOutlined /> : <EyeOutlined />}
                size="small"
                onClick={() => setShowToken(!showToken)}
              />
              <Button
                icon={<CopyOutlined />}
                size="small"
                onClick={handleCopyToken}
              >
                复制
              </Button>
            </div>
          ) : (
            <Text type="warning">未登录</Text>
          )}
        </Descriptions.Item>
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
