import React, { useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider, Layout } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Devices from './pages/Devices';
import Tokens from './pages/Tokens';
import Settings from './pages/Settings';

const { Header, Content } = Layout;

const App: React.FC = () => {
  const [isLoggedIn, setIsLoggedIn] = useState(false);

  return (
    <ConfigProvider locale={zhCN}>
      <BrowserRouter>
        <Layout style={{ minHeight: '100vh' }}>
          <Header style={{ background: '#001529', padding: '0 24px', display: 'flex', alignItems: 'center' }}>
            <div style={{ color: '#fff', fontSize: 18, fontWeight: 'bold' }}>
              ntd-cloud 管理后台
            </div>
          </Header>
          <Content style={{ padding: '24px' }}>
            <Routes>
              <Route path="/login" element={<Login onLogin={() => setIsLoggedIn(true)} />} />
              <Route path="/dashboard" element={isLoggedIn ? <Dashboard /> : <Navigate to="/login" />} />
              <Route path="/devices" element={isLoggedIn ? <Devices /> : <Navigate to="/login" />} />
              <Route path="/tokens" element={isLoggedIn ? <Tokens /> : <Navigate to="/login" />} />
              <Route path="/settings" element={isLoggedIn ? <Settings /> : <Navigate to="/login" />} />
              <Route path="/" element={<Navigate to="/dashboard" />} />
            </Routes>
          </Content>
        </Layout>
      </BrowserRouter>
    </ConfigProvider>
  );
};

export default App;
