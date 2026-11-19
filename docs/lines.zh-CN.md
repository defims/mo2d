---
type: Design Document
title: "线到底：空间、时间、生成、流"
description: 四句本体论把整个引擎统一在一个名词和两个动词上——万物皆线(空间)、动即行走(时间线)、生即切片(lift/view)、流即行迹(帧流/事件流)；fold 与 slice 是仅有的两个普适算子，确定性即可走性。描述性综合，不是协议改动；为将来的生成关系切片铺概念地基(编号未定——C8 已被手势时间线占用)。
tags: [lines, ontology, timeline, fold, slice, streaming, events, generation]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-17T14:30:41Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线(空间线:线、节点、三级属性、九大映射)"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数(§1.2 事件也是命令,§4 事件维度)"
  - id: architecture-doc
    resource: /architecture.md
    title: "本 bundle:架构草稿(§3.2 流式渲染:惰性帧流=把 tick→frame 纯函数变现)"
  - id: vernacular-doc
    resource: /vernacular.md
    title: "本 bundle:生活概念层(§6 验收门 N1–N5,N5=概念预算 ≤25 词根)"
  - id: roadmap-doc
    resource: /roadmap.md
    title: "本 bundle:路线图(C7 交互、C8 手势时间线;生成关系切片编号未定,本文件铺其地基)"
  - id: mvp-doc
    resource: /mvp.md
    title: "本 bundle:MVP v5(D7 Tick 时间:时间真值是原始 tick 整数)"
  - id: log-doc
    resource: /log.md
    title: "本 bundle:变更日志(2026-11-17 hull 探针:转面→视觉外壳→三轴派生时间线)"
---

[English](lines.md) | **简体中文**

# 线到底：空间、时间、生成、流

本文件是**描述性综合**(事后统一,不是新机制)：用一个名词和两个动词陈述引擎
本来是什么,并逐条落到已交付的代码与门上。它**不提议任何协议改动**;唯一的
前瞻承诺是为候选的生成关系切片(C8)铺概念地基[^roadmap-doc]。

## 1. Owner 规范表述

原话照录(本节中文为原始规范表述;英文版对应位置附英译):

> 一切皆线

(项目座右铭,线模型的根基[^line-model-doc]。)

> 一条时间线可以是另外时间线的生成线

(2026-11-17,评审视觉外壳探针时提出;奠定"生成"句。)

> 流也是线

(2026-11-17,补齐四句本体论;奠定"流"句。)

由本会话综合出四句口号(agent 措辞,非 owner 原话)：**万物皆线,动即行走,
生即切片,流即行迹**。

## 2. 四句话,四个层级的线

| 层级 | 线 | 已落地的对应 |
|---|---|---|
| 空间 | 画的线:笔画、轮廓、骨架 | L0 模型——Line/Node,九大映射[^line-model-doc] |
| 时间 | 时间线:`t ↦ lines`,形状空间中的扫掠 | 参数+轨道采样;时间真值是原始 tick 整数(D7)[^mvp-doc] |
| 生成 | 生成线:时间线→时间线的算子 | hull 探针的 `lift`/`view`(转面→视觉外壳→三条派生时间线)[^log-doc] |
| 流 | 行走的线:帧流=时间线的采样行走;事件流=按 (tick, seq) 排序的事实线 | §3.2 流式渲染惰性帧流[^architecture-doc];C7 折叠事件前缀[^relations-doc];C8 P1 已实例化本句——事件流双写成只读 step 轨道线(`gesture/<target>`),事实线变成可求值的线[^roadmap-doc] |

线与流的区别只是离散化与遍历:时间线是连续扫掠,帧流是它按 tick 的行走,
事件流是不可逆事实的离散线。**确定性即可走性**:行走是纯函数,同一位置必得
同一帧——重放=从原点重走,seek=沿线重定位,frozen 哈希=行走途中钉下的不动点
(A1 门,P1-7 政策[^architecture-doc])。

## 3. 两个普适算子

上述每个机制都分解为对线的两种运算:

**fold**——沿一条线积分;升维(线→状态/工件/包络):

| fold 实例 | 已落地 |
|---|---|
| 事件前缀 fold → 参数状态 | C7:求值是纯函数 `f(doc, tick, 事件前缀)`[^relations-doc] |
| 整条时间线 fold → hull 工件 | hull 探针:`lift` 把 12 帧剪影雕成切片堆栈[^log-doc] |
| sim 从 tick 0 重放 | B6:固定子步长重走 |

**slice**——横切一族;降维(包络→线):

| slice 实例 | 已落地 |
|---|---|
| 时间线在 tick 处 → 帧 | `evaluate(doc, tick)`——任意帧独立重求值 |
| 实体在姿态处 → 派生时间线 | hull 探针:三条姿态轨迹下的 `view`[^log-doc] |
| 三视图 | 特例切片(同一次转面的 n=3 帧) |

探针与事件系统是同一个形状的两次出现:**slice ∘ fold**。经济性由复合给出
——fold 一次(贵,烘成工件,如同 sim 检查点或 frozen 基准),slice 无数次
(便宜,按需惰性)。这与 §3.2 的惰性帧流、损伤渲染的缓存层是同一个缓存结构
[^architecture-doc]:派生时间线就是实体的懒视图,从不整体物化。

## 4. 概念预算

这条本体论的成本是**一个名词、两个动词**:线、fold、slice。其余——动画、
流、事件、seek、重放、生成——都是被行走、被切片、被折叠的线,不是新原语。
它直接服务小白门预算(N5:≤25 词根[^vernacular-doc]);小白措辞归 vernacular.md,
此处只示意:动画=一条线走到头;拖进度条=在线上跳;录制=画下那条事实线;
导出=把行走写成字。

## 5. 它奠定什么,不奠定什么

奠定:候选的**生成关系切片**(编号未定——C8 已被同日确立的手势时间线占用)
——时间线→时间线的关系,`lift`(时间线→烘焙工件,确定性、带出处)与
`view`(工件×姿态轨迹→参数),与既有门文化逐字节咬合(探针即存在性证明
[^log-doc])。

**不**做:现在就把 fold/slice 暴露为用户原语;不动命令协议、格式、校验器;
也不主张任意两条时间线都可生成——生成要求共享底层模型(同一实体、两条轨迹),
这正是探针的包含门与雕刻单调门所断言的。已知硬边界从探针带过来:剪影可见性
(hull ⊇ 真体)、部分转角欠约束、非刚性作画欺骗,均留待专项切片文档决议。

[^line-model-doc]: 空间线模型:线、节点、三级属性、九大映射。
[^mvp-doc]: 冻结决议 D7——时间真值是原始 tick 整数;fps 仅显示用。
[^relations-doc]: C7 事件维度——事件是不可逆事实;前缀按 (tick, seq) 折叠;空前缀退化为既有行为。
[^architecture-doc]: §3.2 流式渲染——惰性帧流;A1/P1-7 哈希冻结+目检政策;缓存即缓存层。
[^vernacular-doc]: N1–N5 小白门;N5 概念预算 ≤25 词根。
[^log-doc]: log.md 2026-11-17——hull 探针:转面→视觉外壳 lift→三条派生时间线,冻结为 `hull`,gate_hull 六断言。
[^roadmap-doc]: C7 交互已交付;C8(交互手势时间线)owner 同日确立并以 C8 P1 落地;生成关系切片尚无编号——本文件只铺其地基。
