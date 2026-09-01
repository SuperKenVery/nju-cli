# ehall：我的研究生培养方案

用于读取当前统一认证用户在研究生院系统中的培养方案，包括方案概况、培养目标等正文段落、课程设置和学分要求。

```sh
# 按网页展示顺序输出完整培养方案
nju-cli ehall graduate-training-program

# 输出结构化 JSON，包括接口返回的扩展字段
nju-cli ehall graduate-training-program --json
```

普通输出包含：

- 培养方案代码、名称、年级、院系、专业、方案类型和最低总学分；
- 培养目标、研究方向、招生对象、学习年限、教学与考核方式、学位论文等方案正文；
- 课程类别、课程代码、课程名、开课院系、学时、学分、学期、是否必修、备注和多选组；
- 各课程类别及合并组的最低学分要求。

## JSON 数据字典

需要程序化筛选课程、统计学分或辅助制定选课计划时，可使用 JSON：

```sh
nju-cli ehall graduate-training-program --json > /tmp/graduate-training-program.json
```

顶层结构如下：

```json
{
  "summary": {},
  "sections": [],
  "courses": [],
  "credit_requirements": {}
}
```

命令会把尚未专门建模的接口字段原样保留，因此实际 JSON 可能包含下表之外的字段。脚本应只依赖这里记录的核心字段，并允许可选字段为 `null` 或缺失。

### `summary`：方案概况

| 字段 | 含义 |
| ---- | ---- |
| `DM` | 培养方案代码 |
| `MC` | 培养方案名称 |
| `NJDM` / `NJDM_DISPLAY` | 年级代码 / 显示名称 |
| `YXDM` / `YXDM_DISPLAY` | 院系代码 / 院系名称 |
| `ZYDM` / `ZYDM_DISPLAY` | 专业代码 / 专业名称 |
| `YJXK` / `YJXK_DISPLAY` | 一级学科代码 / 名称 |
| `FALXDM` / `FALXDM_DISPLAY` | 方案类型代码 / 名称，如直博生培养方案 |
| `SYXSLBMC` | 适用学生类别 |
| `ZDXF` | 方案标注的最低总学分 |

读取方案摘要：

```sh
jq '.summary | {
  id: .DM,
  name: .MC,
  grade: .NJDM_DISPLAY,
  department: .YXDM_DISPLAY,
  major: .ZYDM_DISPLAY,
  program_type: .FALXDM_DISPLAY,
  minimum_credits: .ZDXF
}' /tmp/graduate-training-program.json
```

### `sections`：方案正文段落

每个元素代表网页中的一个方案段落：

| 字段 | 含义 |
| ---- | ---- |
| `WID` | 段落唯一标识 |
| `MC` | 标题，如培养目标、研究方向、课程设置 |
| `MRZ` | 正文；没有文本内容时可能为 `null` |
| `SM` | 补充说明 |
| `LX` | 段落类型；接口可能返回数字或数字字符串 |
| `PX` | 显示顺序 |

已知的 `LX`：`0` 为普通文本，`1` 为富文本，`2` 为学分要求，`3` 为课程设置，`4` 为附件，`5` 为链接，`9` 为专业方向，`12` 为必修环节。课程与学分要求的实际数据分别在顶层 `courses` 和 `credit_requirements`，不能只读取该段的 `MRZ`。

输出所有非空正文：

```sh
jq -r '.sections[]
  | select((.MRZ // "") != "")
  | "## \(.MC)\n\(.MRZ)\n"' \
  /tmp/graduate-training-program.json
```

读取指定段落：

```sh
jq -r '.sections[]
  | select(.MC == "教学与考核方式")
  | .MRZ // .SM // ""' \
  /tmp/graduate-training-program.json
```

### `courses`：培养方案课程

`courses` 是培养方案允许或要求修读的课程集合，**不是已经选中或已经修完的课程记录**。

| 字段 | 含义 |
| ---- | ---- |
| `FADM` | 所属培养方案代码 |
| `KCDM` | 课程代码；选课分析时应使用它作为课程标识 |
| `KCMC` | 课程名称；名称可能重复，不能替代课程代码 |
| `KCLBDM` / `KCLBDM_DISPLAY` | 课程类别代码 / 名称，如 A、B、C、D、X 类 |
| `KKDWDM` / `KKDWDM_DISPLAY` | 开课单位代码 / 名称 |
| `ZXS` | 总学时 |
| `XF` | 学分；零学分不代表不需要修读 |
| `KKXQDM` / `KKXQ` | 建议修读学期代码 / 显示名称 |
| `SFBX` / `SFBX_DISPLAY` | 是否必修的原始值 / 显示值 |
| `BZ` | 备注，可能包含二选一、无需选课、导师给分等关键规则 |
| `DXZXSMC` | 多选组名称；没有多选组时通常为 `null` |
| `ZYFXDM` / `ZYFXDM_DISPLAY` | 专业或研究方向代码 / 名称 |

判断必修时优先使用 `SFBX_DISPLAY == "是"`，不要假设 `SFBX` 一定是字符串或数字。任何选课建议都必须同时检查 `BZ`；例如系统可能把汇总课标为必修，但备注说明它由其他课程成绩组成。

列出系统标记的全部必修课：

```sh
jq -r '.courses[]
  | select(.SFBX_DISPLAY == "是")
  | [
      .KCLBDM_DISPLAY,
      .KCDM,
      .KCMC,
      (.XF // 0),
      (.KKXQ // ""),
      (.BZ // "")
    ]
  | @tsv' /tmp/graduate-training-program.json
```

列出零学分必修课：

```sh
jq -r '.courses[]
  | select(.SFBX_DISPLAY == "是" and ((.XF // 0) == 0))
  | [.KCDM, .KCMC, (.KKXQ // ""), (.BZ // "")]
  | @tsv' /tmp/graduate-training-program.json
```

按建议学期筛选课程：

```sh
jq -r --arg term "第一学期" '.courses[]
  | select(.KKXQ == $term)
  | [.KCLBDM_DISPLAY, .KCDM, .KCMC, (.XF // 0), .SFBX_DISPLAY, (.BZ // "")]
  | @tsv' /tmp/graduate-training-program.json
```

按课程类别筛选：

```sh
jq -r --arg category "C类" '.courses[]
  | select(.KCLBDM_DISPLAY == $category)
  | [.KCDM, .KCMC, (.XF // 0), (.KKXQ // ""), .SFBX_DISPLAY]
  | @tsv' /tmp/graduate-training-program.json
```

找出带有重要备注的课程：

```sh
jq -r '.courses[]
  | select((.BZ // "") != "")
  | [.KCDM, .KCMC, (.XF // 0), (.KKXQ // ""), .SFBX_DISPLAY, .BZ]
  | @tsv' /tmp/graduate-training-program.json
```

找出明确的二选一课程：

```sh
jq -r '.courses[]
  | select((.BZ // "") | contains("二选一"))
  | [.KCDM, .KCMC, (.XF // 0), (.KKXQ // "")]
  | @tsv' /tmp/graduate-training-program.json
```

按学期汇总系统标记的必修课及其学分：

```sh
jq '[.courses[] | select(.SFBX_DISPLAY == "是")]
  | sort_by(.KKXQ // "")
  | group_by(.KKXQ // "")
  | map({
      term: (.[0].KKXQ // "未指定"),
      credits: ([.[].XF // 0] | add // 0),
      courses: [.[] | {
        code: .KCDM,
        name: .KCMC,
        credits: (.XF // 0),
        note: (.BZ // "")
      }]
    })' /tmp/graduate-training-program.json
```

评估一组候选课程的总学分和分类学分时，使用课程代码显式指定，不要按名称匹配：

```sh
jq --argjson ids '["081200C06", "081200C07", "085400D24"]' '
  [.courses[] | select(.KCDM as $code | $ids | index($code))] as $picked
  | {
      courses: [$picked[] | {code: .KCDM, name: .KCMC, category: .KCLBDM_DISPLAY, credits: (.XF // 0)}],
      total_credits: ([$picked[].XF // 0] | add // 0),
      credits_by_category: (
        $picked
        | sort_by(.KCLBDM_DISPLAY // "")
        | group_by(.KCLBDM_DISPLAY // "")
        | map({
            category: (.[0].KCLBDM_DISPLAY // "未分类"),
            credits: ([.[].XF // 0] | add // 0)
          })
      )
    }' /tmp/graduate-training-program.json
```

### `credit_requirements`：学分要求

结构如下：

```json
{
  "falxzxfyqResults": [],
  "falxdykclbxfyqResults": [],
  "falxdykclbxfyqhbzResults": []
}
```

- `falxzxfyqResults`：方案总体要求，通常重复包含方案概况。
- `falxdykclbxfyqResults`：每个课程类别的要求。
- `falxdykclbxfyqhbzResults`：多个课程类别合并计算时的合并组要求。

课程类别要求的核心字段：

| 字段 | 含义 |
| ---- | ---- |
| `FADM` | 培养方案代码 |
| `KCLBDM` / `KCLBDM_DISPLAY` | 课程类别代码 / 名称 |
| `ZDXF` | 该类别单独要求的最低学分 |
| `HBZWID` | 合并组 ID；为 `null` 表示没有合并组 |

合并组要求的核心字段：

| 字段 | 含义 |
| ---- | ---- |
| `WID` | 合并组 ID，与类别记录的 `HBZWID` 对应 |
| `FADM` | 培养方案代码 |
| `ZDXF` | 该组内所有关联类别合计需要达到的最低学分 |

如果 C 类和 D 类拥有相同的 `HBZWID`，且对应组的 `ZDXF` 为 13，含义是 **C+D 合计至少 13 学分**，不是 C、D 各修 13 学分。不要把各类别的 `ZDXF` 与合并组 `ZDXF` 重复相加。

把类别要求与合并组要求关联起来：

```sh
jq '.credit_requirements as $requirements
  | $requirements.falxdykclbxfyqResults[]
  | {
      category: .KCLBDM_DISPLAY,
      category_minimum_credits: (.ZDXF // 0),
      group_id: .HBZWID,
      group_minimum_credits: (
        .HBZWID as $group_id
        | [
            $requirements.falxdykclbxfyqhbzResults[]
            | select(.WID == $group_id)
            | .ZDXF
          ][0] // null
      )
    }' /tmp/graduate-training-program.json
```

## Agent 分析流程

当用户询问“哪些课必须选”“应该选什么课”或要求制定选课计划时：

1. 执行一次 `--json` 并写入临时文件，避免反复请求接口。
2. 先读取 `summary.ZDXF`、类别要求和合并组要求，不要只看最低总学分。
3. 用 `SFBX_DISPLAY` 筛选系统必修课，再逐条检查 `BZ`、学期和零学分课程。
4. 用 `KKXQ` 区分建议学期，但明确它不代表该课程当学期一定实际开设，也不包含上课时间和冲突信息。
5. 用 `KCDM` 标识和组合课程；同名课程可能有不同代码、学期或类别。
6. 专业选修课应结合培养方案正文中的研究方向、导师意见和用户偏好。若用户没有提供方向，应给出通用方案和按方向划分的替代方案，并把推荐明确标注为建议而非培养方案硬性要求。
7. 如果最低总学分与分类/合并组要求冲突，指出冲突并优先按更严格的分类约束规划，同时建议向导师或研究生秘书确认，不要自行断言某一口径无效。
8. 对合成课程、二选一、导师给分、无需选课等情况，以 `BZ` 为依据；语义仍不明确时必须提示用户确认，不能仅根据 `SFBX_DISPLAY` 推断操作方式。
9. 分析完成后删除包含个人培养方案的临时文件。

可先检查实际数据有哪些字段，再使用上面的稳定字段：

```sh
jq 'keys' /tmp/graduate-training-program.json
jq '.summary | keys' /tmp/graduate-training-program.json
jq '.sections[0] | keys' /tmp/graduate-training-program.json
jq '.courses[0] | keys' /tmp/graduate-training-program.json
jq '.credit_requirements | keys' /tmp/graduate-training-program.json
```

## 会话说明


如果提示登录失效，请重新执行：

```sh
nju-cli login --username USERNAME --password PASSWORD
```
