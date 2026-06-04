import React, { useEffect, useState } from 'react';
import { Card, Table, Tag, Typography, Modal, Descriptions, Select, Button, Popconfirm, message, Space, List } from 'antd';
import { DeleteOutlined, EditOutlined, UserOutlined, ExpandAltOutlined } from '@ant-design/icons';
import { admin } from '../api/client';

const { Text } = Typography;

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

interface ParsedSnapshot {
  snapshot: Snapshot;
  version?: string;
  todos: ParsedTodo[];
  tags: string[];
  skills: string[];
}

interface ParsedTodo {
  title: string;
  prompt?: string;
  status?: string;
  executor?: string;
  scheduler_enabled?: boolean;
  scheduler_config?: string;
  tag_names?: string[];
  workspace?: string;
  worktree?: string;
}

const Snapshots: React.FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedUser, setSelectedUser] = useState<number | null>(null);
  const [expandedKeys, setExpandedKeys] = useState<string[]>([]);
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

  // 解析 YAML payload
  const parsePayload = (payload: string): ParsedSnapshot => {
    const result: ParsedSnapshot = {
      snapshot: { id: 0, user_id: 0, data_type: '', data_payload: payload, created_at: '' },
      todos: [],
      tags: [],
      skills: [],
    };

    const lines = payload.split('\n');
    let currentSection = '';
    let currentTodo: ParsedTodo | null = null;

    for (const line of lines) {
      const trimmed = line.trim();

      // 空行跳过
      if (!trimmed) {
        if (currentTodo) {
          result.todos.push(currentTodo);
          currentTodo = null;
        }
        continue;
      }

      // 缩进级别判断
      const indent = line.length - line.trimStart().length;

      if (trimmed === 'todos:' || trimmed === 'tags:' || trimmed === 'skills:') {
        // 保存上一个 todo
        if (currentTodo) {
          result.todos.push(currentTodo);
          currentTodo = null;
        }
        if (trimmed === 'todos:') currentSection = 'todos';
        else if (trimmed === 'tags:') currentSection = 'tags';
        else if (trimmed === 'skills:') currentSection = 'skills';
      } else if (trimmed.startsWith('version:')) {
        result.version = trimmed.substring(8).trim().replace(/'/g, '');
      } else if (trimmed.startsWith('- title:')) {
        // 保存上一个 todo
        if (currentTodo) {
          result.todos.push(currentTodo);
        }
        currentTodo = {
          title: trimmed.substring(9).trim().replace(/'/g, ''),
        };
        currentSection = 'todos';
      } else if (currentTodo && indent > 1) {
        // Todo 的子字段
        if (trimmed.startsWith('prompt:')) {
          currentTodo.prompt = trimmed.substring(7).trim().replace(/'/g, '');
        } else if (trimmed.startsWith('status:')) {
          currentTodo.status = trimmed.substring(7).trim().replace(/'/g, '');
        } else if (trimmed.startsWith('executor:')) {
          currentTodo.executor = trimmed.substring(9).trim().replace(/'/g, '');
        } else if (trimmed.startsWith('scheduler_enabled:')) {
          currentTodo.scheduler_enabled = trimmed.substring(18).trim() === 'true';
        } else if (trimmed.startsWith('scheduler_config:')) {
          currentTodo.scheduler_config = trimmed.substring(17).trim().replace(/'/g, '');
        } else if (trimmed.startsWith('tag_names:')) {
          currentSection = 'tag_names';
        } else if (trimmed.startsWith('workspace:')) {
          currentTodo.workspace = trimmed.substring(10).trim().replace(/'/g, '');
        } else if (trimmed.startsWith('worktree:')) {
          currentTodo.worktree = trimmed.substring(10).trim().replace(/'/g, '');
        } else if (currentSection === 'tag_names' && trimmed.startsWith('- ')) {
          if (!currentTodo.tag_names) currentTodo.tag_names = [];
          currentTodo.tag_names.push(trimmed.substring(2).trim().replace(/'/g, ''));
        }
      } else if (currentSection === 'tags' && trimmed.startsWith('- ')) {
        result.tags.push(trimmed.substring(2).trim().replace(/'/g, ''));
      } else if (currentSection === 'skills' && trimmed.startsWith('- ')) {
        result.skills.push(trimmed.substring(2).trim().replace(/'/g, ''));
      }
    }

    // 保存最后一个 todo
    if (currentTodo) {
      result.todos.push(currentTodo);
    }

    return result;
  };

  // 按用户筛选
  const filteredSnapshots = selectedUser
    ? snapshots.filter(s => s.user_id === selectedUser)
    : snapshots;

  // 获取用户邮箱
  const getUserEmail = (userId: number) => {
    const user = users.find(u => u.id === userId);
    return user?.email || `用户 #${userId}`;
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

  // 切换展开
  const toggleExpand = (id: number) => {
    setExpandedKeys(prev =>
      prev.includes(String(id))
        ? prev.filter(k => k !== String(id))
        : [...prev, String(id)]
    );
  };

  const columns = [
    {
      title: '用户',
      dataIndex: 'user_id',
      key: 'user_id',
      width: 180,
      render: (userId: number) => (
        <Text>
          <UserOutlined style={{ marginRight: 8 }} />
          {getUserEmail(userId)}
        </Text>
      ),
    },
    {
      title: '类型',
      dataIndex: 'data_type',
      key: 'data_type',
      width: 80,
      render: (type: string) => (
        <Tag color={type === 'todos' ? 'blue' : type === 'tags' ? 'green' : 'orange'}>
          {type}
        </Tag>
      ),
    },
    {
      title: '统计',
      key: 'stats',
      render: (_: any, record: Snapshot) => {
        const parsed = parsePayload(record.data_payload);
        return (
          <Space direction="vertical" size="small">
            {parsed.todos.length > 0 && <Text>{parsed.todos.length} 条 Todo</Text>}
            {parsed.tags.length > 0 && <Text type="secondary">{parsed.tags.length} 条 Tag</Text>}
            {parsed.skills.length > 0 && <Text type="secondary">{parsed.skills.length} 条 Skill</Text>}
          </Space>
        );
      },
    },
    {
      title: '更新时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 160,
      render: (t: string) => new Date(t).toLocaleString('zh-CN'),
    },
    {
      title: '操作',
      key: 'action',
      width: 120,
      render: (_: any, record: Snapshot) => (
        <Space>
          <Button
            type="link"
            size="small"
            icon={<ExpandAltOutlined />}
            onClick={() => toggleExpand(record.id)}
          >
            {expandedKeys.includes(String(record.id)) ? '收起' : '展开'}
          </Button>
          <Popconfirm
            title="确定删除?"
            onConfirm={() => handleDelete(record.id)}
            okText="删除"
            okType="danger"
          >
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>
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
        <Space>
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
          <Text type="secondary">共 {filteredSnapshots.length} 条记录</Text>
        </Space>
      </Card>

      {/* 快照列表 */}
      <Card title="用户数据">
        <Table
          columns={columns}
          dataSource={filteredSnapshots}
          rowKey="id"
          loading={loading}
          size="small"
          pagination={{ pageSize: 20, showSizeChanger: false }}
          expandable={{
            expandedRowKeys: expandedKeys.map(Number),
            expandedRowRender: (record: Snapshot) => {
              const parsed = parsePayload(record.data_payload);
              return (
                <Card size="small" type="inner" title="详细数据">
                  {/* Tags */}
                  {parsed.tags.length > 0 && (
                    <div style={{ marginBottom: 16 }}>
                      <Text strong>Tags: </Text>
                      {parsed.tags.map(tag => (
                        <Tag key={tag} color="green">{tag}</Tag>
                      ))}
                    </div>
                  )}

                  {/* Skills */}
                  {parsed.skills.length > 0 && (
                    <div style={{ marginBottom: 16 }}>
                      <Text strong>Skills: </Text>
                      {parsed.skills.map(skill => (
                        <Tag key={skill} color="orange">{skill}</Tag>
                      ))}
                    </div>
                  )}

                  {/* Todos */}
                  {parsed.todos.length > 0 && (
                    <List
                      size="small"
                      bordered
                      dataSource={parsed.todos}
                      renderItem={(todo: ParsedTodo) => (
                        <List.Item>
                          <Descriptions column={3} size="small" style={{ width: '100%' }}>
                            <Descriptions.Item label="标题" span={2}>
                              <Text strong>{todo.title}</Text>
                            </Descriptions.Item>
                            <Descriptions.Item label="状态">
                              <Tag color={todo.status === 'completed' ? 'success' : 'default'}>
                                {todo.status || 'pending'}
                              </Tag>
                            </Descriptions.Item>
                            {todo.prompt && (
                              <Descriptions.Item label="Prompt">
                                {todo.prompt}
                              </Descriptions.Item>
                            )}
                            {todo.executor && (
                              <Descriptions.Item label="执行器">
                                {todo.executor}
                              </Descriptions.Item>
                            )}
                            {todo.scheduler_enabled && (
                              <Descriptions.Item label="定时">
                                {todo.scheduler_config || '已启用'}
                              </Descriptions.Item>
                            )}
                            {todo.tag_names && todo.tag_names.length > 0 && (
                              <Descriptions.Item label="标签">
                                {todo.tag_names.map(t => (
                                  <Tag key={t}>{t}</Tag>
                                ))}
                              </Descriptions.Item>
                            )}
                            {todo.workspace && (
                              <Descriptions.Item label="工作空间">
                                {todo.workspace}
                              </Descriptions.Item>
                            )}
                            {todo.worktree && (
                              <Descriptions.Item label="Worktree">
                                {todo.worktree}
                              </Descriptions.Item>
                            )}
                          </Descriptions>
                        </List.Item>
                      )}
                    />
                  )}

                  <div style={{ marginTop: 16 }}>
                    <Button
                      type="link"
                      icon={<EditOutlined />}
                      onClick={() => openEdit(record)}
                    >
                      编辑完整 YAML
                    </Button>
                  </div>
                </Card>
              );
            },
          }}
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
              <Descriptions.Item label="更新时间">
                {new Date(editModal.created_at).toLocaleString('zh-CN')}
              </Descriptions.Item>
            </Descriptions>
            <Text strong>YAML 内容:</Text>
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
          </>
        )}
      </Modal>
    </div>
  );
};

export default Snapshots;
