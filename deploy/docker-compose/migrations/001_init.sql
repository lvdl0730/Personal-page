-- 你现在主要做健康检查 + 验证码，所以这里先放最小内容
-- 确认初始化脚本确实会被执行
CREATE TABLE IF NOT EXISTS __init_check (
  id BIGINT PRIMARY KEY AUTO_INCREMENT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO __init_check () VALUES ();