use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const APP_ID: &str = "4768574631264620";
const DEFAULT_ROLE_ID: &str = "20230211151103310";
const APP_SHOW_URL: &str = "https://ehall.nju.edu.cn/appShow";
const APP_INDEX_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/cjcx/*default/index.do";
const APP_INDEX_REFERER: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/cjcx/*default/index.do?EMAP_LANG=zh&THEME=";
const CHANGE_ROLE_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/funauthapp/api/changeAppRole/cjcx";
const CURRENT_TERM_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/cjcx/modules/cjcx/cxdqxnxqdm.do";
const RECENT_TERMS_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/code/1daca2c6-abd2-4278-af49-5af038db0926.do";
const GRADE_LIST_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/cjcx/modules/cjcx/cxxscjd.do";
const CET_GRADE_LIST_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/cjcx/modules/cjcx/cxsljcj.do";
const CURRENT_TERM_ACTION: &str = "cxdqxnxqdm";
const GRADE_LIST_ACTION: &str = "cxxscjd";
const CET_GRADE_LIST_ACTION: &str = "cxsljcj";
const GRADE_ORDER: &str = "-XNXQDM,+KCH,+KCXZ";
const CET_GRADE_ORDER: &str = "+YXDM,+XH,-KSSJ";
const DEFAULT_GRADE_TYPES: &str = "0,4,5,9";
pub const DEFAULT_CET_EXAM_TYPES: &[&str] = &["CET4", "CET6"];
pub const FITNESS_EXAM_TYPE: &str = "01";

/// 「成绩查询」默认使用的教务学生组角色。
pub fn default_role_id() -> &'static str {
    DEFAULT_ROLE_ID
}

/// 使用统一认证态初始化「成绩查询」ehallapp 会话，并设置应用角色。
pub async fn prepare_session(client: &common::Client, role_id: &str) -> Result<()> {
    client
        .get(APP_SHOW_URL)
        .query(&[("appId", APP_ID)])
        .send()
        .await
        .context("failed to open ehall grade query app")?
        .error_for_status()
        .context("ehall grade query app entry returned an error status")?;

    client
        .get(APP_INDEX_URL)
        .query(&[("_roleId", role_id), ("EMAP_LANG", "zh"), ("THEME", "")])
        .send()
        .await
        .context("failed to open ehall grade query app index")?
        .error_for_status()
        .context("ehall grade query app index returned an error status")?;

    client
        .get(format!("{CHANGE_ROLE_URL}/{role_id}.do"))
        .send()
        .await
        .context("failed to change ehall grade query app role")?
        .error_for_status()
        .context("ehall grade query change role endpoint returned an error status")?;

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct GradeListOptions {
    pub terms: Vec<String>,
    pub course_name: Option<String>,
    pub course_id: Option<String>,
    pub passed: Option<bool>,
    pub show_max_grade: bool,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct CetGradeListOptions {
    pub terms: Vec<String>,
    pub exam_types: Vec<String>,
    pub page_number: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct EhallPage<T> {
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    #[serde(rename = "pageNumber", default)]
    pub page_number: u64,
    #[serde(rename = "pageSize", default)]
    pub page_size: u64,
    #[serde(default)]
    pub rows: Vec<T>,
    #[serde(default)]
    pub ext_params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GradeTerm {
    #[serde(rename = "XNXQDM")]
    pub id: String,
    #[serde(rename = "XNXQDM_DISPLAY", default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl GradeTerm {
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Grade {
    #[serde(rename = "XH")]
    pub student_id: String,
    #[serde(rename = "XNXQDM")]
    pub term_id: String,
    #[serde(rename = "XNXQDM_DISPLAY", default)]
    pub term_name: Option<String>,
    #[serde(rename = "KCH")]
    pub course_id: String,
    #[serde(rename = "KCM")]
    pub course_name: String,
    #[serde(rename = "YWKCM", default)]
    pub english_course_name: Option<String>,
    #[serde(rename = "XF", default)]
    pub credits: Option<Value>,
    #[serde(rename = "KCXZDM", default)]
    pub course_nature_id: Option<String>,
    #[serde(rename = "KCXZDM_DISPLAY", default)]
    pub course_nature_name: Option<String>,
    #[serde(rename = "ZCJ", default)]
    pub total_grade: Option<Value>,
    #[serde(rename = "SFJG", default)]
    pub passed_id: Option<String>,
    #[serde(rename = "SFJG_DISPLAY", default)]
    pub passed_name: Option<String>,
    #[serde(rename = "CJBZBS", default)]
    pub transcript_mark: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CetGrade {
    #[serde(rename = "WID")]
    pub id: String,
    #[serde(rename = "XH")]
    pub student_id: String,
    #[serde(rename = "XM", default)]
    pub student_name: Option<String>,
    #[serde(rename = "XNXQDM")]
    pub term_id: String,
    #[serde(rename = "XNXQDM_DISPLAY", default)]
    pub term_name: Option<String>,
    #[serde(rename = "KSXM")]
    pub exam_type_id: String,
    #[serde(rename = "KSXM_DISPLAY")]
    pub exam_type_name: String,
    #[serde(rename = "ZCJ", default)]
    pub score: Option<Value>,
    #[serde(rename = "SFTG", default)]
    pub passed_name: Option<String>,
    #[serde(rename = "KSSJ", default)]
    pub exam_date: Option<String>,
    #[serde(rename = "YXDM", default)]
    pub department_id: Option<String>,
    #[serde(rename = "YXDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "DRSJ", default)]
    pub imported_at: Option<String>,
    #[serde(rename = "DRR", default)]
    pub importer_id: Option<String>,
    #[serde(rename = "DRRXM", default)]
    pub importer_name: Option<String>,
    #[serde(rename = "BZ", default)]
    pub remark: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct EhallEnvelope<T> {
    datas: HashMap<String, EhallPage<T>>,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct EhallRowsEnvelope<T> {
    datas: HashMap<String, EhallRows<T>>,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct EhallRows<T> {
    #[serde(default)]
    rows: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct CodeGradeTerm {
    id: String,
    name: Option<String>,
}

impl From<CodeGradeTerm> for GradeTerm {
    fn from(term: CodeGradeTerm) -> Self {
        Self {
            id: term.id,
            name: term.name,
            extra: Map::new(),
        }
    }
}

pub async fn get_current_term(client: &common::Client) -> Result<GradeTerm> {
    let page: EhallPage<GradeTerm> =
        post_page(client, CURRENT_TERM_URL, &[], CURRENT_TERM_ACTION).await?;

    page.rows
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("current grade term was not found"))
}

pub async fn list_recent_terms(client: &common::Client) -> Result<Vec<GradeTerm>> {
    let rows: EhallRows<CodeGradeTerm> = get_rows(client, RECENT_TERMS_URL, "code").await?;

    Ok(rows.rows.into_iter().map(GradeTerm::from).collect())
}

pub async fn list_grades(
    client: &common::Client,
    options: &GradeListOptions,
) -> Result<EhallPage<Grade>> {
    let page_number = options.page_number.max(1);
    let page_size = options.page_size.max(1);
    let terms = resolve_terms(client, &options.terms).await?;
    let query_setting = serde_json::to_string(&grade_query_setting(options, &terms))
        .context("failed to serialize grade query settings")?;
    let form = vec![
        ("querySetting".to_string(), query_setting),
        ("*order".to_string(), GRADE_ORDER.to_string()),
        ("pageSize".to_string(), page_size.to_string()),
        ("pageNumber".to_string(), page_number.to_string()),
    ];

    post_page(client, GRADE_LIST_URL, &form, GRADE_LIST_ACTION).await
}

pub async fn list_cet_grades(
    client: &common::Client,
    options: &CetGradeListOptions,
) -> Result<EhallPage<CetGrade>> {
    let page_number = options.page_number.max(1);
    let page_size = options.page_size.max(1);
    let query_setting = serde_json::to_string(&cet_grade_query_setting(options))
        .context("failed to serialize CET grade query settings")?;
    let mut form = vec![
        ("*order".to_string(), CET_GRADE_ORDER.to_string()),
        ("pageSize".to_string(), page_size.to_string()),
        ("pageNumber".to_string(), page_number.to_string()),
    ];
    if query_setting != "[]" {
        form.push(("querySetting".to_string(), query_setting));
    }

    post_page(client, CET_GRADE_LIST_URL, &form, CET_GRADE_LIST_ACTION).await
}

pub async fn list_all_grades(
    client: &common::Client,
    options: &GradeListOptions,
) -> Result<Vec<Grade>> {
    let mut page_options = options.clone();
    page_options.page_number = 1;
    page_options.page_size = page_options.page_size.max(100);
    let mut grades = Vec::new();

    loop {
        let page = list_grades(client, &page_options).await?;
        let total_size = page.total_size;
        grades.extend(page.rows);
        if grades.len() as u64 >= total_size || total_size == 0 {
            break;
        }
        page_options.page_number += 1;
    }

    Ok(grades)
}

pub async fn list_all_cet_grades(
    client: &common::Client,
    options: &CetGradeListOptions,
) -> Result<Vec<CetGrade>> {
    let mut page_options = options.clone();
    page_options.page_number = 1;
    page_options.page_size = page_options.page_size.max(100);
    let mut grades = Vec::new();

    loop {
        let page = list_cet_grades(client, &page_options).await?;
        let total_size = page.total_size;
        grades.extend(page.rows);
        if grades.len() as u64 >= total_size || total_size == 0 {
            break;
        }
        page_options.page_number += 1;
    }

    Ok(grades)
}

async fn resolve_terms(client: &common::Client, terms: &[String]) -> Result<Vec<String>> {
    let terms = terms
        .iter()
        .map(|term| term.trim())
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if !terms.is_empty() {
        return Ok(terms);
    }

    let recent_terms = list_recent_terms(client).await?;
    let terms = recent_terms
        .into_iter()
        .map(|term| term.id)
        .filter(|term| !term.trim().is_empty())
        .collect::<Vec<_>>();

    if terms.is_empty() {
        Ok(vec![get_current_term(client).await?.id])
    } else {
        Ok(terms)
    }
}

fn grade_query_setting(options: &GradeListOptions, terms: &[String]) -> Vec<Value> {
    let mut settings = vec![
        json!({
            "name": "SHOWMAXCJ",
            "caption": "显示最高成绩",
            "linkOpt": "AND",
            "builderList": "cbl_m_List",
            "builder": "m_value_equal",
            "value": if options.show_max_grade { "1" } else { "0" },
            "value_display": if options.show_max_grade { "是" } else { "否" },
        }),
        json!({
            "name": "CJXZ",
            "linkOpt": "AND",
            "builder": "m_value_equal",
            "value": DEFAULT_GRADE_TYPES,
        }),
    ];

    if !terms.is_empty() {
        settings.push(json!({
            "name": "XNXQDM",
            "value": terms.join(","),
            "linkOpt": "and",
            "builder": "m_value_equal",
        }));
    }
    if let Some(course_name) = options
        .course_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        settings.push(json!({
            "name": "KCM",
            "value": course_name,
            "linkOpt": "and",
            "builder": "include",
        }));
    }
    if let Some(course_id) = options
        .course_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        settings.push(json!({
            "name": "KCH",
            "value": course_id,
            "linkOpt": "and",
            "builder": "include",
        }));
    }
    if let Some(passed) = options.passed {
        settings.push(json!({
            "name": "SFJG",
            "value": if passed { "1" } else { "0" },
            "linkOpt": "and",
            "builder": "equal",
        }));
    }

    settings
}

fn cet_grade_query_setting(options: &CetGradeListOptions) -> Vec<Value> {
    let mut settings = Vec::new();
    let terms = options
        .terms
        .iter()
        .map(|term| term.trim())
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    let exam_types = options
        .exam_types
        .iter()
        .map(|exam_type| exam_type.trim())
        .filter(|exam_type| !exam_type.is_empty())
        .collect::<Vec<_>>();

    if !terms.is_empty() {
        settings.push(json!({
            "name": "XNXQDM",
            "value": terms.join(","),
            "linkOpt": "and",
            "builder": "m_value_equal",
        }));
    }

    let exam_type_value = if !exam_types.is_empty() {
        exam_types.join(",")
    } else {
        DEFAULT_CET_EXAM_TYPES.join(",")
    };
    settings.push(json!({
        "name": "KSXM",
        "value": exam_type_value,
        "linkOpt": "and",
        "builder": "m_value_equal",
    }));

    settings
}

async fn get_rows<T>(client: &common::Client, url: &str, action: &str) -> Result<EhallRows<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let cache_bust = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is earlier than UNIX epoch")?
        .as_millis()
        .to_string();
    let body = client
        .get(url)
        .query(&[("_", cache_bust.as_str())])
        .header("Accept", "application/json, text/javascript, */*; q=0.01")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Referer", APP_INDEX_REFERER)
        .send()
        .await
        .with_context(|| format!("failed to request {url}"))?
        .error_for_status()
        .with_context(|| format!("{url} returned an error status"))?
        .text()
        .await
        .with_context(|| format!("failed to read {url} response body"))?;
    let envelope: EhallRowsEnvelope<T> = serde_json::from_str(&body).with_context(|| {
        format!(
            "failed to parse {url} response as JSON: response starts with {:?}",
            body.chars().take(200).collect::<String>()
        )
    })?;

    if envelope.code != "0" {
        return Err(anyhow!("{url} returned application code {}", envelope.code));
    }

    envelope
        .datas
        .into_iter()
        .find_map(|(key, rows)| (key == action).then_some(rows))
        .ok_or_else(|| anyhow!("{url} response did not contain data action {action}"))
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
