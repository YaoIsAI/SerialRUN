# SerialRUN 开发日报 — 2026年5月31日

## 一、项目成果

### GitHub 仓库
- 公开仓库：https://github.com/YaoIsAI/SerialRUN
- Release v0.1.0 发布（Windows x64 + macOS arm64）
- 排除 GUI 代码（serialrun-gui），保持开源部分干净
- 清理敏感信息（字体版权、个人收款码）

### 官方网站
- 域名：https://www.serialrun.com
- 部署平台：Cloudflare Pages
- 项目名：serialrun
- 生产分支：master

### 文档更新
- README.md / README_CN.md 全面更新
- docs/MCP_API.md 补全 15 个工具文档
- docs/DEPLOY_WEB.md 网站部署指南
- wechat_article.html 微信公众号文章

---

## 二、网站功能清单

### 页面
| 页面 | 说明 |
|------|------|
| index.html | 首页（功能介绍、下载、社区、咖啡） |
| guide.html | 使用指南（8 个章节，中英文） |
| license.html | BSL 1.1 许可证说明（中英文） |

### 核心功能
- ✅ 中英文切换（i18n.js）
- ✅ 移动端响应式适配（汉堡菜单、自适应布局）
- ✅ SEO 优化（meta 标签、Open Graph、JSON-LD、sitemap.xml）
- ✅ 访问统计（counter.dev）
- ✅ GitHub 实时徽章（Stars/Forks/Downloads）
- ✅ 下载按钮（Windows + macOS zip，Linux 编译说明）
- ✅ macOS 安全提示（首次运行授权说明）
- ✅ 请作者喝咖啡（微信收款码）
- ✅ 社区板块（GitHub Issues + Discussions）
- ✅ 滚动动画（script.js）

### 设计规范
- 深色主题（#0a0a0f 背景）
- 绿色主色调（#22c55e）
- 字体：Inter + JetBrains Mono
- 响应式断点：768px + 480px

---

## 三、部署记录

### 首次部署
```bash
npx wrangler pages deploy website --project-name=serialrun
```
- 需要 Cloudflare OAuth 登录
- 选择生产分支：master
- 首次部署到 Preview 环境

### 修复生产部署
- 问题：所有部署显示 Preview，不是 Production
- 原因：生产分支设置为 `production`，实际用的是 `master`
- 修复：Pages 设置 → 生产分支 → 改为 `master`
- 重新部署后生效

### 自定义域名
- 注册 serialrun.com（Cloudflare 注册）
- 添加 CNAME：`@` → `serialrun.pages.dev`
- SSL 自动配置（等待 1-5 分钟）
- 添加 www 子域名

### 后续部署命令
```bash
npx wrangler pages deploy website --project-name=serialrun --branch=master --commit-dirty=true
```

---

## 四、修复的问题

| 问题 | 修复方案 |
|------|----------|
| 移动端导航菜单不显示 | 添加汉堡菜单按钮 + CSS toggle |
| AI 区域文字溢出 | 添加 word-wrap、overflow-wrap |
| macOS 提示文字看不清 | 改为亮黄色 + 半透明背景 |
| macOS 提示条太长 | 改为 inline-block，紧贴文字宽度 |
| 顶部提示条太长 | 精简文案 |
| BSL 1.1 链接 404 | 改为 mariadb.com/bsl11/ |
| 工具数量不一致（14 vs 15） | 统一为 15 |
| 下载按钮文字错误 | .exe → .zip |
| Linux 图标不对 | 替换为标准 Tux SVG |
| GitHub GUI 未删除 | git rm -r --cached + .gitignore |
| 字体版权问题 | 删除 fonts/msyh.ttc |
| 个人收款码泄露 | 从仓库删除（网站保留） |

---

## 五、文件变更统计

### 新增文件
| 文件 | 说明 |
|------|------|
| website/index.html | 首页 |
| website/guide.html | 使用指南 |
| website/license.html | 许可证页面 |
| website/style.css | 样式表 |
| website/i18n.js | 中英文翻译 |
| website/script.js | 滚动动画 |
| website/tux.svg | Linux 图标 |
| website/sitemap.xml | 站点地图 |
| website/DEPLOY_WEB.md | 部署指南 |
| website/wechat_article.html | 公众号文章 |
| docs/MCP_API.md | MCP API 参考 |
| docs/DEPLOY_WEB.md | 部署指南（repo 副本） |
| assets/screenshot_website_en.png | 网站截图（英文） |
| assets/screenshot_website_zh.png | 网站截图（中文） |

### 修改文件
| 文件 | 变更 |
|------|------|
| README.md | 添加 macOS 下载、安全提示、网站截图、咖啡支持 |
| README_CN.md | 同上中文版 |
| .gitignore | 排除 .claude/、test_*.py、website/、serialrun-gui/、.wrangler/ |
| Cargo.toml | 从 workspace 移除 serialrun-gui |
| LICENSE | MIT → BSL 1.1 |
| docs/help_en.md | MCP 工具数 14→15 |
| docs/help_zh.md | 同上 |
| docs/MANUAL.md | 修复 GitHub URL |
| docs/MANUAL_CN.md | 同上 |
| docs/QUALITY_REPORT.md | MCP 验证 12→15 |

### 删除文件
| 文件 | 原因 |
|------|------|
| fonts/msyh.ttc | 版权字体，不可公开分发 |
| assets/wechat_pay_qr.jpg | 从仓库删除（网站保留） |
| crates/serialrun-gui/ | 从 Git 跟踪移除（代码保留本地） |
| test_*.py | Python 测试文件，排除 |
| website/ | 从 Git 排除（独立部署） |

---

## 六、Git 提交记录

| Commit | 说明 |
|--------|------|
| dfde9d6 | feat: multi-viewport windows, MCP enhancements, UI/UX overhaul |
| 22ae8ba | docs: update all documentation |
| 38c24c5 | docs: add screenshots and download section |
| b447b6c | docs: update download links to GitHub releases |
| a0fc65b | revert: remove deployment guide from repo |
| 20b4c84 | license: replace MIT with BSL 1.1 |
| 0145ad1 | chore: exclude GUI from public repos |
| 0d29b46 | security: remove copyrighted font and personal QR code |
| e05be14 | docs: add "Buy me a coffee" section |
| 7ddcfa5 | docs: add website screenshots and serialrun.com link |
| c5a8eed | fix: resolve merge conflict and fix URLs |
| 7b733e5 | docs: add macOS download link and security note |

---

## 七、推广物料

### 微信公众号文章
- 文件：wechat_article.html
- 标题：「3 天，30 亿 Token，我用 AI 开发了「可能最 AI 的串口助手」」
- 内容：项目介绍、功能列表、MCP 详解、技术栈、下载、咖啡
- 样式：全部内联样式，适配微信编辑器

### Reddit 帖子
- 目标社区：r/rust、r/embedded、r/opensource、r/programming
- 状态：r/rust 被 AutoModerator 删除（AI 内容检测）
- 已发送 modmail 给版主申请恢复
- r/embedded 也被删除（新账号 karma 不足）

### Hacker News
- 账号已注册：YaoIsAI
- 状态：等待激活（新账号需几小时）
- 格式：`Show HN: SerialRUN – cross-platform serial debugger in Rust`

---

## 八、待办事项

| 优先级 | 事项 |
|--------|------|
| 高 | Reddit r/rust 等版主回复后重新发帖 |
| 高 | Hacker News 账号激活后发 Show HN |
| 中 | 试发 V2EX（国内开发者社区） |
| 中 | 养 Reddit karma（评论互动） |
| 低 | 考虑买域名（已用 Cloudflare 注册 serialrun.com） |
| 低 | counter.dev 数据观察和分析 |

---

## 九、关键数据

| 指标 | 数值 |
|------|------|
| GitHub Stars | 2 |
| GitHub Forks | 1 |
| GitHub Downloads | 7 |
| 网站部署次数 | 15+ |
| 修复问题数 | 12 |
| 新增文件 | 14 |
| 文档更新 | 10 |

---

*报告生成时间：2026-05-31*
*工作时长：约 10 小时*
*Token 消耗：约 30 亿*
