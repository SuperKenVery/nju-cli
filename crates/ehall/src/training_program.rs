use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const APP_ID: &str = "4766860087431764";
const DEFAULT_ROLE_ID: &str = "20220913102717892";
const APP_SHOW_URL: &str = "https://ehall.nju.edu.cn/appShow";
const APP_INDEX_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/qxfacx/*default/index.do";
const CHANGE_ROLE_URL_PREFIX: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/funauthapp/api/changeAppRole/qxfacx";
const PROGRAM_LIST_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/qxfacx/modules/pyfacxepg/qxpyfacxzl.do";
const PROGRAM_DETAIL_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/jwpubapp/modules/pyfa/cxqxpyfandcxwqx.do";
const PROGRAM_NODES_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/jwpubapp/modules/pyfa/kzcx.do";
const NODE_COURSES_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/jwpubapp/modules/pyfa/kzkccxnddz.do";
const LIST_ACTION: &str = "qxpyfacxzl";
const DETAIL_ACTION: &str = "cxqxpyfandcxwqx";
const NODES_ACTION: &str = "kzcx";
const COURSES_ACTION: &str = "kzkccxnddz";
const PROGRAM_ORDER: &str = "-NJDM,+DWDM,+ZYDM";
const COURSE_ORDER: &str = "+KCH";

/// 「本-培养方案查询」默认使用的本科学生组角色。
pub fn default_role_id() -> &'static str {
    DEFAULT_ROLE_ID
}

/// 使用统一认证态初始化 ehall 和 ehallapp 会话，并设置应用角色。
///
/// 调用方需要传入已经带有 authserver `CASTGC` cookie 的 reqwest client。
///
/// 普通 CAS 登录跳转可以靠 reqwest 自动 follow redirect 完成，但「本-培养方案查询」
/// 有多角色选择页，单独访问 `appShow` 会停在 `select_role.html`，直接访问接口会 302；
/// 直接访问应用 index 也会缺少应用授权而 403。因此这里先访问应用入口和带 `_roleId`
/// 的应用页建立 ehall/ehallapp 会话，再调用 `changeAppRole` 选择本科学生组角色。
pub async fn prepare_session(client: &common::Client, role_id: &str) -> Result<()> {
    client
        .get(APP_SHOW_URL)
        .query(&[("appId", APP_ID)])
        .send()
        .await
        .context("failed to open ehall training program app")?
        .error_for_status()
        .context("ehall training program app entry returned an error status")?;

    client
        .get(APP_INDEX_URL)
        .query(&[("_roleId", role_id), ("EMAP_LANG", "zh"), ("THEME", "")])
        .send()
        .await
        .context("failed to open ehall training program app index")?
        .error_for_status()
        .context("ehall training program app index returned an error status")?;

    client
        .post(format!("{CHANGE_ROLE_URL_PREFIX}/{role_id}.do"))
        .form(&[("selectedRoleId", role_id)])
        .send()
        .await
        .context("failed to change ehall training program app role")?
        .error_for_status()
        .context("ehall training program change role endpoint returned an error status")?;

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct TrainingProgramListOptions {
    pub page_number: u64,
    pub page_size: u64,
    pub name: Option<String>,
    pub grade: Option<String>,
    pub department: Option<String>,
    pub study_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EhallPage<T> {
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    #[serde(rename = "pageNumber", default)]
    pub page_number: u64,
    #[serde(rename = "pageSize", default)]
    pub page_size: u64,
    pub rows: Vec<T>,
    #[serde(default)]
    pub ext_params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TrainingProgram {
    #[serde(rename = "PYFADM")]
    pub id: String,
    #[serde(rename = "PYFAMC")]
    pub name: String,
    #[serde(rename = "NJDM", default)]
    pub grade: Option<String>,
    #[serde(rename = "NJDM_DISPLAY", default)]
    pub grade_display: Option<String>,
    #[serde(rename = "DWDM", default)]
    pub department_id: Option<String>,
    #[serde(rename = "DWDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "ZYDM", default)]
    pub major_id: Option<String>,
    #[serde(rename = "ZYDM_DISPLAY", default)]
    pub major_name: Option<String>,
    #[serde(rename = "ZYFXDM", default)]
    pub major_direction_id: Option<String>,
    #[serde(rename = "ZYFXDM_DISPLAY", default)]
    pub major_direction_name: Option<String>,
    #[serde(rename = "XDLXDM", default)]
    pub study_type_id: Option<String>,
    #[serde(rename = "XDLXDM_DISPLAY", default)]
    pub study_type_name: Option<String>,
    #[serde(rename = "PYCCDM_DISPLAY", default)]
    pub level_name: Option<String>,
    #[serde(rename = "FATS", default)]
    pub feature: Option<String>,
    #[serde(rename = "PYMB", default)]
    pub objective: Option<String>,
    #[serde(rename = "XDYQ", default)]
    pub requirement: Option<String>,
    #[serde(rename = "ZGKC", default)]
    pub graduation_requirement: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TrainingProgramNode {
    #[serde(rename = "PYFADM")]
    pub program_id: String,
    #[serde(rename = "KZH")]
    pub id: String,
    #[serde(rename = "FKZH", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "KZM")]
    pub name: String,
    #[serde(rename = "KZLXDM", default)]
    pub node_type_id: Option<String>,
    #[serde(rename = "KZLXDM_DISPLAY", default)]
    pub node_type_name: Option<String>,
    #[serde(rename = "KCLBDM", default)]
    pub course_category_id: Option<String>,
    #[serde(rename = "KCLBDM_DISPLAY", default)]
    pub course_category_name: Option<String>,
    #[serde(rename = "KCZXF", default)]
    pub total_credits: Option<Value>,
    #[serde(rename = "ZSXDXF", default)]
    pub required_credits: Option<Value>,
    #[serde(rename = "KCZMS", default)]
    pub course_count: Option<Value>,
    #[serde(rename = "XDYQC", default)]
    pub requirement: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TrainingProgramCourse {
    #[serde(rename = "PYFADM")]
    pub program_id: String,
    #[serde(rename = "PYFAMC", default)]
    pub program_name: Option<String>,
    #[serde(rename = "KZH")]
    pub node_id: String,
    #[serde(rename = "KZM", default)]
    pub node_name: Option<String>,
    #[serde(rename = "KCH")]
    pub course_id: String,
    #[serde(rename = "KCM")]
    pub course_name: String,
    #[serde(rename = "XF", default)]
    pub credits: Option<String>,
    #[serde(rename = "XS", default)]
    pub hours: Option<String>,
    #[serde(rename = "XNXQ", default)]
    pub term: Option<String>,
    #[serde(rename = "XDXQ", default)]
    pub relative_term: Option<String>,
    #[serde(rename = "KKDWDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "KCLBDM_DISPLAY", default)]
    pub course_category_name: Option<String>,
    #[serde(rename = "KCXZDM_DISPLAY", default)]
    pub course_nature_name: Option<String>,
    #[serde(rename = "SFKK_DISPLAY", default)]
    pub is_open_name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct EhallEnvelope<T> {
    datas: HashMap<String, EhallPage<T>>,
    code: String,
}

pub async fn list_training_programs(
    client: &common::Client,
    options: &TrainingProgramListOptions,
) -> Result<EhallPage<TrainingProgram>> {
    let page_number = options.page_number.max(1);
    let page_size = options.page_size.max(1);
    let query_setting = serde_json::to_string(&program_query_setting(options))
        .context("failed to serialize training program query settings")?;

    let form = vec![
        ("querySetting".to_string(), query_setting),
        ("*order".to_string(), PROGRAM_ORDER.to_string()),
        ("pageSize".to_string(), page_size.to_string()),
        ("pageNumber".to_string(), page_number.to_string()),
    ];

    post_page(client, PROGRAM_LIST_URL, &form, LIST_ACTION).await
}

pub async fn list_all_training_programs(
    client: &common::Client,
    options: &TrainingProgramListOptions,
) -> Result<Vec<TrainingProgram>> {
    let mut page_options = options.clone();
    page_options.page_number = 1;
    page_options.page_size = page_options.page_size.max(200);
    let mut programs = Vec::new();

    loop {
        let page = list_training_programs(client, &page_options).await?;
        let total_size = page.total_size;
        programs.extend(page.rows);
        if programs.len() as u64 >= total_size || total_size == 0 {
            break;
        }
        page_options.page_number += 1;
    }

    Ok(programs)
}

pub async fn get_training_program_detail(
    client: &common::Client,
    program_id: &str,
) -> Result<TrainingProgram> {
    let form = vec![
        ("PYFADM".to_string(), program_id.to_string()),
        ("pageSize".to_string(), "1".to_string()),
        ("pageNumber".to_string(), "1".to_string()),
    ];
    let page: EhallPage<TrainingProgram> =
        post_page(client, PROGRAM_DETAIL_URL, &form, DETAIL_ACTION).await?;

    page.rows
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("training program {program_id} was not found"))
}

pub async fn list_training_program_nodes(
    client: &common::Client,
    program_id: &str,
) -> Result<Vec<TrainingProgramNode>> {
    let form = vec![("PYFADM".to_string(), program_id.to_string())];
    let page: EhallPage<TrainingProgramNode> =
        post_page(client, PROGRAM_NODES_URL, &form, NODES_ACTION).await?;

    Ok(page.rows)
}

pub async fn get_training_program_node_detail(
    client: &common::Client,
    program_id: &str,
    node_id: &str,
) -> Result<TrainingProgramNode> {
    let form = vec![
        ("PYFADM".to_string(), program_id.to_string()),
        ("KZH".to_string(), node_id.to_string()),
        ("pageSize".to_string(), "1".to_string()),
        ("pageNumber".to_string(), "1".to_string()),
    ];
    let page: EhallPage<TrainingProgramNode> =
        post_page(client, PROGRAM_NODES_URL, &form, NODES_ACTION).await?;

    page.rows.into_iter().next().ok_or_else(|| {
        anyhow!("training program node {node_id} was not found in program {program_id}")
    })
}

pub async fn list_node_courses(
    client: &common::Client,
    program_id: &str,
    node_id: &str,
    page_number: u64,
    page_size: u64,
) -> Result<EhallPage<TrainingProgramCourse>> {
    let form = vec![
        ("PYFADM".to_string(), program_id.to_string()),
        ("KZH".to_string(), node_id.to_string()),
        ("*order".to_string(), COURSE_ORDER.to_string()),
        ("pageSize".to_string(), page_size.max(1).to_string()),
        ("pageNumber".to_string(), page_number.max(1).to_string()),
    ];

    post_page(client, NODE_COURSES_URL, &form, COURSES_ACTION).await
}

pub async fn list_all_node_courses(
    client: &common::Client,
    program_id: &str,
    node_id: &str,
) -> Result<Vec<TrainingProgramCourse>> {
    let mut page_number = 1;
    let page_size = 200;
    let mut courses = Vec::new();

    loop {
        let page = list_node_courses(client, program_id, node_id, page_number, page_size).await?;
        let total_size = page.total_size;
        courses.extend(page.rows);
        if courses.len() as u64 >= total_size || total_size == 0 {
            break;
        }
        page_number += 1;
    }

    Ok(courses)
}

fn program_query_setting(options: &TrainingProgramListOptions) -> Vec<Value> {
    let mut query_setting = Vec::new();

    if let Some(name) = non_empty(options.name.as_deref()) {
        query_setting.push(json!({
            "name": "PYFAMC",
            "value": name,
            "linkOpt": "and",
            "builder": "include"
        }));
    }
    if let Some(grade) = non_empty(options.grade.as_deref()) {
        query_setting.push(json!({
            "name": "NJDM",
            "value": grade,
            "linkOpt": "and",
            "builder": "equal"
        }));
    }
    if let Some(department) = non_empty(options.department.as_deref()) {
        query_setting.push(json!({
            "name": "DWDM",
            "value": department,
            "linkOpt": "and",
            "builder": "equal"
        }));
    }
    if let Some(study_type) = non_empty(options.study_type.as_deref()) {
        query_setting.push(json!({
            "name": "XDLXDM",
            "value": study_type,
            "linkOpt": "and",
            "builder": "equal"
        }));
    }

    query_setting.push(json!({
        "name": "SFXYSHFA",
        "value": "0",
        "linkOpt": "and",
        "builder": "equal"
    }));
    query_setting.push(json!({
        "name": "FAZTDM",
        "value": "99",
        "linkOpt": "and",
        "builder": "equal"
    }));

    query_setting
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

async fn post_page<T>(
    client: &common::Client,
    url: &str,
    form: &[(String, String)],
    action: &str,
) -> Result<EhallPage<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let envelope: EhallEnvelope<T> = client
        .post(url)
        .form(form)
        .send()
        .await
        .with_context(|| format!("failed to request {url}"))?
        .error_for_status()
        .with_context(|| format!("{url} returned an error status"))?
        .json()
        .await
        .with_context(|| format!("failed to parse {url} response as JSON"))?;

    if envelope.code != "0" {
        return Err(anyhow!("{url} returned application code {}", envelope.code));
    }

    envelope
        .datas
        .into_iter()
        .find_map(|(key, page)| (key == action).then_some(page))
        .ok_or_else(|| anyhow!("{url} response did not contain data action {action}"))
}
