use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const APP_ID: &str = "4766960573884517";
const DEFAULT_ROLE_ID: &str = "20220913102717892";
const APP_SHOW_URL: &str = "https://ehall.nju.edu.cn/appShow";
const APP_INDEX_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/kcbcx/*default/index.do";
const CHANGE_ROLE_URL_PREFIX: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/funauthapp/api/changeAppRole/kcbcx";
const CURRENT_TERM_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/kcbcx/modules/bjkcb/dqxnxq.do";
const COURSE_LIST_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/kcbcx/modules/qxkcb/qxfbkccx.do";
const COURSE_LIST_ACTION: &str = "qxfbkccx";
const CURRENT_TERM_ACTION: &str = "dqxnxq";
const COURSE_ORDER: &str = "+KKDWDM,+KCH,+KXH";

/// 「本-课表查询」默认使用的本科学生组角色。
pub fn default_role_id() -> &'static str {
    DEFAULT_ROLE_ID
}

/// 使用统一认证态初始化「本-课表查询」ehallapp 会话，并设置应用角色。
pub async fn prepare_session(client: &common::Client, role_id: &str) -> Result<()> {
    client
        .get(APP_SHOW_URL)
        .query(&[("appId", APP_ID)])
        .send()
        .await
        .context("failed to open ehall course schedule app")?
        .error_for_status()
        .context("ehall course schedule app entry returned an error status")?;

    client
        .get(APP_INDEX_URL)
        .query(&[("_roleId", role_id), ("EMAP_LANG", "zh"), ("THEME", "")])
        .send()
        .await
        .context("failed to open ehall course schedule app index")?
        .error_for_status()
        .context("ehall course schedule app index returned an error status")?;

    client
        .post(format!("{CHANGE_ROLE_URL_PREFIX}/{role_id}.do"))
        .form(&[("selectedRoleId", role_id)])
        .send()
        .await
        .context("failed to change ehall course schedule app role")?
        .error_for_status()
        .context("ehall course schedule change role endpoint returned an error status")?;

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct CourseScheduleListOptions {
    pub page_number: u64,
    pub page_size: u64,
    pub term: Option<String>,
    pub filters: Vec<CourseScheduleFilter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseScheduleFilter {
    pub name: String,
    pub value: String,
    pub builder: String,
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
pub struct CurrentTerm {
    #[serde(rename = "DM")]
    pub id: String,
    #[serde(rename = "MC")]
    pub name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CourseSchedule {
    #[serde(rename = "WID")]
    pub id: String,
    #[serde(rename = "JXBID")]
    pub teaching_class_id: String,
    #[serde(rename = "KCH")]
    pub course_id: String,
    #[serde(rename = "KCM")]
    pub course_name: String,
    #[serde(rename = "KXH", default)]
    pub course_sequence: Option<String>,
    #[serde(rename = "JXBMC", default)]
    pub teaching_class_name: Option<String>,
    #[serde(rename = "SKJS", default)]
    pub teachers: Option<String>,
    #[serde(rename = "YPSJDD", default)]
    pub time_place: Option<String>,
    #[serde(rename = "SKXQ", default)]
    pub weekday: Option<String>,
    #[serde(rename = "SKJC", default)]
    pub periods: Option<String>,
    #[serde(rename = "SKZC", default)]
    pub weeks: Option<String>,
    #[serde(rename = "SKJAS", default)]
    pub classrooms: Option<String>,
    #[serde(rename = "XXXQDM", default)]
    pub campus_id: Option<String>,
    #[serde(rename = "XXXQDM_DISPLAY", default)]
    pub campus_name: Option<String>,
    #[serde(rename = "XF", default)]
    pub credits: Option<Value>,
    #[serde(rename = "XS", default)]
    pub hours: Option<Value>,
    #[serde(rename = "PKDWDM", default)]
    pub department_id: Option<String>,
    #[serde(rename = "PKDWDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "XKZRS", default)]
    pub student_count: Option<Value>,
    #[serde(rename = "XNXQDM", default)]
    pub term_id: Option<String>,
    #[serde(rename = "XNXQDM_DISPLAY", default)]
    pub term_name: Option<String>,
    #[serde(rename = "TKJG", default)]
    pub reschedule_info: Option<String>,
    #[serde(rename = "SKBJ", default)]
    pub student_majors: Option<String>,
    #[serde(rename = "TXKCLB", default)]
    pub general_course_category_id: Option<String>,
    #[serde(rename = "TXKCLB_DISPLAY", default)]
    pub general_course_category_name: Option<String>,
    #[serde(rename = "XGXKLBDM", default)]
    pub public_course_category_id: Option<String>,
    #[serde(rename = "XGXKLBDM_DISPLAY", default)]
    pub public_course_category_name: Option<String>,
    #[serde(rename = "SFTK", default)]
    pub is_suspended: Option<String>,
    #[serde(rename = "SFTK_DISPLAY", default)]
    pub is_suspended_name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct EhallEnvelope<T> {
    datas: HashMap<String, EhallPage<T>>,
    code: String,
}

pub async fn get_current_term(client: &common::Client) -> Result<CurrentTerm> {
    let page: EhallPage<CurrentTerm> =
        post_page(client, CURRENT_TERM_URL, &[], CURRENT_TERM_ACTION).await?;

    page.rows
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("current term was not found"))
}

pub async fn list_course_schedules(
    client: &common::Client,
    options: &CourseScheduleListOptions,
) -> Result<EhallPage<CourseSchedule>> {
    let page_number = options.page_number.max(1);
    let page_size = options.page_size.max(1);
    let term = match options
        .term
        .as_deref()
        .map(str::trim)
        .filter(|term| !term.is_empty())
    {
        Some(term) => term.to_string(),
        None => get_current_term(client).await?.id,
    };
    let query_setting = serde_json::to_string(&course_query_setting(&term, &options.filters))
        .context("failed to serialize course schedule query settings")?;

    let mut form = vec![
        ("CXYH".to_string(), "true".to_string()),
        ("querySetting".to_string(), query_setting),
        ("*order".to_string(), COURSE_ORDER.to_string()),
        ("pageSize".to_string(), page_size.to_string()),
        ("pageNumber".to_string(), page_number.to_string()),
    ];
    for filter in &options.filters {
        if uses_top_level_param(&filter.name) {
            form.push((filter.name.clone(), filter.value.clone()));
        }
    }

    post_page(client, COURSE_LIST_URL, &form, COURSE_LIST_ACTION).await
}

pub async fn list_all_course_schedules(
    client: &common::Client,
    options: &CourseScheduleListOptions,
) -> Result<Vec<CourseSchedule>> {
    let mut page_options = options.clone();
    page_options.page_number = 1;
    page_options.page_size = page_options.page_size.max(200);
    if page_options
        .term
        .as_deref()
        .map(str::trim)
        .is_none_or(str::is_empty)
    {
        page_options.term = Some(get_current_term(client).await?.id);
    }
    let mut courses = Vec::new();

    loop {
        let page = list_course_schedules(client, &page_options).await?;
        let total_size = page.total_size;
        courses.extend(page.rows);
        if courses.len() as u64 >= total_size || total_size == 0 {
            break;
        }
        page_options.page_number += 1;
    }

    Ok(courses)
}

pub fn infer_filter_builder(name: &str) -> &'static str {
    match name {
        "KCH" | "KCM" | "JXBMC" | "KXH" | "SKJS" | "YPSJDD" | "SKJAS" | "SKBJ" => "include",
        "PKDWDM" | "KKDWDM" | "XNXQDM" | "XGXKLBDM" | "SFTK" => "m_value_equal",
        _ => "equal",
    }
}

fn course_query_setting(term: &str, filters: &[CourseScheduleFilter]) -> Vec<Value> {
    let mut query_setting = vec![
        json!({
            "name": "XNXQDM",
            "value": term,
            "linkOpt": "and",
            "builder": "equal"
        }),
        json!([
            {
                "name": "RWZTDM",
                "value": "1",
                "linkOpt": "and",
                "builder": "equal"
            },
            {
                "name": "RWZTDM",
                "linkOpt": "or",
                "builder": "isNull"
            }
        ]),
        json!({
            "name": "CXYH",
            "value": true,
            "linkOpt": "and",
            "builder": "equal"
        }),
        json!({
            "name": "*order",
            "value": COURSE_ORDER,
            "linkOpt": "and",
            "builder": "m_value_equal"
        }),
    ];

    for filter in filters {
        if filter.name == "XNXQDM" {
            continue;
        }
        query_setting.push(json!({
            "name": filter.name,
            "value": filter.value,
            "linkOpt": "and",
            "builder": filter.builder
        }));
    }

    query_setting
}

fn uses_top_level_param(name: &str) -> bool {
    matches!(name, "KSJC" | "JSJC" | "SKZC" | "SKXQ" | "JXLDM")
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
