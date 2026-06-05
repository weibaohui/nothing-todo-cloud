/**
 * ntd-cloud 登录页面
 *
 * 设计特点：
 * - 渐变背景 + 居中卡片布局
 * - 玻璃态效果
 * - 平滑过渡动画
 */
import React, { useState } from 'react';
import { Form, Input, Button, Card, message, Typography } from 'antd';
import { UserOutlined, LockOutlined, CloudOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { auth } from '../api/client';

const { Title, Text } = Typography;

interface Props {
  onLogin: () => void;
}

/** 登录表单数据 */
interface LoginForm {
  email: string;
  password: string;
}

const Login: React.FC<Props> = ({ onLogin }) => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  // 处理登录提交
  const handleLogin = async (values: LoginForm) => {
    setLoading(true);
    try {
      const res = await auth.login(values.email, values.password);
      if (res.data.token) {
        localStorage.setItem('ntd_cloud_token', res.data.token);
        message.success('登录成功');
        onLogin();
        navigate('/dashboard');
      }
    } catch (err: any) {
      message.error(err.response?.data?.error || '登录失败，请检查邮箱和密码');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      minHeight: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      // 渐变背景
      background: 'linear-gradient(135deg, #0D9488 0%, #0F766E 50%, #134E4A 100%)',
      padding: '20px',
    }}>
      {/* 装饰性背景元素 */}
      <div style={{
        position: 'fixed',
        top: '10%',
        left: '5%',
        width: 300,
        height: 300,
        borderRadius: '50%',
        background: 'rgba(255,255,255,0.05)',
        filter: 'blur(60px)',
        pointerEvents: 'none',
      }} />
      <div style={{
        position: 'fixed',
        bottom: '15%',
        right: '8%',
        width: 250,
        height: 250,
        borderRadius: '50%',
        background: 'rgba(249,115,22,0.15)',
        filter: 'blur(50px)',
        pointerEvents: 'none',
      }} />

      {/* 登录卡片 */}
      <Card
        style={{
          width: '100%',
          maxWidth: 420,
          borderRadius: 'var(--radius-xl)',
          boxShadow: '0 20px 40px rgba(0,0,0,0.2)',
          border: 'none',
          overflow: 'hidden',
        }}
        bodyStyle={{ padding: '40px 36px' }}
      >
        {/* Logo 和标题 */}
        <div style={{ textAlign: 'center', marginBottom: 32 }}>
          {/* Logo 图标 */}
          <div style={{
            width: 64,
            height: 64,
            borderRadius: 'var(--radius-lg)',
            background: 'linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-dark) 100%)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            margin: '0 auto 16px',
            boxShadow: '0 8px 20px rgba(13,148,136,0.3)',
          }}>
            <CloudOutlined style={{ fontSize: 28, color: '#fff' }} />
          </div>
          <Title level={3} style={{ marginBottom: 4, color: 'var(--color-text)' }}>
            欢迎回来
          </Title>
          <Text type="secondary">
            登录 ntd-cloud 管理后台
          </Text>
        </div>

        {/* 登录表单 */}
        <Form<LoginForm>
          layout="vertical"
          onFinish={handleLogin}
          size="large"
          requiredMark={false}
        >
          {/* 邮箱输入 */}
          <Form.Item
            name="email"
            rules={[
              { required: true, message: '请输入邮箱' },
              { type: 'email', message: '请输入有效的邮箱地址' },
            ]}
          >
            <Input
              prefix={<UserOutlined style={{ color: 'var(--color-text-muted)' }} />}
              placeholder="邮箱地址"
              autoComplete="email"
              style={{ height: 48 }}
            />
          </Form.Item>

          {/* 密码输入 */}
          <Form.Item
            name="password"
            rules={[{ required: true, message: '请输入密码' }]}
          >
            <Input.Password
              prefix={<LockOutlined style={{ color: 'var(--color-text-muted)' }} />}
              placeholder="密码"
              autoComplete="current-password"
              style={{ height: 48 }}
            />
          </Form.Item>

          {/* 提交按钮 */}
          <Form.Item style={{ marginBottom: 0, marginTop: 8 }}>
            <Button
              type="primary"
              htmlType="submit"
              loading={loading}
              block
              size="large"
              style={{
                height: 48,
                fontSize: 16,
                fontWeight: 600,
                background: 'linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-dark) 100%)',
                border: 'none',
                boxShadow: '0 4px 12px rgba(13,148,136,0.3)',
              }}
            >
              {loading ? '登录中...' : '登录'}
            </Button>
          </Form.Item>
        </Form>
      </Card>

      {/* 底部版权信息 */}
      <div style={{
        position: 'fixed',
        bottom: 20,
        textAlign: 'center',
        color: 'rgba(255,255,255,0.6)',
        fontSize: 12,
      }}>
        ntd-cloud · 云端数据同步服务
      </div>
    </div>
  );
};

export default Login;
