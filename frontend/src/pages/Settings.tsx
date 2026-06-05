/**
 * ntd-cloud 系统设置页面
 *
 * 设计特点：
 * - 分区卡片布局
 * - 设置项分组显示
 * - 敏感信息脱敏显示
 */
import React, { useState } from 'react';
import {
  Card, Descriptions, Button, message, Input, Typography, Space, Divider, Tag
} from 'antd';
import {
  CopyOutlined,
  EyeOutlined,
  EyeInvisibleOutlined,
  SettingOutlined,
  InfoCircleOutlined,
  DatabaseOutlined,
  LockOutlined,
  CloudOutlined,
} from '@ant-design/icons';

const { Text, Paragraph } = Typography;

const Settings: React.FC = () => {
  const [showToken, setShowToken] = useState(false);

  // 从 localStorage 读取当前登录的 JWT Token
  const token = localStorage.getItem('ntd_cloud_token') || '';

  // 复制 Token 到剪贴板
  const handleCopyToken = () => {
    if (token) {
      navigator.clipboard.writeText(token);
      message.success('Token 已复制到剪贴板');
    }
  };

  // 复制用户 ID
  const handleCopyUserId = () => {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      if (payload.user_id) {
        navigator.clipboard.writeText(String(payload.user_id));
        message.success('User ID 已复制');
      }
    } catch {
      message.error('解析 Token 失败');
    }
  };

  // 导出数据
  const handleExportData = () => {
    message.info('数据导出功能开发中...');
  };

  return (
    <div className="animate-fade-in">
      {/* 页面标题 */}
      <div style={{ marginBottom: 24 }}>
        <h2 style={{ fontSize: 22, fontWeight: 600, marginBottom: 4 }}>系统设置</h2>
        <Text type="secondary">查看系统配置和当前登录信息</Text>
      </div>

      {/* 系统信息 */}
      <Card
        title={
          <Space size={8}>
            <InfoCircleOutlined style={{ color: 'var(--color-primary)' }} />
            <span>系统信息</span>
          </Space>
        }
        style={{ marginBottom: 16, borderRadius: 'var(--radius-lg)' }}
      >
        <Descriptions column={{ xs: 1, sm: 2 }} bordered size="small">
          <Descriptions.Item label="服务器版本">
            <Tag color="blue">v0.1.0</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="数据库">
            <Space>
              <DatabaseOutlined style={{ color: 'var(--color-primary)' }} />
              SQLite
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="认证方式">
            <Space>
              <LockOutlined style={{ color: 'var(--color-primary)' }} />
              JWT Token
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="部署方式">
            <Space>
              <CloudOutlined style={{ color: 'var(--color-primary)' }} />
              Docker
            </Space>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {/* 当前登录信息 */}
      <Card
        title={
          <Space size={8}>
            <SettingOutlined style={{ color: 'var(--color-primary)' }} />
            <span>当前会话</span>
          </Space>
        }
        style={{ marginBottom: 16, borderRadius: 'var(--radius-lg)' }}
      >
        <Descriptions column={1} bordered size="small">
          <Descriptions.Item label="登录状态">
            {token ? (
              <Tag color="success" icon={<LockOutlined />}>已登录</Tag>
            ) : (
              <Tag color="default">未登录</Tag>
            )}
          </Descriptions.Item>
          {token && (
            <>
              <Descriptions.Item label="User ID">
                <Space>
                  {(() => {
                    try {
                      const payload = JSON.parse(atob(token.split('.')[1]));
                      return payload.user_id || '-';
                    } catch {
                      return '-';
                    }
                  })()}
                  <Button
                    type="text"
                    size="small"
                    icon={<CopyOutlined />}
                    onClick={handleCopyUserId}
                  />
                </Space>
              </Descriptions.Item>
              <Descriptions.Item label="JWT Token">
                <Space align="center">
                  <Input
                    type={showToken ? 'text' : 'password'}
                    value={token}
                    readOnly
                    style={{
                      width: 280,
                      fontFamily: "'SF Mono', Monaco, monospace",
                      fontSize: 11,
                    }}
                  />
                  <Button
                    icon={showToken ? <EyeInvisibleOutlined /> : <EyeOutlined />}
                    size="small"
                    onClick={() => setShowToken(!showToken)}
                    style={{ borderRadius: 'var(--radius-sm)' }}
                  />
                  <Button
                    icon={<CopyOutlined />}
                    size="small"
                    onClick={handleCopyToken}
                    style={{ borderRadius: 'var(--radius-sm)' }}
                  >
                    复制
                  </Button>
                </Space>
              </Descriptions.Item>
            </>
          )}
        </Descriptions>
      </Card>

      {/* 数据管理 */}
      <Card
        title={
          <Space size={8}>
            <DatabaseOutlined style={{ color: 'var(--color-primary)' }} />
            <span>数据管理</span>
          </Space>
        }
        style={{ marginBottom: 16, borderRadius: 'var(--radius-lg)' }}
      >
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <Button
            icon={<CopyOutlined />}
            onClick={handleExportData}
            style={{ borderRadius: 'var(--radius-md)' }}
          >
            导出所有数据
          </Button>
          <Text type="secondary" style={{ fontSize: 12 }}>
            导出格式为 JSON，包含所有 Todo、标签和技能数据
          </Text>
        </Space>
      </Card>

      {/* 关于 */}
      <Card
        title={
          <Space size={8}>
            <CloudOutlined style={{ color: 'var(--color-primary)' }} />
            <span>关于</span>
          </Space>
        }
        style={{ borderRadius: 'var(--radius-lg)' }}
      >
        <Space direction="vertical" size="middle">
          <div>
            <Text strong style={{ fontSize: 16 }}>ntd-cloud</Text>
            <br />
            <Text type="secondary">nothing-todo 云端同步服务器</Text>
          </div>
          <Divider style={{ margin: '8px 0' }} />
          <Paragraph type="secondary" style={{ margin: 0 }}>
            ntd-cloud 是 nothing-todo 的云端同步服务器，支持多设备间的 Todos、Tags 和 Skills 同步。
            使用 Rust (Axum) + React 构建，数据库采用 SQLite。
          </Paragraph>
        </Space>
      </Card>
    </div>
  );
};

export default Settings;
