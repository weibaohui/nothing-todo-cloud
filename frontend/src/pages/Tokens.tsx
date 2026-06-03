import React, { useEffect, useState } from 'react';
import { Card, Table, Button, Modal, Form, Input, message, Popconfirm } from 'antd';
import { PlusOutlined, CopyOutlined, DeleteOutlined } from '@ant-design/icons';
import { tokens } from '../api/client';

interface Token {
  id: number;
  name: string;
  token?: string;
  last_used_at: string | null;
  created_at: string;
}

const Tokens: React.FC = () => {
  const [data, setData] = useState<Token[]>([]);
  const [modalVisible, setModalVisible] = useState(false);
  const [newToken, setNewToken] = useState<string | null>(null);
  const [form] = Form.useForm();

  useEffect(() => {
    loadTokens();
  }, []);

  const loadTokens = async () => {
    try {
      const res = await tokens.list();
      setData(res.data);
    } catch (err) {
      console.error(err);
    }
  };

  const handleCreate = async () => {
    try {
      await form.validateFields();
      const res = await tokens.create(form.getFieldValue('name'));
      setNewToken(res.data.token);
      message.success('Token 创建成功，请及时复制保存');
      form.resetFields();
      loadTokens();
    } catch (err) {
      console.error(err);
    }
  };

  const handleRevoke = async (id: number) => {
    try {
      await tokens.revoke(id);
      message.success('Token 已撤销');
      loadTokens();
    } catch (err) {
      console.error(err);
    }
  };

  const copyToken = (token: string) => {
    navigator.clipboard.writeText(token);
    message.success('已复制到剪贴板');
  };

  const columns = [
    { title: 'ID', dataIndex: 'id', key: 'id' },
    { title: '名称', dataIndex: 'name', key: 'name' },
    { title: '最后使用', dataIndex: 'last_used_at', key: 'last_used_at', render: (v: string | null) => v || '从未使用' },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at' },
    {
      title: '操作',
      key: 'action',
      render: (_: any, record: Token) => (
        <>
          {record.token && (
            <Button icon={<CopyOutlined />} onClick={() => copyToken(record.token!)} size="small" style={{ marginRight: 8 }}>
              复制
            </Button>
          )}
          <Popconfirm title="确定撤销此 Token？" onConfirm={() => handleRevoke(record.id)}>
            <Button icon={<DeleteOutlined />} danger size="small">撤销</Button>
          </Popconfirm>
        </>
      ),
    },
  ];

  return (
    <Card
      title="API Token 管理"
      extra={<Button type="primary" icon={<PlusOutlined />} onClick={() => { setModalVisible(true); setNewToken(null); }}>
        创建 Token
      </Button>}
    >
      <Table columns={columns} dataSource={data} rowKey="id" />

      <Modal
        title="创建新 Token"
        open={modalVisible}
        onOk={handleCreate}
        onCancel={() => { setModalVisible(false); setNewToken(null); }}
      >
        {newToken ? (
          <div>
            <p style={{ color: '#52c41a', marginBottom: 16 }}>Token 创建成功，请立即复制保存！</p>
            <Input value={newToken} readOnly addonAfter={<Button onClick={() => copyToken(newToken)} size="small">复制</Button>} />
          </div>
        ) : (
          <Form form={form} layout="vertical">
            <Form.Item name="name" label="Token 名称" rules={[{ required: true }]}>
              <Input placeholder="如：Home Server" />
            </Form.Item>
          </Form>
        )}
      </Modal>
    </Card>
  );
};

export default Tokens;
