use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const SCHEDULE_APP_ID: &str = "4770397878132218";
const EXEMPTION_APP_ID: &str = "4792812354642924";
const DEFAULT_ROLE_ID: &str = "20230211151103310";

const APP_SHOW_URL: &str = "https://ehall.nju.edu.cn/appShow";
const SCHEDULE_INDEX_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/*default/index.do";
const EXEMPTION_INDEX_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/*default/index.do";
const CHANGE_ROLE_URL_PREFIX: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/funauthapp/api/changeAppRole";

const TERM_LIST_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/xnxqcx.do";
const CURRENT_TERM_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/jshkcb/dqxnxq.do";
const STUDENT_INFO_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxxsjbxx.do";
const SCHEDULE_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxxskclb.do";
const COURSE_INFO_URL: &str = "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxkcxx.do";
const COURSE_TEXTBOOK_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/modules/xskcb/cxkczjjcxx.do";
const EXAM_NOTES_HTML_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/wdkb/*default/public/commonpage/kblb/kblbIndexPage.html";
const EXEMPTION_LIST_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/modules/mtsq/cxmtsq.do";
const EXEMPTION_COURSE_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/modules/mtsq/cxxkkc.do";
const EXEMPTION_ALLOWED_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/MtsqController/cxkcsfksq.do";
const EXEMPTION_SETTING_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/modules/mtsq/cxxtcs.do";
const SPECIAL_STUDENT_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/modules/mtsq/xscxtsmd.do";
const FLOW_NEXT_STATUS_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/stateapp/*default/showprocess/T_GG_ZTJ_LCSL_QUERY.do";
const EXEMPTION_APPLY_URL: &str =
    "https://ehallapp.nju.edu.cn/jwapp/sys/mtxkbl/modules/mtsq/sqmtkc.do";

const TERM_LIST_ACTION: &str = "xnxqcx";
const CURRENT_TERM_ACTION: &str = "dqxnxq";
const STUDENT_INFO_ACTION: &str = "cxxsjbxx";
const SCHEDULE_ACTION: &str = "cxxskclb";
const COURSE_INFO_ACTION: &str = "cxkcxx";
const COURSE_TEXTBOOK_ACTION: &str = "cxkczjjcxx";
const EXEMPTION_LIST_ACTION: &str = "cxmtsq";
const EXEMPTION_COURSE_ACTION: &str = "cxxkkc";
const EXEMPTION_SETTING_ACTION: &str = "cxxtcs";
const SPECIAL_STUDENT_ACTION: &str = "xscxtsmd";
const FLOW_NEXT_STATUS_ACTION: &str = "T_GG_ZTJ_LCSL_QUERY";
const EXEMPTION_APPLY_ACTION: &str = "sqmtkc";

pub fn default_role_id() -> &'static str {
    DEFAULT_ROLE_ID
}

pub async fn prepare_schedule_session(client: &common::Client, role_id: &str) -> Result<()> {
    prepare_session(client, SCHEDULE_APP_ID, SCHEDULE_INDEX_URL, "wdkb", role_id).await
}

pub async fn prepare_exemption_session(client: &common::Client, role_id: &str) -> Result<()> {
    prepare_session(
        client,
        EXEMPTION_APP_ID,
        EXEMPTION_INDEX_URL,
        "mtxkbl",
        role_id,
    )
    .await
}

async fn prepare_session(
    client: &common::Client,
    app_id: &str,
    index_url: &str,
    app_name: &str,
    role_id: &str,
) -> Result<()> {
    client
        .get(APP_SHOW_URL)
        .query(&[("appId", app_id)])
        .send()
        .await
        .with_context(|| format!("failed to open ehall app {app_id}"))?
        .error_for_status()
        .with_context(|| format!("ehall app {app_id} entry returned an error status"))?;

    client
        .get(index_url)
        .query(&[("_roleId", role_id), ("EMAP_LANG", "zh"), ("THEME", "")])
        .send()
        .await
        .with_context(|| format!("failed to open ehall app {app_id} index"))?
        .error_for_status()
        .with_context(|| format!("ehall app {app_id} index returned an error status"))?;

    client
        .get(format!("{CHANGE_ROLE_URL_PREFIX}/{app_name}/{role_id}.do"))
        .send()
        .await
        .with_context(|| format!("failed to change ehall app {app_id} role"))?
        .error_for_status()
        .with_context(|| format!("ehall app {app_id} change role returned an error status"))?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct EhallPage<T> {
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    #[serde(rename = "pageNumber", default)]
    pub page_number: i64,
    #[serde(rename = "pageSize", default)]
    pub page_size: i64,
    #[serde(default)]
    pub rows: Vec<T>,
    #[serde(default)]
    pub ext_params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Term {
    #[serde(rename = "DM")]
    pub id: String,
    #[serde(rename = "MC")]
    pub name: String,
    #[serde(rename = "XNDM", default)]
    pub academic_year: Option<String>,
    #[serde(rename = "XQDM", default)]
    pub term_code: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StudentInfo {
    #[serde(rename = "XH")]
    pub student_id: String,
    #[serde(rename = "XM", default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MyCourse {
    #[serde(rename = "KCH")]
    pub course_id: String,
    #[serde(rename = "JXBID")]
    pub teaching_class_id: String,
    #[serde(rename = "JXBMC")]
    pub teaching_class_name: String,
    #[serde(rename = "KCM", default)]
    pub course_name: Option<String>,
    #[serde(rename = "SKJS", default)]
    pub teachers: Option<String>,
    #[serde(rename = "ZCXQJCDD", default)]
    pub time_place: Option<String>,
    #[serde(rename = "PKDWDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "XF", default)]
    pub credits: Option<Value>,
    #[serde(rename = "XKLY_DISPLAY", default)]
    pub selection_type_name: Option<String>,
    #[serde(rename = "SKSM", default)]
    pub class_note: Option<String>,
    #[serde(rename = "TKJG", default)]
    pub reschedule_info: Option<String>,
    #[serde(rename = "QMKSXX", default)]
    pub final_exam_info: Option<String>,
    #[serde(rename = "QTXX", default)]
    pub other_info: Option<String>,
    #[serde(rename = "JSHS", default)]
    pub teacher_contact: Option<String>,
    #[serde(rename = "XNXQDM", default)]
    pub term_id: Option<String>,
    #[serde(rename = "KXH", default)]
    pub course_sequence: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl MyCourse {
    pub fn display_course_name(&self) -> &str {
        self.course_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&self.teaching_class_name)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CourseDetail {
    pub schedule: MyCourse,
    pub course_info: Option<Value>,
    pub textbook_info: Option<Value>,
    pub selected_course_info: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ExemptionApplication {
    #[serde(rename = "KCH")]
    pub course_id: String,
    #[serde(rename = "KCM", default)]
    pub course_name: Option<String>,
    #[serde(rename = "JXBID", default)]
    pub teaching_class_id: Option<String>,
    #[serde(rename = "SKJS", default)]
    pub teachers: Option<String>,
    #[serde(rename = "YPSJDD", default)]
    pub time_place: Option<String>,
    #[serde(rename = "XNXQDM", default)]
    pub term_id: Option<String>,
    #[serde(rename = "SQYY", default)]
    pub reason: Option<String>,
    #[serde(rename = "ZTDM_DISPLAY", default)]
    pub status_name: Option<String>,
    #[serde(rename = "SHYJ", default)]
    pub review_comment: Option<String>,
    #[serde(rename = "SQSJ", default)]
    pub applied_at: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ApplyExemptionResult {
    pub application_id: Option<String>,
    pub response: Value,
}

#[derive(Debug, Deserialize)]
struct EhallEnvelope<T> {
    datas: HashMap<String, EhallPage<T>>,
    code: String,
}

pub async fn list_terms(client: &common::Client) -> Result<Vec<Term>> {
    let page: EhallPage<Term> = post_page(
        client,
        TERM_LIST_URL,
        &[("*order".to_string(), "-DM".to_string())],
        TERM_LIST_ACTION,
    )
    .await?;

    Ok(page.rows)
}

pub async fn get_current_term(client: &common::Client) -> Result<Term> {
    let page: EhallPage<Term> =
        post_page(client, CURRENT_TERM_URL, &[], CURRENT_TERM_ACTION).await?;

    page.rows
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("current term was not found"))
}

pub async fn get_student_info(client: &common::Client) -> Result<StudentInfo> {
    let page: EhallPage<StudentInfo> =
        post_page(client, STUDENT_INFO_URL, &[], STUDENT_INFO_ACTION).await?;

    page.rows
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("student info was not found"))
}

#[derive(Debug, Clone, Default)]
pub struct MyCourseListOptions {
    pub term: Option<String>,
    pub page_number: u64,
    pub page_size: u64,
}

pub async fn list_courses(
    client: &common::Client,
    options: &MyCourseListOptions,
) -> Result<EhallPage<MyCourse>> {
    let term = resolve_term(client, options.term.as_deref()).await?;
    let query_setting = serde_json::to_string(&term_query_setting(&term))
        .context("failed to serialize my course schedule query settings")?;
    let form = vec![
        ("XNXQDM".to_string(), term),
        ("querySetting".to_string(), query_setting),
        ("pageSize".to_string(), options.page_size.max(1).to_string()),
        (
            "pageNumber".to_string(),
            options.page_number.max(1).to_string(),
        ),
    ];

    post_page(client, SCHEDULE_URL, &form, SCHEDULE_ACTION).await
}

pub async fn list_all_courses(
    client: &common::Client,
    options: &MyCourseListOptions,
) -> Result<Vec<MyCourse>> {
    let mut options = options.clone();
    options.page_number = 1;
    options.page_size = options.page_size.max(100);
    let mut courses = Vec::new();

    loop {
        let page = list_courses(client, &options).await?;
        let total_size = page.total_size;
        courses.extend(page.rows);
        if total_size <= 0 || courses.len() as i64 >= total_size {
            break;
        }
        options.page_number += 1;
    }

    Ok(courses)
}

pub async fn get_course_detail(
    client: &common::Client,
    term: Option<&str>,
    identifier: &str,
) -> Result<CourseDetail> {
    let courses = list_all_courses(
        client,
        &MyCourseListOptions {
            term: term.map(ToOwned::to_owned),
            page_number: 1,
            page_size: 100,
        },
    )
    .await?;
    let schedule = courses
        .into_iter()
        .find(|course| course.course_id == identifier || course.teaching_class_id == identifier)
        .ok_or_else(|| anyhow!("course {identifier} was not found in my schedule"))?;
    let student = get_student_info(client).await.ok();
    let course_info = post_raw_action(
        client,
        COURSE_INFO_URL,
        &[("KCH".to_string(), schedule.course_id.clone())],
        COURSE_INFO_ACTION,
    )
    .await
    .ok();
    let textbook_info = post_raw_action(
        client,
        COURSE_TEXTBOOK_URL,
        &[("KCH".to_string(), schedule.course_id.clone())],
        COURSE_TEXTBOOK_ACTION,
    )
    .await
    .ok();
    let selected_course_info = match (student, schedule.term_id.as_deref()) {
        (Some(student), Some(term_id)) => post_raw_action(
            client,
            EXEMPTION_COURSE_URL,
            &[
                ("JXBID".to_string(), schedule.teaching_class_id.clone()),
                ("XH".to_string(), student.student_id),
                ("XNXQDM".to_string(), term_id.to_string()),
            ],
            EXEMPTION_COURSE_ACTION,
        )
        .await
        .ok(),
        _ => None,
    };

    Ok(CourseDetail {
        schedule,
        course_info,
        textbook_info,
        selected_course_info,
    })
}

pub async fn get_exam_notes(client: &common::Client) -> Result<String> {
    let html = client
        .get(EXAM_NOTES_HTML_URL)
        .send()
        .await
        .context("failed to request exam notes HTML")?
        .error_for_status()
        .context("exam notes HTML returned an error status")?
        .text()
        .await
        .context("failed to read exam notes HTML")?;

    extract_kssm_text(&html).ok_or_else(|| anyhow!("kssm-container was not found"))
}

pub async fn list_exemption_applications(
    client: &common::Client,
    term: Option<&str>,
) -> Result<Vec<ExemptionApplication>> {
    let student = get_student_info(client).await?;
    let mut query_setting = vec![json!({
        "name": "XH",
        "value": student.student_id,
        "linkOpt": "and",
        "builder": "equal"
    })];
    if let Some(term) = term.map(str::trim).filter(|term| !term.is_empty()) {
        query_setting.push(json!({
            "name": "XNXQDM",
            "value": term,
            "linkOpt": "and",
            "builder": "equal"
        }));
    }
    let form = vec![
        (
            "querySetting".to_string(),
            serde_json::to_string(&query_setting)
                .context("failed to serialize exemption query settings")?,
        ),
        ("*order".to_string(), "-SQSJ".to_string()),
        ("pageSize".to_string(), "999".to_string()),
        ("pageNumber".to_string(), "1".to_string()),
    ];
    let page: EhallPage<ExemptionApplication> =
        post_page(client, EXEMPTION_LIST_URL, &form, EXEMPTION_LIST_ACTION).await?;

    Ok(page.rows)
}

pub async fn apply_exemption(
    client: &common::Client,
    term: Option<&str>,
    identifier: &str,
    reason: &str,
) -> Result<ApplyExemptionResult> {
    let student = get_student_info(client).await?;
    let detail = get_course_detail(client, term, identifier).await?;
    let term_id = detail.schedule.term_id.as_deref().ok_or_else(|| {
        anyhow!(
            "course {} did not include XNXQDM",
            detail.schedule.course_id
        )
    })?;
    let selected_course = list_selected_course_for_exemption(
        client,
        &detail.schedule.teaching_class_id,
        &student.student_id,
        term_id,
    )
    .await?;

    ensure_course_can_apply(client, &detail.schedule.course_id).await?;

    let workflow_instance = current_workflow_instance(client, &student.student_id, term_id).await?;
    let next_status = first_workflow_status(client, &workflow_instance).await?;
    let payload = json!([{
        "KCH": detail.schedule.course_id,
        "KXH": selected_course.get("KXH").and_then(Value::as_str).unwrap_or_default(),
        "XNXQDM": term_id,
        "JXBID": detail.schedule.teaching_class_id,
        "ZTDM": next_status,
        "SQYY": reason,
    }]);
    let response = post_raw_action(
        client,
        EXEMPTION_APPLY_URL,
        &[("param".to_string(), payload.to_string())],
        EXEMPTION_APPLY_ACTION,
    )
    .await?;
    let application_id = response
        .get("extParams")
        .and_then(|value| value.get("msg"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    Ok(ApplyExemptionResult {
        application_id,
        response,
    })
}

async fn list_selected_course_for_exemption(
    client: &common::Client,
    teaching_class_id: &str,
    student_id: &str,
    term: &str,
) -> Result<Value> {
    let page = post_raw_action(
        client,
        EXEMPTION_COURSE_URL,
        &[
            ("JXBID".to_string(), teaching_class_id.to_string()),
            ("XH".to_string(), student_id.to_string()),
            ("XNXQDM".to_string(), term.to_string()),
        ],
        EXEMPTION_COURSE_ACTION,
    )
    .await?;

    page.get("rows")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .cloned()
        .ok_or_else(|| anyhow!("selected course {teaching_class_id} was not found"))
}

async fn ensure_course_can_apply(client: &common::Client, course_id: &str) -> Result<()> {
    let response: Value = client
        .post(EXEMPTION_ALLOWED_URL)
        .form(&[("KCH", course_id)])
        .send()
        .await
        .context("failed to request exemption eligibility")?
        .error_for_status()
        .context("exemption eligibility endpoint returned an error status")?
        .json()
        .await
        .context("failed to parse exemption eligibility response")?;
    let allowed = response
        .get("data")
        .and_then(|data| data.get("code"))
        .and_then(Value::as_str)
        .is_some_and(|code| code == "0");

    if allowed {
        Ok(())
    } else {
        let message = response
            .get("data")
            .and_then(|data| data.get("msg"))
            .and_then(Value::as_str)
            .or_else(|| response.get("msg").and_then(Value::as_str))
            .unwrap_or("course cannot apply for exemption");
        Err(anyhow!("{message}"))
    }
}

async fn current_workflow_instance(
    client: &common::Client,
    student_id: &str,
    term: &str,
) -> Result<String> {
    let setting = r#"[{"name":"CSDM","value":"XK","builder":"equal","linkOpt":"AND"},{"name":"ZCSDM","value":"ZCXSLC,TSXSLC","builder":"m_value_equal","linkOpt":"AND"}]"#;
    let page: EhallPage<Value> = post_page(
        client,
        EXEMPTION_SETTING_URL,
        &[("setting".to_string(), setting.to_string())],
        EXEMPTION_SETTING_ACTION,
    )
    .await?;
    let mut workflows = HashMap::<String, String>::new();
    for row in page.rows {
        if let (Some(key), Some(value)) = (
            row.get("ZCSDM").and_then(Value::as_str),
            row.get("CSZA").and_then(Value::as_str),
        ) {
            workflows.insert(key.to_string(), value.to_string());
        }
    }
    let special_page: EhallPage<Value> = post_page(
        client,
        SPECIAL_STUDENT_URL,
        &[
            ("XH".to_string(), student_id.to_string()),
            ("XNXQDM".to_string(), term.to_string()),
        ],
        SPECIAL_STUDENT_ACTION,
    )
    .await?;
    let key = if special_page.rows.is_empty() {
        "ZCXSLC"
    } else {
        "TSXSLC"
    };

    workflows
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("workflow setting {key} was not found"))
}

async fn first_workflow_status(client: &common::Client, workflow_instance: &str) -> Result<String> {
    let page: EhallPage<Value> = post_page(
        client,
        FLOW_NEXT_STATUS_URL,
        &[
            ("SLDM".to_string(), workflow_instance.to_string()),
            ("DQZTDM".to_string(), "0".to_string()),
        ],
        FLOW_NEXT_STATUS_ACTION,
    )
    .await?;

    page.rows
        .first()
        .and_then(|row| row.get("XBZTDM"))
        .and_then(display_value)
        .ok_or_else(|| anyhow!("first workflow status was not found"))
}

async fn resolve_term(client: &common::Client, term: Option<&str>) -> Result<String> {
    match term.map(str::trim).filter(|term| !term.is_empty()) {
        Some(term) => Ok(term.to_string()),
        None => Ok(get_current_term(client).await?.id),
    }
}

fn term_query_setting(term: &str) -> Vec<Value> {
    vec![json!({
        "name": "XNXQDM",
        "value": term,
        "linkOpt": "AND",
        "builder": "equal"
    })]
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

async fn post_raw_action(
    client: &common::Client,
    url: &str,
    form: &[(String, String)],
    action: &str,
) -> Result<Value> {
    let value: Value = client
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

    if value.get("code").and_then(Value::as_str) != Some("0") {
        return Err(anyhow!("{url} returned application response {value}"));
    }

    value
        .get("datas")
        .and_then(|datas| datas.get(action))
        .cloned()
        .ok_or_else(|| anyhow!("{url} response did not contain data action {action}"))
}

fn extract_kssm_text(html: &str) -> Option<String> {
    let start = html.find("kssm-container")?;
    let div_start = html[..start].rfind("<div")?;
    let mut depth = 0_i32;
    let mut end = None;
    let mut index = div_start;

    while let Some(relative) = html[index..].find('<') {
        let tag_start = index + relative;
        if html[tag_start..].starts_with("<div") {
            depth += 1;
        } else if html[tag_start..].starts_with("</div") {
            depth -= 1;
            if depth == 0 {
                end = html[tag_start..]
                    .find('>')
                    .map(|relative| tag_start + relative + 1);
                break;
            }
        }
        index = tag_start + 1;
    }

    let fragment = &html[div_start..end?];
    let mut text = String::new();
    let mut in_tag = false;
    let mut last_was_space = false;
    for ch in fragment.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                if !last_was_space {
                    text.push('\n');
                    last_was_space = true;
                }
            }
            _ if in_tag => {}
            _ if ch.is_whitespace() => {
                if !last_was_space {
                    text.push(' ');
                    last_was_space = true;
                }
            }
            _ => {
                text.push(ch);
                last_was_space = false;
            }
        }
    }

    let decoded = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    let lines = decoded
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    Some(lines.join("\n"))
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
