# mpush

轻量级微信模板消息推送命令行工具。

## 功能

- 通过微信公众号 API 发送模板消息
- 支持逗号分隔的多用户批量推送
- 日志自动脱敏（openid/模板ID 仅显示首尾4位）
- 跨平台支持
- 极小体积，约 1.6MB

## 使用

```bash
# 单用户
mpush -u <openid> -t <模板ID> -d <消息内容>

# 多用户（逗号分隔）
mpush -u <openid1>,<openid2>,<openid3> -t <模板ID> -d <消息内容>

# 带跳转链接
mpush -u <openid> -t <模板ID> -l <链接> -d <消息内容>
```

### 参数

| 参数             | 说明                            |
| ---------------- | ------------------------------- |
| `-u, --user`     | 接收者 openid（多个用逗号分隔） |
| `-t, --template` | 模板 ID                         |
| `-l, --link`     | 模板跳转链接（可选）            |
| `-d, --data`     | 消息内容                        |
| `-h, --help`     | 显示帮助信息                    |

### 环境变量

| 变量               | 说明                 |
| ------------------ | -------------------- |
| `MPUSH_APP_ID`     | 微信公众号 AppID     |
| `MPUSH_APP_SECRET` | 微信公众号 AppSecret |

### 多用户行为

- **尽力发送**：即使部分用户失败，也会继续发送给剩余用户
- **退出码**：全部成功返回 0，任意失败返回 1
- **输出**：单用户成功时无输出；多用户时逐行输出 `[OK]`/`[ERR]` 到 stderr

## 编译

```bash
# 本地编译
cargo build --release

# 交叉编译 Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu
```

产物路径：`target/release/mpush` 或 `target/x86_64-unknown-linux-gnu/release/mpush`

## 日志

日志写入 `/var/log/mpush/mpush-YYYY-MM-DD.log`，自动清理 15 天前的日志。需确保目录存在且有写入权限：

```bash
sudo mkdir -p /var/log/mpush
sudo chown $(whoami) /var/log/mpush
```

## 示例：定时监控

```bash
#!/bin/bash
export MPUSH_APP_ID="your_app_id"
export MPUSH_APP_SECRET="your_app_secret"

OPENID="your_openid"
TEMPLATE_ID="your_template_id"

# 检测后端
http_code=$(curl -s -o /dev/null -m 10 -w "%{http_code}" "https://example.com/api/health")
if [ "$http_code" != "200" ]; then
  /usr/local/bin/mpush -u "$OPENID" -t "$TEMPLATE_ID" -d "告警: 后端 HTTP $http_code"
fi
```

添加 crontab 每 2 小时执行一次：

```bash
0 */2 * * * /path/to/monitor.sh
```

## License

MIT
