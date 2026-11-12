#!/usr/bin/env python3
"""OKF v0.2 符合性校验器(docs/ bundle)。

规则(OKF SPEC v0.2 + docs/AGENTS.md 本地规程):
1. 除 index.md/log.md 外,每个 .md 必须有可解析的 frontmatter(首行 --- 起始,配对 --- 结束)。
2. frontmatter 必须含非空 type。
3. index.md 不得携带 frontmatter(bundle 根 index.md 例外:仅允许 okf_version)。
4. log.md 首行应为含 "Update Log" 的一级标题。
5. generated(若存在)必须含 by 与 at(at 为 ISO 8601 带 UTC 偏移)。
6. sources(若存在)每条必须含 resource。

用法:python3 docs/check-okf.py   (在仓库任意位置运行;退出码 0=通过)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent


def parse_frontmatter(text: str) -> tuple[dict[str, str], str] | None:
    """极简解析:返回 (键→原始值文本, 正文)。无法解析返回 None。"""
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    block = text[4:end]
    body = text[end + 5 :]
    fields: dict[str, str] = {}
    for line in block.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        m = re.match(r"^([A-Za-z_][\w-]*)\s*:\s*(.*)$", line)
        if m:
            fields[m.group(1)] = m.group(2).strip()
    return fields, body


def top_value(raw: str) -> str:
    """取标量值(去引号;列表取首个元素)。"""
    v = raw.strip()
    if v.startswith("[") and v.endswith("]"):
        inner = v[1:-1].strip()
        return inner.split(",")[0].strip().strip("'\"") if inner else ""
    return v.strip("'\"")


def main() -> int:
    errors: list[str] = []
    concepts = 0
    for path in sorted(BUNDLE.rglob("*.md")):
        rel = path.relative_to(BUNDLE)
        text = path.read_text(encoding="utf-8")
        reserved = path.name in ("index.md", "log.md")
        if reserved:
            if path.name == "index.md":
                is_root = rel == Path("index.md")
                if text.startswith("---"):
                    parsed = parse_frontmatter(text)
                    keys = set(parsed[0]) if parsed else set()
                    allowed = {"okf_version"} if is_root else set()
                    extra = keys - allowed
                    if extra:
                        errors.append(f"{rel}: index.md frontmatter 只允许 {allowed or '∅'},出现 {sorted(extra)}")
                    if is_root and "okf_version" not in keys:
                        errors.append(f"{rel}: bundle 根 index.md 应声明 okf_version")
            else:  # log.md
                first = text.splitlines()[0] if text.splitlines() else ""
                if "Update Log" not in first:
                    errors.append(f"{rel}: log.md 首行应为目录更新日志标题")
            continue
        concepts += 1
        parsed = parse_frontmatter(text)
        if parsed is None:
            errors.append(f"{rel}: 缺少可解析的 YAML frontmatter(首行 --- 起始、配对 --- 结束)")
            continue
        fields, _body = parsed
        t = top_value(fields.get("type", ""))
        if not t:
            errors.append(f"{rel}: frontmatter 缺少非空 type")
        g = fields.get("generated", "")
        if g:
            if not re.search(r"\bby\s*:", g) or not re.search(r"\bat\s*:", g):
                errors.append(f"{rel}: generated 必须同时含 by 与 at")
            at = re.search(r"at\s*:\s*['\"]?([\dT:Z+\-]+)", g)
            if at and not re.match(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(Z|[+-]\d{2}:\d{2})$", at.group(1)):
                errors.append(f"{rel}: generated.at 不是带 UTC 偏移的 ISO 8601:{at.group(1)}")
        s = fields.get("sources", "")
        if s and not s.startswith("["):
            # 多行 sources 块:粗查每个条目行含 resource
            for line in text.split("sources:")[1].splitlines()[:20]:
                if not line.strip().startswith(("-", " ")):
                    break
                if line.strip().startswith("- ") and "resource:" not in line:
                    errors.append(f"{rel}: sources 条目缺 resource:{line.strip()[:60]}")
                    break
    reserved_seen = {p.name for p in BUNDLE.rglob("*.md") if p.name in ("index.md", "log.md")}
    print(f"OKF v0.2 check on {BUNDLE}: {concepts} concept docs, reserved files: {sorted(reserved_seen)}")
    if errors:
        print(f"FAIL ({len(errors)}):")
        for e in errors:
            print(f"  - {e}")
        return 1
    print("PASS: 0 errors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
