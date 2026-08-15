use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use serde_json::Value;

#[derive(Debug, Subcommand)]
pub enum EhallCommand {
    /// 本科人才培养方案查询。
    #[command(name = "training-program")]
    TrainingProgram {
        #[command(subcommand)]
        command: TrainingProgramCommand,
    },
    /// 本科全校课表查询。
    #[command(name = "all-undergraduate-courses")]
    CourseSchedule {
        #[command(subcommand)]
        command: CourseScheduleCommand,
    },
    /// 我的课表、课程详情、考试说明和免修不免考。
    #[command(name = "my-course-schedule")]
    MyCourseSchedule {
        #[command(subcommand)]
        command: MyCourseScheduleCommand,
    },
    /// 我的成绩查询。
    #[command(name = "grades")]
    Grades {
        #[command(subcommand)]
        command: GradeCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum TrainingProgramCommand {
    /// 列出所有专业培养方案。
    List(ListTrainingProgramsOptions),
    /// 查看某专业培养方案详情。
    Detail {
        /// 培养方案代码 PYFADM。
        program_id: String,
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
    /// 列出培养方案内的节点。
    Nodes {
        /// 培养方案代码 PYFADM。
        program_id: String,
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
    /// 查看培养方案内某个节点的详情。
    #[command(name = "node-detail")]
    NodeDetail {
        /// 培养方案代码 PYFADM。
        program_id: String,
        /// 节点代码 KZH。
        node_id: String,
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
    /// 查看培养方案内某个节点里的所有课程。
    Courses {
        /// 培养方案代码 PYFADM。
        program_id: String,
        /// 节点代码 KZH。
        node_id: String,
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Args)]
pub struct ListTrainingProgramsOptions {
    /// 每次请求页大小。
    #[arg(long, default_value_t = 200)]
    page_size: u64,
    /// 按培养方案名称模糊搜索。
    #[arg(long)]
    name: Option<String>,
    /// 年级代码，如 2026。
    #[arg(long)]
    grade: Option<String>,
    /// 院系代码 DWDM。
    #[arg(long)]
    department: Option<String>,
    /// 修读类型代码 XDLXDM，如 01。
    #[arg(long)]
    study_type: Option<String>,
    /// 输出完整 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
pub enum CourseScheduleCommand {
    /// 列出一页全校本科课程。
    List(ListCourseSchedulesOptions),
    /// 下载所有匹配的全校本科课程。
    Download(DownloadCourseSchedulesOptions),
}

#[derive(Debug, Subcommand)]
pub enum MyCourseScheduleCommand {
    /// 列出可选学期。
    Terms {
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
    /// 列出指定学期下我的课表。
    List(ListMyCoursesOptions),
    /// 显示或下载课程详情，参数可以是课程号 KCH 或教学班 JXBID。
    Detail(MyCourseDetailOptions),
    /// 显示网页底部的考试说明和上课冲突说明。
    #[command(name = "exam-notes")]
    ExamNotes,
    /// 查看已经申请的免修不免考课程。
    Exemptions {
        /// 学年学期代码，如 2025-2026-2；不传则列出所有已申请记录。
        #[arg(long)]
        term: Option<String>,
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
    /// 申请某门课免修不免考。参数可以是课程号 KCH 或教学班 JXBID。
    #[command(name = "apply-exemption")]
    ApplyExemption {
        /// 课程号 KCH 或教学班 JXBID。
        course: String,
        /// 学年学期代码，如 2025-2026-2；默认当前学期。
        #[arg(long)]
        term: Option<String>,
        /// 申请理由。
        #[arg(long)]
        reason: String,
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum GradeCommand {
    /// 列出所有学期。
    Terms {
        /// 输出完整 JSON。
        #[arg(long)]
        json: bool,
    },
    /// 列出一页成绩。
    List(ListGradesOptions),
    /// 下载所有匹配成绩。
    Download(DownloadGradesOptions),
    /// 列出一页四六级成绩。
    Cet(ListCetGradesOptions),
    /// 下载所有匹配四六级成绩。
    #[command(name = "download-cet")]
    DownloadCet(DownloadCetGradesOptions),
    /// 列出一页体测成绩。
    Fitness(ListFitnessGradesOptions),
    /// 下载所有体测成绩。
    #[command(name = "download-fitness")]
    DownloadFitness(DownloadFitnessGradesOptions),
}

#[derive(Debug, Args)]
pub struct ListMyCoursesOptions {
    /// 学年学期代码，如 2025-2026-2；默认当前学期。
    #[arg(long)]
    term: Option<String>,
    /// 页码，从 1 开始。
    #[arg(long, default_value_t = 1)]
    page: u64,
    /// 每页数量。
    #[arg(long, default_value_t = 100)]
    page_size: u64,
    /// 输出完整 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct ListGradesOptions {
    /// 页码，从 1 开始。
    #[arg(long, default_value_t = 1)]
    page: u64,
    /// 每页数量。
    #[arg(long, default_value_t = 20)]
    page_size: u64,
    /// 输出完整 JSON。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: GradeQueryOptions,
}

#[derive(Debug, Args)]
pub struct DownloadGradesOptions {
    /// 每次请求页大小。
    #[arg(long, default_value_t = 100)]
    page_size: u64,
    /// 输出路径。默认按输出格式写到 grades.tsv 或 grades.json。
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// 下载为 JSON；默认下载 TSV。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: GradeQueryOptions,
}

#[derive(Debug, Args)]
pub struct GradeQueryOptions {
    /// 学年学期代码，如 2025-2026-2；可传多个。不传则使用页面默认展示学期。
    #[arg(long = "term")]
    terms: Vec<String>,
    /// 课程名 KCM，模糊匹配。
    #[arg(long)]
    course_name: Option<String>,
    /// 课程号 KCH，模糊匹配。
    #[arg(long)]
    course_id: Option<String>,
    /// 只看及格课程。
    #[arg(long, conflicts_with = "failed")]
    passed: bool,
    /// 只看未及格课程。
    #[arg(long, conflicts_with = "passed")]
    failed: bool,
    /// 显示最高成绩。
    #[arg(long)]
    show_max_grade: bool,
}

#[derive(Debug, Args)]
pub struct ListCetGradesOptions {
    /// 页码，从 1 开始。
    #[arg(long, default_value_t = 1)]
    page: u64,
    /// 每页数量。
    #[arg(long, default_value_t = 100)]
    page_size: u64,
    /// 输出完整 JSON。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: CetGradeQueryOptions,
}

#[derive(Debug, Args)]
pub struct DownloadCetGradesOptions {
    /// 每次请求页大小。
    #[arg(long, default_value_t = 100)]
    page_size: u64,
    /// 输出路径。默认按输出格式写到 cet-grades.tsv 或 cet-grades.json。
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// 下载为 JSON；默认下载 TSV。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: CetGradeQueryOptions,
}

#[derive(Debug, Args)]
pub struct CetGradeQueryOptions {
    /// 学年学期代码，如 2023-2024-1；可传多个。
    #[arg(long = "term")]
    terms: Vec<String>,
    /// 考试项目代码，如 CET4、CET6；不传默认查询 CET4 和 CET6。
    #[arg(long = "exam-type")]
    exam_types: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ListFitnessGradesOptions {
    /// 页码，从 1 开始。
    #[arg(long, default_value_t = 1)]
    page: u64,
    /// 每页数量。
    #[arg(long, default_value_t = 100)]
    page_size: u64,
    /// 输出完整 JSON。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: FitnessGradeQueryOptions,
}

#[derive(Debug, Args)]
pub struct DownloadFitnessGradesOptions {
    /// 每次请求页大小。
    #[arg(long, default_value_t = 100)]
    page_size: u64,
    /// 输出路径。默认按输出格式写到 fitness-grades.tsv 或 fitness-grades.json。
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// 下载为 JSON；默认下载 TSV。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: FitnessGradeQueryOptions,
}

#[derive(Debug, Args)]
pub struct FitnessGradeQueryOptions {
    /// 学年学期代码，如 2025-2026-2；可传多个。
    #[arg(long = "term")]
    terms: Vec<String>,
}

#[derive(Debug, Args)]
pub struct MyCourseDetailOptions {
    /// 课程号 KCH 或教学班 JXBID，支持多个。
    courses: Vec<String>,
    /// 学年学期代码，如 2025-2026-2；默认当前学期。
    #[arg(long)]
    term: Option<String>,
    /// 下载到目录；不传则直接输出。
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ListCourseSchedulesOptions {
    /// 页码，从 1 开始。
    #[arg(long, default_value_t = 1)]
    page: u64,
    /// 每页数量。
    #[arg(long, default_value_t = 20)]
    page_size: u64,
    /// 输出完整 JSON。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: CourseScheduleQueryOptions,
}

#[derive(Debug, Args)]
pub struct DownloadCourseSchedulesOptions {
    /// 每次请求页大小。
    #[arg(long, default_value_t = 200)]
    page_size: u64,
    /// 输出路径。默认按输出格式写到 all-undergraduate-courses.tsv 或 all-undergraduate-courses.json。
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// 下载为 JSON；默认下载 TSV。
    #[arg(long)]
    json: bool,
    #[command(flatten)]
    query: CourseScheduleQueryOptions,
}

#[derive(Debug, Args)]
pub struct CourseScheduleQueryOptions {
    /// 学年学期代码，如 2025-2026-2；默认当前学期。
    #[arg(long)]
    term: Option<String>,
    /// 课程号 KCH，模糊匹配。
    #[arg(long)]
    course_id: Option<String>,
    /// 课程名 KCM，模糊匹配。
    #[arg(long)]
    course_name: Option<String>,
    /// 教学班名称 JXBMC，模糊匹配。
    #[arg(long)]
    class_name: Option<String>,
    /// 上课教师 SKJS，模糊匹配。
    #[arg(long)]
    teacher: Option<String>,
    /// 校区代码 XXXQDM，如 1/3/4。
    #[arg(long)]
    campus: Option<String>,
    /// 开课单位代码 PKDWDM。
    #[arg(long)]
    department: Option<String>,
    /// 通修课程类别代码 TXKCLB。
    #[arg(long)]
    general_category: Option<String>,
    /// 上课日期 SKXQ，如 周一。
    #[arg(long)]
    weekday: Option<String>,
    /// 开始节次 KSJC。
    #[arg(long)]
    start_period: Option<String>,
    /// 结束节次 JSJC。
    #[arg(long)]
    end_period: Option<String>,
    /// 上课周次 SKZC。该字段通常是页面周次选择器生成的值。
    #[arg(long)]
    week: Option<String>,
    /// 教学楼代码 JXLDM。
    #[arg(long)]
    building: Option<String>,
    /// 上课教室 SKJAS，模糊匹配。
    #[arg(long)]
    classroom: Option<String>,
    /// 任意字段筛选，格式 FIELD=VALUE 或 FIELD:BUILDER=VALUE。
    #[arg(long = "filter")]
    filters: Vec<String>,
}

pub async fn handle(command: EhallCommand, client: &common::Client) -> Result<()> {
    match command {
        EhallCommand::TrainingProgram { command } => {
            ehall::training_program::prepare_session(
                client,
                ehall::training_program::default_role_id(),
            )
            .await
            .context("failed to prepare ehall training program session; try running `nju-cli login` again")?;
            handle_training_program(command, client).await
        }
        EhallCommand::CourseSchedule { command } => {
            ehall::course_schedule::prepare_session(
                client,
                ehall::course_schedule::default_role_id(),
            )
            .await
            .context("failed to prepare ehall course schedule session; try running `nju-cli login` again")?;
            handle_course_schedule(command, client).await
        }
        EhallCommand::MyCourseSchedule { command } => {
            ehall::my_course_schedule::prepare_schedule_session(
                client,
                ehall::my_course_schedule::default_role_id(),
            )
            .await
            .context("failed to prepare ehall my course schedule session; try running `nju-cli login` again")?;
            handle_my_course_schedule(command, client).await
        }
        EhallCommand::Grades { command } => {
            ehall::grade_query::prepare_session(client, ehall::grade_query::default_role_id())
                .await
                .context(
                    "failed to prepare ehall grade query session; try running `nju-cli login` again",
                )?;
            handle_grades(command, client).await
        }
    }
}

async fn handle_training_program(
    command: TrainingProgramCommand,
    client: &common::Client,
) -> Result<()> {
    match command {
        TrainingProgramCommand::List(options) => {
            let programs = ehall::training_program::list_all_training_programs(
                client,
                &ehall::training_program::TrainingProgramListOptions {
                    page_number: 1,
                    page_size: options.page_size,
                    name: options.name,
                    grade: options.grade,
                    department: options.department,
                    study_type: options.study_type,
                },
            )
            .await
            .context("failed to list training programs")?;

            if options.json {
                println!("{}", serde_json::to_string_pretty(&programs)?);
            } else {
                for program in programs {
                    println!(
                        "{}\t{}\t{}\t{}\t{}",
                        program.id,
                        program.name,
                        program.grade_display.unwrap_or_default(),
                        program.department_name.unwrap_or_default(),
                        program.major_name.unwrap_or_default()
                    );
                }
            }
        }
        TrainingProgramCommand::Detail { program_id, json } => {
            let program = ehall::training_program::get_training_program_detail(client, &program_id)
                .await
                .with_context(|| format!("failed to get training program {program_id}"))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&program)?);
            } else {
                println!("{} {}", program.id, program.name);
                println!("年级: {}", program.grade_display.unwrap_or_default());
                println!("院系: {}", program.department_name.unwrap_or_default());
                println!("专业: {}", program.major_name.unwrap_or_default());
                println!("修读类型: {}", program.study_type_name.unwrap_or_default());
                if let Some(objective) = program.objective {
                    println!("\n培养目标:\n{objective}");
                }
                if let Some(requirement) = program.requirement {
                    println!("\n修读要求:\n{requirement}");
                }
                if let Some(graduation_requirement) = program.graduation_requirement {
                    println!("\n准出要求:\n{graduation_requirement}");
                }
            }
        }
        TrainingProgramCommand::Nodes { program_id, json } => {
            let nodes = ehall::training_program::list_training_program_nodes(client, &program_id)
                .await
                .with_context(|| {
                    format!("failed to list nodes of training program {program_id}")
                })?;

            if json {
                println!("{}", serde_json::to_string_pretty(&nodes)?);
            } else {
                print_training_program_nodes(&program_id, &nodes);
            }
        }
        TrainingProgramCommand::NodeDetail {
            program_id,
            node_id,
            json,
        } => {
            let node = ehall::training_program::get_training_program_node_detail(
                client,
                &program_id,
                &node_id,
            )
            .await
            .with_context(|| format!("failed to get detail of training program node {node_id}"))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&node)?);
            } else {
                print_training_program_node_detail(&node);
            }
        }
        TrainingProgramCommand::Courses {
            program_id,
            node_id,
            json,
        } => {
            let courses =
                ehall::training_program::list_all_node_courses(client, &program_id, &node_id)
                    .await
                    .with_context(|| {
                        format!("failed to list courses of training program node {node_id}")
                    })?;

            if json {
                println!("{}", serde_json::to_string_pretty(&courses)?);
            } else {
                for course in courses {
                    println!(
                        "{}\t{}\t{}\t{}\t{}",
                        course.course_id,
                        course.course_name,
                        course.credits.unwrap_or_default(),
                        course.term.unwrap_or_default(),
                        course.department_name.unwrap_or_default()
                    );
                }
            }
        }
    }

    Ok(())
}

async fn handle_course_schedule(
    command: CourseScheduleCommand,
    client: &common::Client,
) -> Result<()> {
    match command {
        CourseScheduleCommand::List(options) => {
            let page = ehall::course_schedule::list_course_schedules(
                client,
                &ehall::course_schedule::CourseScheduleListOptions {
                    page_number: options.page,
                    page_size: options.page_size,
                    term: options.query.term.clone(),
                    filters: course_schedule_filters(&options.query)?,
                },
            )
            .await
            .context("failed to list course schedules")?;

            if options.json {
                println!("{}", serde_json::to_string_pretty(&page)?);
            } else {
                for course in page.rows {
                    print_course_schedule_row(&course);
                }
            }
        }
        CourseScheduleCommand::Download(options) => {
            let courses = ehall::course_schedule::list_all_course_schedules(
                client,
                &ehall::course_schedule::CourseScheduleListOptions {
                    page_number: 1,
                    page_size: options.page_size,
                    term: options.query.term.clone(),
                    filters: course_schedule_filters(&options.query)?,
                },
            )
            .await
            .context("failed to download course schedules")?;
            let output = options.output.unwrap_or_else(|| {
                if options.json {
                    PathBuf::from("all-undergraduate-courses.json")
                } else {
                    PathBuf::from("all-undergraduate-courses.tsv")
                }
            });

            if options.json {
                let json = serde_json::to_string_pretty(&courses)
                    .context("failed to serialize course schedules")?;
                std::fs::write(&output, json)
                    .with_context(|| format!("failed to write {}", output.display()))?;
            } else {
                std::fs::write(&output, course_schedules_to_tsv(&courses))
                    .with_context(|| format!("failed to write {}", output.display()))?;
            }
            println!("{} {}", courses.len(), output.display());
        }
    }

    Ok(())
}

async fn handle_my_course_schedule(
    command: MyCourseScheduleCommand,
    client: &common::Client,
) -> Result<()> {
    match command {
        MyCourseScheduleCommand::Terms { json } => {
            let terms = ehall::my_course_schedule::list_terms(client)
                .await
                .context("failed to list my course schedule terms")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&terms)?);
            } else {
                for term in terms {
                    println!("{}\t{}", term.id, term.name);
                }
            }
        }
        MyCourseScheduleCommand::List(options) => {
            let page = ehall::my_course_schedule::list_courses(
                client,
                &ehall::my_course_schedule::MyCourseListOptions {
                    term: options.term,
                    page_number: options.page,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to list my course schedule")?;

            if options.json {
                println!("{}", serde_json::to_string_pretty(&page)?);
            } else {
                for course in page.rows {
                    print_my_course_row(&course);
                }
            }
        }
        MyCourseScheduleCommand::Detail(options) => {
            if options.courses.is_empty() {
                return Err(anyhow!(
                    "please provide at least one course id or teaching class id"
                ));
            }

            if let Some(output_dir) = options.output_dir {
                std::fs::create_dir_all(&output_dir)
                    .with_context(|| format!("failed to create {}", output_dir.display()))?;
                for course in options.courses {
                    let detail = ehall::my_course_schedule::get_course_detail(
                        client,
                        options.term.as_deref(),
                        &course,
                    )
                    .await
                    .with_context(|| format!("failed to get detail for course {course}"))?;
                    let file_name = format!(
                        "{}-{}.json",
                        sanitize_filename::sanitize(&detail.schedule.course_id),
                        sanitize_filename::sanitize(&detail.schedule.teaching_class_id)
                    );
                    let path = output_dir.join(file_name);
                    let json = serde_json::to_string_pretty(&detail)
                        .context("failed to serialize my course detail")?;
                    std::fs::write(&path, json)
                        .with_context(|| format!("failed to write {}", path.display()))?;
                    println!("{}\t{}", course, path.display());
                }
            } else {
                let mut details = Vec::new();
                for course in options.courses {
                    details.push(
                        ehall::my_course_schedule::get_course_detail(
                            client,
                            options.term.as_deref(),
                            &course,
                        )
                        .await
                        .with_context(|| format!("failed to get detail for course {course}"))?,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&details)?);
            }
        }
        MyCourseScheduleCommand::ExamNotes => {
            let notes = ehall::my_course_schedule::get_exam_notes(client)
                .await
                .context("failed to read my course schedule exam notes")?;
            println!("{notes}");
        }
        MyCourseScheduleCommand::Exemptions { term, json } => {
            ehall::my_course_schedule::prepare_exemption_session(
                client,
                ehall::my_course_schedule::default_role_id(),
            )
            .await
            .context(
                "failed to prepare ehall exemption session; try running `nju-cli login` again",
            )?;
            let applications =
                ehall::my_course_schedule::list_exemption_applications(client, term.as_deref())
                    .await
                    .context("failed to list exemption applications")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&applications)?);
            } else {
                for application in applications {
                    println!(
                        "{}\t{}\t{}\t{}\t{}\t{}",
                        application.course_id,
                        application.course_name.as_deref().unwrap_or_default(),
                        application.term_id.as_deref().unwrap_or_default(),
                        application.status_name.as_deref().unwrap_or_default(),
                        application.teachers.as_deref().unwrap_or_default(),
                        application.reason.as_deref().unwrap_or_default()
                    );
                }
            }
        }
        MyCourseScheduleCommand::ApplyExemption {
            course,
            term,
            reason,
            json,
        } => {
            if reason.trim().is_empty() {
                return Err(anyhow!("please provide a non-empty --reason"));
            }
            ehall::my_course_schedule::prepare_exemption_session(
                client,
                ehall::my_course_schedule::default_role_id(),
            )
            .await
            .context(
                "failed to prepare ehall exemption session; try running `nju-cli login` again",
            )?;
            let result = ehall::my_course_schedule::apply_exemption(
                client,
                term.as_deref(),
                &course,
                &reason,
            )
            .await
            .with_context(|| format!("failed to apply exemption for {course}"))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else if let Some(application_id) = result.application_id {
                println!("submitted\t{application_id}");
            } else {
                println!("submitted");
            }
        }
    }

    Ok(())
}

async fn handle_grades(command: GradeCommand, client: &common::Client) -> Result<()> {
    match command {
        GradeCommand::Terms { json } => {
            let terms = ehall::grade_query::list_recent_terms(client)
                .await
                .context("failed to list grade query terms")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&terms)?);
            } else {
                for term in terms {
                    println!("{}\t{}", term.id, term.display_name());
                }
            }
        }
        GradeCommand::List(options) => {
            let passed = grade_passed_filter(&options.query);
            let page = ehall::grade_query::list_grades(
                client,
                &ehall::grade_query::GradeListOptions {
                    terms: options.query.terms,
                    course_name: options.query.course_name,
                    course_id: options.query.course_id,
                    passed,
                    show_max_grade: options.query.show_max_grade,
                    page_number: options.page,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to list grades")?;

            if options.json {
                println!("{}", serde_json::to_string_pretty(&page)?);
            } else {
                for grade in page.rows {
                    print_grade_row(&grade);
                }
            }
        }
        GradeCommand::Download(options) => {
            let passed = grade_passed_filter(&options.query);
            let grades = ehall::grade_query::list_all_grades(
                client,
                &ehall::grade_query::GradeListOptions {
                    terms: options.query.terms,
                    course_name: options.query.course_name,
                    course_id: options.query.course_id,
                    passed,
                    show_max_grade: options.query.show_max_grade,
                    page_number: 1,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to download grades")?;
            let output = options.output.unwrap_or_else(|| {
                if options.json {
                    PathBuf::from("grades.json")
                } else {
                    PathBuf::from("grades.tsv")
                }
            });

            if options.json {
                let json =
                    serde_json::to_string_pretty(&grades).context("failed to serialize grades")?;
                std::fs::write(&output, json)
                    .with_context(|| format!("failed to write {}", output.display()))?;
            } else {
                std::fs::write(&output, grades_to_tsv(&grades))
                    .with_context(|| format!("failed to write {}", output.display()))?;
            }
            println!("{} {}", grades.len(), output.display());
        }
        GradeCommand::Cet(options) => {
            let page = ehall::grade_query::list_cet_grades(
                client,
                &ehall::grade_query::CetGradeListOptions {
                    terms: options.query.terms,
                    exam_types: options.query.exam_types,
                    page_number: options.page,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to list CET grades")?;

            if options.json {
                println!("{}", serde_json::to_string_pretty(&page)?);
            } else {
                for grade in page.rows {
                    print_external_grade_row(&grade);
                }
            }
        }
        GradeCommand::DownloadCet(options) => {
            let grades = ehall::grade_query::list_all_cet_grades(
                client,
                &ehall::grade_query::CetGradeListOptions {
                    terms: options.query.terms,
                    exam_types: options.query.exam_types,
                    page_number: 1,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to download CET grades")?;
            let output = options.output.unwrap_or_else(|| {
                if options.json {
                    PathBuf::from("cet-grades.json")
                } else {
                    PathBuf::from("cet-grades.tsv")
                }
            });

            if options.json {
                let json = serde_json::to_string_pretty(&grades)
                    .context("failed to serialize CET grades")?;
                std::fs::write(&output, json)
                    .with_context(|| format!("failed to write {}", output.display()))?;
            } else {
                std::fs::write(&output, external_grades_to_tsv(&grades))
                    .with_context(|| format!("failed to write {}", output.display()))?;
            }
            println!("{} {}", grades.len(), output.display());
        }
        GradeCommand::Fitness(options) => {
            let page = ehall::grade_query::list_cet_grades(
                client,
                &ehall::grade_query::CetGradeListOptions {
                    terms: options.query.terms,
                    exam_types: vec![ehall::grade_query::FITNESS_EXAM_TYPE.to_string()],
                    page_number: options.page,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to list fitness grades")?;

            if options.json {
                println!("{}", serde_json::to_string_pretty(&page)?);
            } else {
                for grade in page.rows {
                    print_external_grade_row(&grade);
                }
            }
        }
        GradeCommand::DownloadFitness(options) => {
            let grades = ehall::grade_query::list_all_cet_grades(
                client,
                &ehall::grade_query::CetGradeListOptions {
                    terms: options.query.terms,
                    exam_types: vec![ehall::grade_query::FITNESS_EXAM_TYPE.to_string()],
                    page_number: 1,
                    page_size: options.page_size,
                },
            )
            .await
            .context("failed to download fitness grades")?;
            let output = options.output.unwrap_or_else(|| {
                if options.json {
                    PathBuf::from("fitness-grades.json")
                } else {
                    PathBuf::from("fitness-grades.tsv")
                }
            });

            if options.json {
                let json = serde_json::to_string_pretty(&grades)
                    .context("failed to serialize fitness grades")?;
                std::fs::write(&output, json)
                    .with_context(|| format!("failed to write {}", output.display()))?;
            } else {
                std::fs::write(&output, external_grades_to_tsv(&grades))
                    .with_context(|| format!("failed to write {}", output.display()))?;
            }
            println!("{} {}", grades.len(), output.display());
        }
    }

    Ok(())
}

fn grade_passed_filter(options: &GradeQueryOptions) -> Option<bool> {
    if options.passed {
        Some(true)
    } else if options.failed {
        Some(false)
    } else {
        None
    }
}

fn course_schedule_filters(
    options: &CourseScheduleQueryOptions,
) -> Result<Vec<ehall::course_schedule::CourseScheduleFilter>> {
    let mut filters = Vec::new();

    push_course_schedule_filter(&mut filters, "KCH", options.course_id.as_deref());
    push_course_schedule_filter(&mut filters, "KCM", options.course_name.as_deref());
    push_course_schedule_filter(&mut filters, "JXBMC", options.class_name.as_deref());
    push_course_schedule_filter(&mut filters, "SKJS", options.teacher.as_deref());
    push_course_schedule_filter(&mut filters, "XXXQDM", options.campus.as_deref());
    push_course_schedule_filter(&mut filters, "PKDWDM", options.department.as_deref());
    push_course_schedule_filter(&mut filters, "TXKCLB", options.general_category.as_deref());
    push_course_schedule_filter(&mut filters, "SKXQ", options.weekday.as_deref());
    push_course_schedule_filter(&mut filters, "KSJC", options.start_period.as_deref());
    push_course_schedule_filter(&mut filters, "JSJC", options.end_period.as_deref());
    push_course_schedule_filter(&mut filters, "SKZC", options.week.as_deref());
    push_course_schedule_filter(&mut filters, "JXLDM", options.building.as_deref());
    push_course_schedule_filter(&mut filters, "SKJAS", options.classroom.as_deref());

    for filter in &options.filters {
        filters.push(parse_course_schedule_filter(filter)?);
    }

    Ok(filters)
}

fn push_course_schedule_filter(
    filters: &mut Vec<ehall::course_schedule::CourseScheduleFilter>,
    name: &str,
    value: Option<&str>,
) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        filters.push(ehall::course_schedule::CourseScheduleFilter {
            name: name.to_string(),
            value: value.to_string(),
            builder: ehall::course_schedule::infer_filter_builder(name).to_string(),
        });
    }
}

fn parse_course_schedule_filter(
    filter: &str,
) -> Result<ehall::course_schedule::CourseScheduleFilter> {
    let (left, value) = filter
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid filter {filter:?}; expected FIELD=VALUE"))?;
    let (name, builder) = match left.split_once(':') {
        Some((name, builder)) => (name.trim(), builder.trim()),
        None => (
            left.trim(),
            ehall::course_schedule::infer_filter_builder(left.trim()),
        ),
    };
    let value = value.trim();

    if name.is_empty() || builder.is_empty() || value.is_empty() {
        return Err(anyhow!(
            "invalid filter {filter:?}; FIELD, BUILDER, and VALUE must be non-empty"
        ));
    }

    Ok(ehall::course_schedule::CourseScheduleFilter {
        name: name.to_string(),
        value: value.to_string(),
        builder: builder.to_string(),
    })
}

fn print_course_schedule_row(course: &ehall::course_schedule::CourseSchedule) {
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        course.course_id,
        course.teaching_class_name.as_deref().unwrap_or_default(),
        course.teachers.as_deref().unwrap_or_default(),
        course.time_place.as_deref().unwrap_or_default(),
        course.campus_name.as_deref().unwrap_or_default(),
        course
            .credits
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default(),
        course.department_name.as_deref().unwrap_or_default(),
        course
            .student_count
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default(),
        course.term_name.as_deref().unwrap_or_default()
    );
}

fn print_my_course_row(course: &ehall::my_course_schedule::MyCourse) {
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        course.course_id,
        course.display_course_name(),
        course.teachers.as_deref().unwrap_or_default(),
        course.time_place.as_deref().unwrap_or_default(),
        course.department_name.as_deref().unwrap_or_default(),
        course
            .credits
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default(),
        course.final_exam_info.as_deref().unwrap_or_default()
    );
}

fn print_grade_row(grade: &ehall::grade_query::Grade) {
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        grade.term_name.as_deref().unwrap_or(&grade.term_id),
        grade.course_id,
        grade.course_name,
        grade
            .credits
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default(),
        grade.course_nature_name.as_deref().unwrap_or_default(),
        grade
            .total_grade
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default(),
        grade.passed_name.as_deref().unwrap_or_default(),
        grade
            .transcript_mark
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default()
    );
}

fn print_external_grade_row(grade: &ehall::grade_query::CetGrade) {
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        grade.term_name.as_deref().unwrap_or(&grade.term_id),
        grade.exam_type_name,
        grade
            .score
            .as_ref()
            .and_then(display_value)
            .unwrap_or_default(),
        grade.passed_name.as_deref().unwrap_or_default(),
        grade.exam_date.as_deref().unwrap_or_default(),
        grade.department_name.as_deref().unwrap_or_default()
    );
}

fn course_schedules_to_tsv(courses: &[ehall::course_schedule::CourseSchedule]) -> String {
    let mut tsv = String::from(
        "课程号\t教学班名称\t上课教师\t时间地点\t校区\t学分\t开课单位\t学生数\t学年学期\t后续调课信息\t上课专业\t上课教室\n",
    );

    for course in courses {
        let row = vec![
            course.course_id.clone(),
            course
                .teaching_class_name
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            course.teachers.as_deref().unwrap_or_default().to_string(),
            course.time_place.as_deref().unwrap_or_default().to_string(),
            course
                .campus_name
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            course
                .credits
                .as_ref()
                .and_then(display_value)
                .unwrap_or_default(),
            course
                .department_name
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            course
                .student_count
                .as_ref()
                .and_then(display_value)
                .unwrap_or_default(),
            course.term_name.as_deref().unwrap_or_default().to_string(),
            course
                .reschedule_info
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            course
                .student_majors
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            course.classrooms.as_deref().unwrap_or_default().to_string(),
        ];
        tsv.push_str(
            &row.iter()
                .map(|value| escape_tsv_value(value.as_str()))
                .collect::<Vec<_>>()
                .join("\t"),
        );
        tsv.push('\n');
    }

    tsv
}

fn grades_to_tsv(grades: &[ehall::grade_query::Grade]) -> String {
    let mut tsv = String::from(
        "学年学期\t课程号\t课程名\t英文课程名\t学分\t课程性质\t总成绩\t是否及格\t成绩单标记\n",
    );

    for grade in grades {
        let row = vec![
            grade
                .term_name
                .as_deref()
                .unwrap_or(&grade.term_id)
                .to_string(),
            grade.course_id.clone(),
            grade.course_name.clone(),
            grade
                .english_course_name
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            grade
                .credits
                .as_ref()
                .and_then(display_value)
                .unwrap_or_default(),
            grade
                .course_nature_name
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            grade
                .total_grade
                .as_ref()
                .and_then(display_value)
                .unwrap_or_default(),
            grade.passed_name.as_deref().unwrap_or_default().to_string(),
            grade
                .transcript_mark
                .as_ref()
                .and_then(display_value)
                .unwrap_or_default(),
        ];
        tsv.push_str(
            &row.iter()
                .map(|value| escape_tsv_value(value.as_str()))
                .collect::<Vec<_>>()
                .join("\t"),
        );
        tsv.push('\n');
    }

    tsv
}

fn external_grades_to_tsv(grades: &[ehall::grade_query::CetGrade]) -> String {
    let mut tsv = String::from("学年学期\t考试项目\t成绩\t是否通过\t考试日期\t院系\t备注\n");

    for grade in grades {
        let row = vec![
            grade
                .term_name
                .as_deref()
                .unwrap_or(&grade.term_id)
                .to_string(),
            grade.exam_type_name.clone(),
            grade
                .score
                .as_ref()
                .and_then(display_value)
                .unwrap_or_default(),
            grade.passed_name.as_deref().unwrap_or_default().to_string(),
            grade.exam_date.as_deref().unwrap_or_default().to_string(),
            grade
                .department_name
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            grade.remark.as_deref().unwrap_or_default().to_string(),
        ];
        tsv.push_str(
            &row.iter()
                .map(|value| escape_tsv_value(value.as_str()))
                .collect::<Vec<_>>()
                .join("\t"),
        );
        tsv.push('\n');
    }

    tsv
}

fn escape_tsv_value(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

fn print_training_program_nodes(
    program_id: &str,
    nodes: &[ehall::training_program::TrainingProgramNode],
) {
    println!("PYFADM: {program_id}");
    println!("课程查询: nju-cli ehall training-program courses {program_id} <KZH>");
    println!("节点详情: nju-cli ehall training-program node-detail {program_id} <KZH>");
    println!();
    println!("节点树:");

    let id_set = nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let mut children = HashMap::<String, Vec<usize>>::new();
    let mut roots = Vec::new();

    for (index, node) in nodes.iter().enumerate() {
        match normalized_parent_id(node.parent_id.as_deref()) {
            Some(parent_id) if id_set.contains(parent_id) => {
                children
                    .entry(parent_id.to_string())
                    .or_default()
                    .push(index);
            }
            _ => roots.push(index),
        }
    }

    let mut visited = HashSet::new();
    for root in roots {
        print_training_program_node(nodes, &children, root, 0, &mut visited);
    }

    for index in 0..nodes.len() {
        if !visited.contains(&index) {
            print_training_program_node(nodes, &children, index, 0, &mut visited);
        }
    }
}

fn print_training_program_node(
    nodes: &[ehall::training_program::TrainingProgramNode],
    children: &HashMap<String, Vec<usize>>,
    index: usize,
    depth: usize,
    visited: &mut HashSet<usize>,
) {
    if !visited.insert(index) {
        return;
    }

    let node = &nodes[index];
    let indent = "  ".repeat(depth);
    println!(
        "{indent}- {} {}{}",
        node.name,
        format_node_metadata(node),
        format_node_note_hint(node)
    );

    if let Some(child_indexes) = children.get(&node.id) {
        for &child_index in child_indexes {
            print_training_program_node(nodes, children, child_index, depth + 1, visited);
        }
    }
}

fn format_node_metadata(node: &ehall::training_program::TrainingProgramNode) -> String {
    let mut parts = vec![format!("KZH={}", node.id)];

    if let Some(node_type) = node
        .node_type_name
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("类型={node_type}"));
    }
    if let Some(category) = node
        .course_category_name
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("类别={category}"));
    }
    if let Some(course_count) = node.course_count.as_ref().and_then(display_value) {
        parts.push(format!("课程数={course_count}"));
    }
    if let Some(required_credits) = node.required_credits.as_ref().and_then(display_value) {
        parts.push(format!("要求学分={required_credits}"));
    }
    if let Some(total_credits) = node.total_credits.as_ref().and_then(display_value) {
        parts.push(format!("合计学分={total_credits}"));
    }

    format!("({})", parts.join(", "))
}

fn format_node_note_hint(node: &ehall::training_program::TrainingProgramNode) -> &'static str {
    if has_node_notes(node) {
        " [node-detail有额外信息]"
    } else {
        ""
    }
}

fn has_node_notes(node: &ehall::training_program::TrainingProgramNode) -> bool {
    has_non_empty_text(node.requirement.as_deref())
        || has_non_empty_json_text(node.extra.get("XDYQ"))
        || has_non_empty_json_text(node.extra.get("BZ"))
}

fn print_training_program_node_detail(node: &ehall::training_program::TrainingProgramNode) {
    println!("PYFADM: {}", node.program_id);
    println!("KZH: {}", node.id);
    println!("节点名称: {}", node.name);

    print_optional_line("父节点 KZH", node.parent_id.as_deref());
    print_optional_line("节点类型", node.node_type_name.as_deref());
    print_optional_line("节点类型代码", node.node_type_id.as_deref());
    print_optional_line("课程类别", node.course_category_name.as_deref());
    print_optional_line("课程类别代码", node.course_category_id.as_deref());
    print_optional_value_line("课程数", node.course_count.as_ref());
    print_optional_value_line("要求学分", node.required_credits.as_ref());
    print_optional_value_line("合计学分", node.total_credits.as_ref());
    print_optional_value_line("最低修读学分", node.extra.get("ZDXDXF"));
    print_optional_value_line("总应修门数", node.extra.get("ZSXDMS"));
    print_optional_value_line("总完成课组数", node.extra.get("ZSWCKZS"));
    print_optional_extra_line("是否共享课组", node, "SFXGXKZ_DISPLAY");
    print_optional_extra_line("是否开课", node, "SFKK_DISPLAY");
    print_optional_extra_line("课程性质", node, "KCXZDM_DISPLAY");
    print_optional_extra_line("专业方向", node, "ZYFXDM_DISPLAY");
    print_optional_value_line("排序", node.extra.get("PX"));

    if let Some(requirement) = node
        .requirement
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        println!();
        println!("修读要求:");
        println!("{requirement}");
    }
    if let Some(requirement) = node
        .extra
        .get("XDYQ")
        .and_then(display_value)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        println!();
        println!("要求说明:");
        println!("{requirement}");
    }
    if let Some(remark) = node
        .extra
        .get("BZ")
        .and_then(display_value)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        println!();
        println!("备注:");
        println!("{remark}");
    }
}

fn print_optional_line(label: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        println!("{label}: {value}");
    }
}

fn print_optional_value_line(label: &str, value: Option<&Value>) {
    if let Some(value) = value.and_then(display_value) {
        if !value.trim().is_empty() {
            println!("{label}: {value}");
        }
    }
}

fn print_optional_extra_line(
    label: &str,
    node: &ehall::training_program::TrainingProgramNode,
    key: &str,
) {
    print_optional_value_line(label, node.extra.get(key));
}

fn has_non_empty_text(value: Option<&str>) -> bool {
    value.map(str::trim).is_some_and(|value| !value.is_empty())
}

fn has_non_empty_json_text(value: Option<&Value>) -> bool {
    value
        .and_then(display_value)
        .is_some_and(|value| !value.trim().is_empty())
}

fn normalized_parent_id(parent_id: Option<&str>) -> Option<&str> {
    parent_id
        .map(str::trim)
        .filter(|parent_id| !parent_id.is_empty() && *parent_id != "-1")
}

fn display_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) if value.is_empty() => None,
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => Some(value.to_string()),
    }
}
