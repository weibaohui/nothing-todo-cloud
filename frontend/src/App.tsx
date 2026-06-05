/**
 * ntd-cloud 主应用组件
 *
 * 布局设计：
 * - 桌面端：左侧固定侧边栏 + 顶部 Header
 * - 移动端：顶部 Header + 底部 Tab 导航
 * - 内容区域最大宽度 1400px，居中显示
 */
import React, { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { ConfigProvider, Layout, Menu } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import {
  DashboardOutlined,
  KeyOutlined,
  SettingOutlined,
  UnorderedListOutlined,
  CloudOutlined,
} from '@ant-design/icons';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Tokens from './pages/Tokens';
import Settings from './pages/Settings';
import Todos from './pages/Todos';

const { Header, Content, Sider } = Layout;

/** 导航菜单配置 */
const menuItems = [
  { key: '/dashboard', icon: <DashboardOutlined />, label: '控制台' },
  { key: '/tokens',    icon: <KeyOutlined />,      label: 'Token 管理' },
  { key: '/todos',     icon: <UnorderedListOutlined />, label: 'Todo 管理' },
  { key: '/settings',  icon: <SettingOutlined />,  label: '设置' },
];

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

/** 桌面端布局 - 左侧侧边栏 */
const DesktopLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <Layout style={{ minHeight: '100vh' }}>
      {/* 顶部导航栏 */}
      <Header style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        zIndex: 100,
        height: 'var(--header-height)',
        padding: '0 24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        boxShadow: 'var(--shadow-md)',
      }}>
        {/* Logo 区域 */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{
            width: 36,
            height: 36,
            borderRadius: 'var(--radius-md)',
            background: 'rgba(255,255,255,0.2)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}>
            <CloudOutlined style={{ fontSize: 20, color: '#fff' }} />
          </div>
          <span style={{ color: '#fff', fontSize: 18, fontWeight: 600, letterSpacing: '-0.5px' }}>
            ntd-cloud
          </span>
        </div>
        {/* 右侧标题 */}
        <span style={{ color: 'rgba(255,255,255,0.85)', fontSize: 14 }}>
          管理后台
        </span>
      </Header>

      {/* 侧边栏 + 内容区域 */}
      <Layout style={{ marginTop: 'var(--header-height)' }}>
        {/* 左侧导航 */}
        <Sider
          width={220}
          style={{
            position: 'fixed',
            left: 0,
            top: 'var(--header-height)',
            bottom: 0,
            background: '#fff',
            borderRight: '1px solid var(--color-border-light)',
            boxShadow: 'var(--shadow-sm)',
            zIndex: 50,
          }}
        >
          <Menu
            mode="inline"
            selectedKeys={[location.pathname]}
            style={{
              height: '100%',
              borderRight: 0,
              padding: '16px 8px',
            }}
            items={menuItems.map(item => ({
              ...item,
              style: {
                borderRadius: 'var(--radius-md)',
                marginBottom: 4,
                height: 44,
                display: 'flex',
                alignItems: 'center',
              },
            }))}
            onClick={({ key }) => navigate(key)}
          />
        </Sider>

        {/* 主内容区域 */}
        <Content style={{
          marginLeft: 220,
          padding: '24px 32px',
          minHeight: 'calc(100vh - var(--header-height))',
          maxWidth: 'calc(100vw - 220px)',
        }}>
          <div style={{ maxWidth: 'var(--content-max-width)', margin: '0 auto' }}>
            {children}
          </div>
        </Content>
      </Layout>
    </Layout>
  );
};

/** 移动端布局 - 底部 Tab 导航 */
const MobileLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <Layout style={{ minHeight: '100vh', background: 'var(--color-bg)' }}>
      {/* 顶部 Header */}
      <Header style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        zIndex: 100,
        height: 'var(--header-height)',
        padding: '0 16px',
        display: 'flex',
        alignItems: 'center',
        boxShadow: 'var(--shadow-md)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div style={{
            width: 32,
            height: 32,
            borderRadius: 'var(--radius-sm)',
            background: 'rgba(255,255,255,0.2)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}>
            <CloudOutlined style={{ fontSize: 16, color: '#fff' }} />
          </div>
          <span style={{ color: '#fff', fontSize: 16, fontWeight: 600 }}>
            ntd-cloud
          </span>
        </div>
      </Header>

      {/* 内容区域 - 预留顶部空间，底部给 TabBar */}
      <Content style={{
        marginTop: 'var(--header-height)',
        marginBottom: 56,
        padding: '16px',
        minHeight: 'calc(100vh - var(--header-height) - 56px)',
      }}>
        {children}
      </Content>

      {/* 底部 Tab 导航 - 使用 div 模拟 */}
      <div style={{
        position: 'fixed',
        bottom: 0,
        left: 0,
        right: 0,
        height: 56,
        background: '#fff',
        borderTop: '1px solid var(--color-border-light)',
        boxShadow: '0 -2px 8px rgba(0,0,0,0.06)',
        zIndex: 100,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-around',
      }}>
        {menuItems.map(item => {
          const isActive = location.pathname === item.key;
          return (
            <div
              key={item.key}
              onClick={() => navigate(item.key)}
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                gap: 2,
                cursor: 'pointer',
                color: isActive ? 'var(--color-primary)' : 'var(--color-text-muted)',
                transition: 'color var(--transition-fast)',
                padding: '4px 16px',
              }}
            >
              {item.icon}
              <span style={{ fontSize: 10, fontWeight: isActive ? 600 : 400 }}>
                {item.label}
              </span>
            </div>
          );
        })}
      </div>
    </Layout>
  );
};

/** 主应用入口 */
const App: React.FC = () => {
  // 初始化时检查 localStorage 中是否有 token，有则自动登录
  const [isLoggedIn, setIsLoggedIn] = useState(() => !!localStorage.getItem('ntd_cloud_token'));
  const isMobile = useIsMobile();

  // 根据设备类型选择布局
  const AppLayout = isMobile ? MobileLayout : DesktopLayout;

  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        token: {
          colorPrimary: '#0D9488',
          borderRadius: 8,
          fontFamily: "'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif",
        },
      }}
    >
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<Login onLogin={() => setIsLoggedIn(true)} />} />
          <Route
            path="/dashboard"
            element={isLoggedIn ? <AppLayout><Dashboard /></AppLayout> : <Navigate to="/login" />}
          />
          <Route
            path="/tokens"
            element={isLoggedIn ? <AppLayout><Tokens /></AppLayout> : <Navigate to="/login" />}
          />
          <Route
            path="/settings"
            element={isLoggedIn ? <AppLayout><Settings /></AppLayout> : <Navigate to="/login" />}
          />
          <Route
            path="/todos"
            element={isLoggedIn ? <AppLayout><Todos /></AppLayout> : <Navigate to="/login" />}
          />
          <Route path="/" element={<Navigate to="/dashboard" />} />
        </Routes>
      </BrowserRouter>
    </ConfigProvider>
  );
};

export default App;
