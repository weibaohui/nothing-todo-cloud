//! 多设备同步集成测试

use std::process::Command;
use std::thread;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:8089";

fn curl_json(method: &str, path: &str, body: Option<&str>, token: Option<&str>) -> String {
    let mut cmd = Command::new("curl");
    cmd.arg("-s")
        .arg("-X")
        .arg(method);

    if let Some(t) = token {
        cmd.arg("-H").arg(format!("Authorization: Bearer {}", t));
    }

    if let Some(b) = body {
        cmd.arg("-H").arg("Content-Type: application/json");
        cmd.arg("-d").arg(b);
    }

    cmd.arg(format!("{}{}", BASE_URL, path));

    cmd.output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default()
}

fn curl_yaml(method: &str, path: &str, body: &str, token: &str) -> String {
    let mut cmd = Command::new("curl");
    cmd.arg("-s")
        .arg("-X")
        .arg(method)
        .arg("-H")
        .arg(format!("Authorization: Bearer {}", token))
        .arg("-H")
        .arg("Content-Type: text/yaml")
        .arg("-d")
        .arg(body);

    cmd.arg(format!("{}{}", BASE_URL, path));

    cmd.output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default()
}

fn register_user(email: &str) -> String {
    curl_json("POST", "/api/auth/register", Some(&serde_json::json!({
        "email": email,
        "password": "123456"
    }).to_string()), None)
}

fn login(email: &str) -> String {
    curl_json("POST", "/api/auth/login", Some(&serde_json::json!({
        "email": email,
        "password": "123456"
    }).to_string()), None)
}

fn create_device(token: &str, name: &str) -> String {
    curl_json("POST", "/api/devices", Some(&serde_json::json!({
        "device_name": name
    }).to_string()), Some(token))
}

fn push_yaml(token: &str, device_id: i64, data_type: &str, data: &str) -> String {
    let body = format!(
        "device_id: {}\ndata_type: {}\ndata: |\n  {}",
        device_id, data_type, data
    );
    curl_yaml("POST", "/api/v1/sync/push", &body, token)
}

fn pull_yaml(token: &str, device_id: i64, data_type: &str) -> String {
    curl_yaml("GET", &format!("/api/v1/sync/pull?device_id={}&data_type={}", device_id, data_type), "", token)
}

fn main() {
    println!("=== 多设备同步测试 ===\n");

    // 1. 启动服务器（假设已经在运行）
    println!("1. 检查服务器状态...");
    let health = curl_json("GET", "/health", None, None);
    if health.is_empty() {
        println!("❌ 服务器未启动，请先运行: cd backend && cargo run");
        return;
    }
    println!("✓ 服务器运行中\n");

    // 2. 注册用户
    println!("2. 注册用户 test@example.com");
    let reg = register_user("test@example.com");
    let token = reg
        .split("\"token\":\"")
        .nth(1)
        .map(|s| s.split('"').next().unwrap_or(""))
        .unwrap_or("");
    println!("✓ 获取到 Token: {}...\n", &token[..token.len().min(30)]);

    // 3. 创建设备 A 和 B
    println!("3. 创建设备 A");
    let device_a = create_device(token, "设备A");
    let device_a_id = device_a.split("\"id\":").nth(1).map(|s| {
        s.split(',').next().unwrap_or("1").parse::<i64>().unwrap_or(1)
    }).unwrap_or(1);
    println!("   设备A ID: {}", device_a_id);

    println!("4. 创建设备 B");
    let device_b = create_device(token, "设备B");
    let device_b_id = device_b.split("\"id\":").nth(1).map(|s| {
        s.split(',').next().unwrap_or("2").parse::<i64>().unwrap_or(2)
    }).unwrap_or(2);
    println!("   设备B ID: {}", device_b_id);

    // 5. 设备 A push todos
    println!("\n5. 设备A push todos");
    let a_todos = "todos:\n  - title: 买菜\n  - title: 做饭\n  - title: 洗澡";
    let a_push = push_yaml(token, device_a_id, "todos", a_todos);
    println!("   A push 结果:\n{}", a_push.lines().take(5).collect::<Vec<_>>().join("\n"));

    // 6. 设备 B push todos（部分重叠）
    println!("\n6. 设备B push todos（与A有重叠）");
    let b_todos = "todos:\n  - title: 买菜\n  - title: 洗衣服\n  - title: 跑步";
    let b_push = push_yaml(token, device_b_id, "todos", b_todos);
    println!("   B push 结果:\n{}", b_push.lines().take(5).collect::<Vec<_>>().join("\n"));

    // 7. 设备 A pull（应该收到合并后的数据）
    println!("\n7. 设备A pull（验证合并结果）");
    let a_pull = pull_yaml(token, device_a_id, "todos");
    let merged_count = a_pull.lines().filter(|l| l.contains("title:")).count();
    println!("   A pull 收到 {} 条记录", merged_count);
    if a_pull.contains("跑步") && a_pull.contains("洗衣服") {
        println!("   ✓ 合并成功！包含了B独有的数据");
    }

    // 8. 设备 B pull
    println!("\n8. 设备B pull");
    let b_pull = pull_yaml(token, device_b_id, "todos");
    println!("   B pull 结果:\n{}", b_pull.lines().take(5).collect::<Vec<_>>().join("\n"));

    // 9. 测试 tags 同步
    println!("\n9. 设备A push tags");
    let a_tags = "tags:\n  - 工作\n  - 生活";
    let a_tags_push = push_yaml(token, device_a_id, "tags", a_tags);
    println!("   A push tags:\n{}", a_tags_push.lines().take(5).collect::<Vec<_>>().join("\n"));

    println!("\n10. 设备B push tags（部分重叠）");
    let b_tags = "tags:\n  - 工作\n  - 学习";
    let b_tags_push = push_yaml(token, device_b_id, "tags", b_tags);
    let merged_tags_count = b_tags_push.lines().filter(|l| l.contains("- ")).count();
    println!("   B push tags 结果: {} 条标签", merged_tags_count);
    if b_tags_push.contains("生活") && b_tags_push.contains("学习") {
        println!("   ✓ tags 合并成功！包含A和B的并集");
    }

    println!("\n=== 测试完成 ===");
}
