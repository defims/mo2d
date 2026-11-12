# mo2d 工程规范(任何会话/贡献者必读)

mo2d——AI 原生的 2D 矢量动画引擎(万物皆线)。**文档库在 [`docs/`](docs/index.md),
按 Open Knowledge Format v0.2 组织**;改 docs/ 下任何 .md 前必读
[docs/AGENTS.md](docs/AGENTS.md),改完必须跑 `python3 docs/check-okf.py`(0 errors 才算完成)。

## 许可证与署名

- 本项目 **MIT OR Apache-2.0 双许可**(见 `LICENSE-MIT` / `LICENSE-APACHE`),
  版权署名 `defims and the mo2d contributors`;
- 新增源文件应在文件头携带 SPDX 标识:`SPDX-License-Identifier: MIT OR Apache-2.0`;
- 文档的生成/修订署名走 OKF 的 `generated: { by, at }` 信任戳
  (代理 `zcode-agent/<model>`、人 `human:<id>`);git 提交署名为仓库身份
  (`Long, Wei <lostaim@gmail.com>`),会话产物在 docs/log.md 留痕。

## GPL 隔离条款(硬红线)

1. **禁止复制、翻译或衍生任何 GPL 许可代码**——特别是
   [BUI](https://github.com/ppmpreetham/BUI)(GPL-3.0,含 Blender GPL-2+ 衍生)
   与 Blender 源码:BUI 仅允许参照其公开文档描述的**设计思想**
   (损伤渲染/实例化 uber-shader 等,见 [docs/architecture.md](docs/architecture.md) §3.1);
2. 代码评审检查项:发现与 BUI/Blender 结构同构的非常规实现,必须溯源质询,
   无合理解释即重写;
3. 依赖引入规则:第三方 crate/package 必须为 MIT/Apache-2.0/BSD/MPL 等宽松许可;
   GPL/AGPL/LGPL 依赖一律禁止(引擎当前零第三方依赖,保持)。

## Moho 语料红线

1. `/Users/wei/moho user content/`(84 个官方教程工程)与一切 Moho 拥有版权的
   fixture:**永不进入本仓库**;CI/测试 fixture 一律自造;
2. mo2d 消费 maohou 知识库中的**格式事实**(字段名/默认值/通道码——事实不受版权),
   不引入任何 Moho 二进制分析产物;mo2d 自身不运行/不自动化 Moho;
3. 公开材料措辞:不使用"复刻/兼容 Moho",不暗示 Moho 背书
   ([docs/legal-risks.md](docs/legal-risks.md) R2/R3/R4)。

## 质量门

- `cd engine && cargo test` 必须 47+ 全绿;`python3 docs/check-okf.py` 必须 0 errors;
- 任何破坏字节级确定性的变更(哈希/时间戳/迭代序)都会被冻结门抓住
  ([demo/frozen.json](demo/frozen.json)),不要绕过它,修根因。
