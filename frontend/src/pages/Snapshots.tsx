import React, { useEffect, useState } from 'react';
import { Card, Table, Tag, Typography, Modal, Descriptions, Select, Button, Popconfirm, message, Space } from 'antd';
import { DeleteOutlined, EditOutlined, UserOutlined } from '@ant-design/icons';
import { admin } from '../api/client';

const { Text, Paragraph } = Typography;

interface User {
  id: number;
  email: string;
  plan: string;
  created_at: string;
}

interface Snapshot {
  id: number;
  user_id: number;
  data_type: string;
  data_payload: string;
  created_at: string;
}

const Snapshots: React.FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedUser, setSelectedUser] = useState<number | null>(null);
  const [editModal, setEditModal] = useState<Snapshot | null>(null);
  const [editPayload, setEditPayload] = useState('');

  const loadData = () => {
    setLoading(true);
    Promise.all([
      admin.listUsers(),
      admin.snapshots(),
    ])
      .then(([usersRes, snapshotsRes]) => {
        setUsers(usersRes.data);
        setSnapshots(snapshotsRes.data);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    loadData();
  }, []);

  // 按用户筛选
  const filteredSnapshots = selectedUser
    ? snapshots.filter(s => s.user_id === selectedUser)
    : snapshots;

  // 获取用户邮箱
  const getUserEmail = (userId: number) => {
    const user = users.find(u => u.id === userId);
    return user?.email || `用户 #${userId}`;
  };

  // 计算摘要
  const getSummary = (payload: string) => {
    const lines = payload.split('\n');
    let todoCount = 0;
    let tagCount = 0;
    let skillCount = 0;
    let currentSection = '';

    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed === 'todos:') {
        currentSection = 'todos';
      } else if (trimmed === 'tags:') {
        currentSection = 'tags';
      } else if (trimmed === 'skills:') {
        currentSection = 'skills';
      } else if (trimmed.startsWith('- ') && currentSection === 'todos') {
        todoCount++;
      } else if (trimmed.startsWith('- ') && currentSection === 'tags') {
        tagCount++;
      } else if (trimmed.startsWith('- ') && currentSection === 'skills') {
        skillCount++;
      }
    }

    const parts = [];
    if (todoCount > 0) parts.push(`${todoCount} 条 Todo`);
    if (tagCount > 0) parts.push(`${tagCount} 条 Tag`);
    if (skillCount > 0) parts.push(`${skillCount} 条 Skill`);
    return parts.join(', ') || '-';
  };

  // 打开编辑弹窗
  const openEdit = (snapshot: Snapshot) => {
    setEditModal(snapshot);
    setEditPayload(snapshot.data_payload);
  };

  // 保存修改
  const handleSave = () => {
    if (!editModal) return;
    admin.updateSnapshot(editModal.id, editPayload)
      .then(() => {
        message.success('保存成功');
        setEditModal(null);
        loadData();
      })
      .catch((err) => {
        message.error('保存失败: ' + err.message);
      });
  };

  // 删除快照
  const handleDelete = (id: number) => {
    admin.deleteSnapshot(id)
      .then(() => {
        message.success('删除成功');
        loadData();
      })
      .catch((err) => {
        message.error('删除失败: ' + err.message);
      });
  };

  const columns = [
    {
      title: '用户',
      dataIndex: 'user_id',
      key: 'user_id',
      width: 200,
      render: (userId: number) => (
        <Text>
          <UserOutlined style={{ marginRight: 8 }} />
          {getUserEmail(userId)}
        </Text>
      ),
    },
    {
      title: '数据类型',
      dataIndex: 'data_type',
      key: 'data_type',
      width: 100,
      render: (type: string) => (
        <Tag color={type === 'todos' ? 'blue' : type === 'tags' ? 'green' : 'orange'}>
          {type}
        </Tag>
      ),
    },
    {
      title: '数据摘要',
      key: 'summary',
      render: (_: any, record: Snapshot) => (
        <Text type="secondary">{getSummary(record.data_payload)}</Text>
      ),
    },
    {
      title: '更新时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
      render: (t: string) => new Date(t).toLocaleString('zh-CN'),
    },
    {
      title: '操作',
      key: 'action',
      width: 150,
      render: (_: any, record: Snapshot) => (
        <Space>
          <Button
            type="link"
            size="small"
            icon={<EditOutlined />}
            onClick={() => openEdit(record)}
          >
            编辑
          </Button>
          <Popconfirm
            title="确定删除此快照?"
            onConfirm={() => handleDelete(record.id)}
            okText="删除"
            cancelText="取消"
            okType="danger"
          >
            <Button
              type="link"
              size="small"
              danger
              icon={<DeleteOutlined />}
            >
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div>
      {/* 用户筛选 */}
      <Card size="small" style={{ marginBottom: 16 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <Text>筛选用户:</Text>
          <Select
            style={{ width: 300 }}
            placeholder="全部用户"
            allowClear
            value={selectedUser}
            onChange={(value) => setSelectedUser(value || null)}
          >
            {users.map(user => (
              <Select.Option key={user.id} value={user.id}>
                {user.email} (#{user.id})
              </Select.Option>
            ))}
          </Select>
          <Text type="secondary">
            共 {filteredSnapshots.length} 条记录
          </Text>
        </div>
      </Card>

      {/* 快照列表 */}
      <Card title="用户数据快照">
        <Table
          columns={columns}
          dataSource={filteredSnapshots}
          rowKey="id"
          loading={loading}
          size="small"
          pagination={{ pageSize: 20, showSizeChanger: false }}
        />
      </Card>

      {/* 编辑弹窗 */}
      <Modal
        title="编辑数据"
        open={!!editModal}
        onCancel={() => setEditModal(null)}
        onOk={handleSave}
        okText="保存"
        cancelText="取消"
        width={700}
      >
        {editModal && (
          <>
            <Descriptions column={2} bordered size="small" style={{ marginBottom: 16 }}>
              <Descriptions.Item label="ID">{editModal.id}</Descriptions.Item>
              <Descriptions.Item label="用户">{getUserEmail(editModal.user_id)}</Descriptions.Item>
              <Descriptions.Item label="数据类型">
                <Tag>{editModal.data_type}</Tag>
              </Descriptions.Item>
            </Descriptions>
            <Text strong>数据内容 (YAML 格式):</Text>
            <Paragraph>
              <textarea
                value={editPayload}
                onChange={(e) => setEditPayload(e.target.value)}
                style={{
                  width: '100%',
                  minHeight: 300,
                  fontFamily: 'monospace',
                  marginTop: 8,
                }}
              />
            </Paragraph>
          </>
        )}
      </Modal>
    </div>
  );
};

export default Snapshots;
