import React, { useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom';
import { ConfigProvider, Layout, Menu } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import {
  DashboardOutlined,
  KeyOutlined,
  SettingOutlined,
  FileTextOutlined,
} from '@ant-design/icons';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Tokens from './pages/Tokens';
import Settings from './pages/Settings';
import Snapshots from './pages/Snapshots';

const { Header, Content, Sider } = Layout;

/** 侧边栏导航菜单项 */
const menuItems = [
  { key: '/dashboard', icon: <DashboardOutlined />, label: '控制台' },
  { key: '/tokens',    icon: <KeyOutlined />,      label: 'Token 管理' },
  { key: '/snapshots', icon: <FileTextOutlined />,   label: 'Todo 管理' },
  { key: '/settings',  icon: <SettingOutlined />,  label: '设置' },
];

/** 登录后才显示侧边栏的布局 */
const AppLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#001529', padding: '0 24px', display: 'flex', alignItems: 'center' }}>
        <div style={{ color: '#fff', fontSize: 18, fontWeight: 'bold' }}>
          ntd-cloud 管理后台
        </div>
      </Header>
      <Layout>
        <Sider width={200} style={{ background: '#fff' }}>
          <Menu
            mode="inline"
            selectedKeys={[location.pathname]}
            style={{ height: '100%', borderRight: 0 }}
            items={menuItems}
            onClick={({ key }) => navigate(key)}
          />
        </Sider>
        <Content style={{ padding: 24, minHeight: 280 }}>
          {children}
        </Content>
      </Layout>
    </Layout>
  );
};

const App: React.FC = () => {
  // 初始化时检查 localStorage 中是否有 token，有则自动登录
  const [isLoggedIn, setIsLoggedIn] = useState(() => {
    return !!localStorage.getItem('ntd_cloud_token');
  });

  return (
    <ConfigProvider locale={zhCN}>
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
            path="/snapshots"
            element={isLoggedIn ? <AppLayout><Snapshots /></AppLayout> : <Navigate to="/login" />}
          />
          <Route path="/" element={<Navigate to="/dashboard" />} />
        </Routes>
      </BrowserRouter>
    </ConfigProvider>
  );
};

export default App;
