import React, { useEffect, useState } from 'react';
import { Card, Table, Button, Modal, Form, Input, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { devices } from '../api/client';

interface Device {
  id: number;
  device_name: string;
  last_seen_at: string;
  created_at: string;
}

const Devices: React.FC = () => {
  const [data, setData] = useState<Device[]>([]);
  const [modalVisible, setModalVisible] = useState(false);
  const [form] = Form.useForm();

  useEffect(() => {
    loadDevices();
  }, []);

  const loadDevices = async () => {
    try {
      const res = await devices.list();
      setData(res.data);
    } catch (err) {
      console.error(err);
    }
  };

  const handleAdd = async () => {
    try {
      await form.validateFields();
      await devices.register(form.getFieldValue('name'));
      message.success('设备注册成功');
      setModalVisible(false);
      form.resetFields();
      loadDevices();
    } catch (err) {
      console.error(err);
    }
  };

  const columns = [
    { title: 'ID', dataIndex: 'id', key: 'id' },
    { title: '设备名称', dataIndex: 'device_name', key: 'device_name' },
    { title: '最后访问', dataIndex: 'last_seen_at', key: 'last_seen_at' },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at' },
  ];

  return (
    <Card
      title="设备管理"
      extra={<Button type="primary" icon={<PlusOutlined />} onClick={() => setModalVisible(true)}>
        注册设备
      </Button>}
    >
      <Table columns={columns} dataSource={data} rowKey="id" />

      <Modal
        title="注册新设备"
        open={modalVisible}
        onOk={handleAdd}
        onCancel={() => setModalVisible(false)}
      >
        <Form form={form} layout="vertical">
          <Form.Item name="name" label="设备名称" rules={[{ required: true }]}>
            <Input placeholder="如：MacBook Pro" />
          </Form.Item>
        </Form>
      </Modal>
    </Card>
  );
};

export default Devices;
