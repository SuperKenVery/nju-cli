# graduate-school

南京大学研究生院公开网站（`grawww.nju.edu.cn`）栏目读取。覆盖通知公告、办事指南、机构简介、培养工作、学位与导师，以及学籍与奖助管理。

该命令不同于 `graduate-admission`：

- `graduate-school`：研究生培养、学位、导师、学籍、奖助和办事指南。
- `graduate-admission`：研究生招生网的硕士、博士、推免和招生信息公开。

## 查看支持栏目

```bash
nju-cli graduate-school columns
```

每行开头会标明类型：

- `list`：文章或附件列表，使用 `list`、`view`。
- `page`：机构介绍等静态页面，使用 `page`。

## 列出文章

```bash
# 通知公告第一页
nju-cli graduate-school list notifications

# 通知公告第三页
nju-cli graduate-school list notifications --page 3

# 拉取办事指南全部页面
nju-cli graduate-school list service-guides --all
```

`list` 输出文章 ID、发布日期和标题，并缓存 ID 与 URL。每次普通 `list` 会缓存当前页；`--all` 会缓存整个栏目。

### 培养工作

```bash
nju-cli graduate-school list training-regulations
nju-cli graduate-school list national-sponsored-programs
nju-cli graduate-school list university-exchange-programs
nju-cli graduate-school list training-innovation-projects
nju-cli graduate-school list combined-master-doctor
nju-cli graduate-school list course-development
nju-cli graduate-school list training-student-status
nju-cli graduate-school list training-downloads
```

对应相关规定、国际交流下的高水平公派/校级交流项目、各类项目下的创新工程/硕博连读/课程建设、学籍相关和下载专区。

### 学位与导师

```bash
nju-cli graduate-school list degree-work
nju-cli graduate-school list supervisor-work
nju-cli graduate-school list excellent-theses
nju-cli graduate-school list degree-improvement-program
nju-cli graduate-school list degree-downloads
nju-cli graduate-school list honorary-doctors
```

对应学位工作、导师工作、项目管理下的优秀学位论文/提升计划、下载专区和名誉博士。

### 学籍与奖助管理

```bash
nju-cli graduate-school list student-policies
nju-cli graduate-school list graduation-certificates
nju-cli graduate-school list student-downloads
nju-cli graduate-school list student-registration
```

对应相关政策规定，以及学籍学历管理下的毕业证书、下载专区和学籍注册。当前下载专区没有条目时会输出空列表。结业证书和学籍异动是静态页面，使用下方的 `page` 命令。

## 读取文章或附件

先列出并缓存对应栏目，再传入文章 ID：

```bash
nju-cli graduate-school list notifications --page 2
nju-cli graduate-school view notifications 835568
```

普通文章会输出 Markdown 正文，并补全正文中的相对链接。列表项如果直接指向 PDF、Word 等附件，`view` 会输出附件名称和下载 URL，可再使用通用下载命令：

```bash
nju-cli graduate-school list training-downloads
nju-cli graduate-school view training-downloads <附件ID>
nju-cli download <附件URL>
```

## 读取机构与静态页面

```bash
nju-cli graduate-school page graduate-school-introduction
nju-cli graduate-school page department-leaders
nju-cli graduate-school page general-office
nju-cli graduate-school page admissions-office
nju-cli graduate-school page training-office
nju-cli graduate-school page development-office
nju-cli graduate-school page degree-office
nju-cli graduate-school page student-affairs-office
nju-cli graduate-school page suzhou-office
nju-cli graduate-school page degree-committee
nju-cli graduate-school page completion-certificates
nju-cli graduate-school page student-status-changes
```

这些页面覆盖研究生院简介、部门领导、机构设置下的各办公室、学位评定委员会，以及学籍学历管理下的结业证书和学籍异动。输出为 Markdown。
