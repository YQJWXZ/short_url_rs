# short_url_rs

A highly available short-link service project implemented using Rust + React.

## 项目结构

```
short_url_rs/                  # Rust 后端服务
│   ├── src/
│   	  │   ├── main.rs            # 主程序入口
│       │   ├── models/            # 数据模型
│       │   ├── services/          # 业务逻辑层
│       │   ├── api/               # API 路由处理
│       │   ├── db/                # 数据库操作
│       │   ├── pb/                # proto生成
│       │   └── utils/             # 工具函数
│       ├── abi.proto              # 基本数据结构定义
│       ├── build.rs               # proto配置生成
│       ├── Cargo.toml             # Rust 依赖配置
│       └── .env                   # 环境变量
└── frontend/                      # React前端服务
    ├── src/
    │   ├── App.tsx                # 主应用组件
    │   ├── pages/                 # 页面组件
    │   ├── api/                   # API 客户端
    │   ├── types/                 # 类型定义
    │   └── styles/                # 样式文件
    ├── public/                    # 静态资源
    ├── package.json               # Node.js 依赖
    ├── webpack.config.js          # Webpack 配置
    └── tsconfig.json              # TypeScript 配置
```

## 功能特性

### 后端 (Rust)
- **短链接生成**：支持自动生成和自定义短码
- **短链接跳转**：高性能重定向服务
- **短链接管理**：查询、删除用户的短链接
- **过期机制**：支持设置链接有效期
- **用户隔离**：基于用户ID的数据隔离
- **数据库**：使用 SQLite 存储数据
- **CORS 支持**：允许跨域请求

### 前端 (TypeScript + React)
- **响应式设计**：支持桌面和移动设备
- **短链接生成界面**：直观的表单输入
- **链接管理界面**：查看和删除已创建的短链接
- **实时状态显示**：显示链接创建时间和过期状态
- **一键复制**：方便的链接复制功能


## 技术栈

### 后端
- **Rust**: 高性能系统编程语言
- **Actix-Web**: 高性能 Web 框架
- **SQLx**: 异步 SQL 工具包
- **SQLite**: 轻量级数据库
- **Chrono**: 日期时间处理
- **Serde**: 序列化/反序列化

### 前端
- **TypeScript**: 类型安全的 JavaScript
- **React**: 用户界面库
- **Webpack**: 模块打包工具
- **CSS3**: 现代样式设计

## API 接口

### 创建短链接
```
POST /api/shorten
Content-Type: application/json

{
  "long_url": "https://example.com",
  "custom_code": "mycode",  // 可选
  "timeout": 3600,          // 可选，秒
  "user_id": "user_123"
}
```

### 获取用户链接
```
GET /api/urls/{user_id}
```

### 删除短链接
```
DELETE /api/urls/{id}/{user_id}
```

### 短链接重定向
```
GET /{short_code}
```

## 运行说明

### 后端启动
```bash
cargo run
```
服务将在 http://0.0.0.0:8080 启动

### 前端开发
```bash
cd frontend
npm start
```
开发服务器将在 http://localhost:3000 启动

### 前端构建
```bash
cd frontend
npm run build
```

## 数据库设计

### short_urls 表
| 字段       | 类型    | 说明           |
| ---------- | ------- | -------------- |
| id         | INTEGER | 主键，自增     |
| long_url   | TEXT    | 原始长链接     |
| short_code | TEXT    | 短码，唯一     |
| created_at | TEXT    | 创建时间       |
| expires_at | TEXT    | 过期时间，可选 |
| user_id    | TEXT    | 用户ID         |

## 扩展功能

- 访问统计和分析
- 批量短链接生成
- 二维码生成
- 链接预览
- 用户认证系统
- 管理后台界面
