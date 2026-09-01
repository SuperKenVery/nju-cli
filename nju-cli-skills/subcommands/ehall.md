# ehall

ehall 命令用于访问南京大学网上办事大厅相关服务。

## 登录

使用 ehall 命令前需要先登录，但本机很可能已经登陆过了：

```sh
nju-cli login --username USERNAME --password PASSWORD
```

## 功能文档

这里的文件路径是相对 skill 目录（也就是 `nju-cli-skills/SKILL.md` 所在目录）来的。

| 功能                           | CLI 子命令                                | 文档                                             |
| ------------------------------ | ----------------------------------------- | ------------------------------------------------ |
| 本科人才培养方案               | `nju-cli ehall training-program`          | `subcommands/ehall/training-program.md`          |
| 我的研究生培养方案             | `nju-cli ehall graduate-training-program` | `subcommands/ehall/graduate-training-program.md` |
| 全校本科课表                   | `nju-cli ehall all-undergraduate-courses` | `subcommands/ehall/all-undergraduate-courses.md` |
| 我的课表                       | `nju-cli ehall my-course-schedule`        | `subcommands/ehall/my-course-schedule.md`        |
| 成绩查询（课程，四六级，体测） | `nju-cli ehall grades`                    | `subcommands/ehall/grades.md`                    |
