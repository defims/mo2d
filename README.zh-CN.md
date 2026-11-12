[English](README.md) | **简体中文**

# mo2d

**AI 原生的 2D 矢量动画引擎——万物皆线。**

骨骼是 2 个节点的线段,骨架是一条折线;时间轴是多条有多个节点的直线,每个节点是一个关键帧;
渐变由控制棒(线段)决定;层和群组都是线的集合(预定义关系);模拟是线与线的关系。
有了 AI,不再需要标准工具栏与菜单——用户直接在舞台上定义线及线间的联动关系。

## 核心理念

1. **万物皆线**——一切结构与控制归约为线;
2. **关系即函数**——一切行为逻辑是线与线之间的关系;
3. **舞台即界面**——无标准控件,直接操纵 + 自然语言双通道;
4. **概念契合 GPU**——概念统一 ⇒ 数据同构 ⇒ GPU 吞吐最大化。

## 文档

本项目文档按 [Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/open-knowledge-format)
组织,位于 [`docs/`](docs/index.zh-CN.md)——维护规程见 [docs/AGENTS.md](docs/AGENTS.zh-CN.md),改完必须跑
`python3 docs/check-okf.py`(0 errors 才算完成)。

建议阅读顺序:[愿景](docs/vision.zh-CN.md) → **[万物皆线](docs/line-model.zh-CN.md)** →
[关系即函数](docs/relations.zh-CN.md) → [AI 原生交互](docs/ai-interaction.zh-CN.md) →
[架构草案](docs/architecture.zh-CN.md) → [MVP:会动的触手](docs/mvp.zh-CN.md) →
[路线图](docs/roadmap.zh-CN.md) → [术语表](docs/glossary.zh-CN.md)

## 引擎实现(MVP)

[`engine/`](engine/README.md) —— Rust 无头引擎(零第三方依赖),MVP 六件全落地:
线内核(3D 就绪)/关系热路径(track·share·bind)/Tick 时间/命令协议 v1 + 校验器 + 确定性门/
`.mo2d` 格式 + PNG 序列导出/A6 最小舞台(四操作,零控件)。

```bash
cd engine && cargo test                 # 47 tests: 单测 + 门 A1–A6 + 吞吐
cargo run --release -- stage --doc ../demo/out/tentacle.mo2d   # 打开最小舞台
```

演示物在 [`demo/`](demo/):触手命令流 + z≠0 3D 场景命令流、48 帧导出 + GIF、
冻结哈希([demo/frozen.json](demo/frozen.json))。

## 许可证

MIT OR Apache-2.0 双许可,任选其一——见 [LICENSE-MIT](LICENSE-MIT) / [LICENSE-APACHE](LICENSE-APACHE)。
**两者均强制署名**:分发本软件(全部或实质部分,含修改)必须保留版权与许可声明,
Apache-2.0 §4.1(d) 另要求**原样随附 [NOTICE](NOTICE)**。版权所有 © 2026 defims 及 mo2d 贡献者。
贡献即同意按此双许可发布(工程规范见 [AGENTS.md](AGENTS.zh-CN.md))。
