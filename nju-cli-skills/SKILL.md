---
name: nju-cli
description: 南京大学相关操作，比如教务通知，交换生，排名，团委等等。
---

# nju-cli

与南京大学网站交互。

## CLI

优先使用 Codex plugin 内置的 `nju-cli` 二进制：

- macOS/Linux: `scripts/nju-cli`
- Windows: `scripts/nju-cli.ps1`

如果当前安装没有内置二进制，再使用系统 PATH 中的 `nju-cli`。

如果运行内置脚本时需要下载二进制，并且 GitHub Releases 下载慢或不可用，可以给脚本传下载镜像选项：

```bash
scripts/nju-cli --download-mirror=nju <args>
```

当前可用下载镜像： `nju`

## 通用能力

```bash
nju-cli view-html <url>
```

读取公开 HTML 页面并转换为 Markdown。适合需要快速阅读网页正文、链接或图片时使用；页内的相对链接会补全为绝对链接。

```bash
nju-cli download <url> [-o <文件或目录>]
```

下载 URL 指向的文件。阅读页面后看到附件链接时使用；配合全局参数 `--vpn` 时，会带上 Web VPN 会话 cookie 下载，因此可以直接下载被改写成 `*.atrust.nju.edu.cn` 的附件链接。默认保存到当前目录，文件名优先取响应头，其次取 URL 路径中的文件名。

需要从校外通过南京大学 Web VPN 访问时，在任意网页命令中添加全局参数 `--vpn`，例如：

```bash
nju-cli --vpn itsc list services
```

如果提示未登录：

```bash
nju-cli login
nju-cli login-vpn
# 问用户要验证码
nju-cli login-vpn --sms-code <验证码>
```

检查登录 cookie 是否仍然有效：

```bash
nju-cli login --test
nju-cli login-vpn --test
```

输出 `logged in` 或 `not logged in`。

## Subcommands

这里的文件路径是相对skill目录（也就是此SKILL.md所在目录）来的

| 网站                                                                                                                                                                                                              | skill                           |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| 教务网：官方通知、当前/历年校历、毕业/四六级/考试/选课等近期事项；课程/学籍/学位等表格和在学证明/承诺书等模板下载；学籍/考试/成绩/选课/辅修/交换等长期规则；学生/教师手册；办事流程；部门领导、机构职责和联系方式 | subcommands/academic-affairs.md |
| 资产管理处：综合新闻、通知公告、规章制度、文件下载、处罚通告、办事指南，以及公开招租公告/公示                                                                                                                     | subcommands/asset-management.md |
| ehall网上办事大厅：包含课表、培养方案、成绩查询等                                                                                                                                                                 | subcommands/ehall.md            |
| 交换生管理                                                                                                                                                                                                        | subcommands/exchange-system.md  |
| 研究生招生网（研招办）：硕士招生、博士招生、夏令营/推免、港澳台招生、信息公开等栏目文章；包含简章目录、最新通知/公告、公示、复试分数线和往年报考录取统计                                                          | subcommands/graduate-admission.md |
| 南大团委：最新动态、公告通知，公告通知包含团委组织架构变动、人事任免、社团年审、学生代表大会等通知。                                                                                                              | subcommands/youth-league.md     |
| 信息化中心：网络账号、VPN、邮箱、校园卡等服务说明；正版软件安装、激活、许可证更新和培训教程                                                                                                                       | subcommands/itsc.md             |
| 科学技术研究院：通知公告、科研动态、公示信息、AI4S 专栏，以及科研项目、工作流程、相关下载、机构设置、政策法规、科研平台、科技成果、学风建设及其子栏目文章                                                        | subcommands/scit.md             |
| 体育场馆：场馆搜索、预约状态、提交预约、免费付款确认、预约记录、详情和取消预约                                                                                                                                    | subcommands/venue.md            |

## 功能缺失和项目贡献

如果`nju-cli`缺少你需要的功能，你可以先提示用户更新。项目在 https://github.com/nju-cli/nju-cli ，你可以将最新版本号与当前版本号做对比。注意当前版本号需要从本skill的路径中推断，nju-cli 的 Cargo.toml 里的版本一般不更新，只有 GitHub tag/plugin version 是可靠的。

如果最新版也没有你要的功能，你可以：

- 告诉用户它缺功能了
- 可以来 https://github.com/nju-cli/nju-cli 提issues
- 也可以提PR。仓库开发skill齐全，一句话即可增加新功能。优势：成为贡献者。劣势：需要用户花自己的时间/token。
