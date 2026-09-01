use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const APP_ID: &str = "5006012186614764";
const APP_SHOW_URL: &str = "https://ehall.nju.edu.cn/appShow";
const PROGRAM_ASSIGNMENT_URL: &str =
    "https://ehallapp.nju.edu.cn/gsapp/sys/wdpyfaapp/modules/pyfaxq/gjxhcxdyfadm.do";
const PROGRAM_SUMMARY_URL: &str =
    "https://ehallapp.nju.edu.cn/gsapp/sys/wdpyfaapp/modules/pyfaxq/facx.do";
const PROGRAM_SECTIONS_URL: &str =
    "https://ehallapp.nju.edu.cn/gsapp/sys/wdpyfaapp/modules/pyfaxq/gjfadmhqdydpyfanr.do";
const PROGRAM_COURSES_URL: &str =
    "https://ehallapp.nju.edu.cn/gsapp/sys/wdpyfaapp/modules/pyfaxq/pyfakcxxcx.do";
const CREDIT_REQUIREMENTS_URL: &str =
    "https://ehallapp.nju.edu.cn/gsapp/sys/wdpyfaapp/wdFacxPyfakclbxfyqcx.do";
const ASSIGNMENT_ACTION: &str = "gjxhcxdyfadm";
const SUMMARY_ACTION: &str = "facx";
const SECTIONS_ACTION: &str = "gjfadmhqdydpyfanr";
const COURSES_ACTION: &str = "pyfakcxxcx";

/// 使用统一认证态打开「我的培养方案」，建立 ehall 和研究生应用会话。
///
/// 调用方需要传入已经带有 authserver `CASTGC` cookie 且允许自动跳转的 client。
/// 应用入口会自动完成 CAS 鉴权并跳转到 ehallapp；即使只调用接口也不能省略此步骤。
pub async fn prepare_session(client: &common::Client) -> Result<()> {
    let response = client
        .get(APP_SHOW_URL)
        .query(&[("appId", APP_ID)])
        .send()
        .await
        .context("failed to open ehall graduate training program app")?
        .error_for_status()
        .context("ehall graduate training program app entry returned an error status")?;
    let final_host = response.url().host_str().unwrap_or_default();

    if final_host != "ehallapp.nju.edu.cn"
        && !(final_host.starts_with("ehallapp-nju-edu-cn")
            && final_host.ends_with(".atrust.nju.edu.cn"))
    {
        return Err(anyhow!(
            "ehall graduate training program authentication ended at unexpected host {final_host:?}"
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgram {
    pub summary: GraduateTrainingProgramSummary,
    pub sections: Vec<GraduateTrainingProgramSection>,
    pub courses: Vec<GraduateTrainingProgramCourse>,
    pub credit_requirements: GraduateTrainingProgramCreditRequirements,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgramSummary {
    #[serde(rename = "DM")]
    pub id: String,
    #[serde(rename = "MC")]
    pub name: String,
    #[serde(rename = "NJDM", default)]
    pub grade_id: Option<String>,
    #[serde(rename = "NJDM_DISPLAY", default)]
    pub grade_name: Option<String>,
    #[serde(rename = "YXDM", default)]
    pub department_id: Option<String>,
    #[serde(rename = "YXDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "ZYDM", default)]
    pub major_id: Option<String>,
    #[serde(rename = "ZYDM_DISPLAY", default)]
    pub major_name: Option<String>,
    #[serde(rename = "YJXK", default)]
    pub discipline_id: Option<String>,
    #[serde(rename = "YJXK_DISPLAY", default)]
    pub discipline_name: Option<String>,
    #[serde(rename = "FALXDM", default)]
    pub program_type_id: Option<String>,
    #[serde(rename = "FALXDM_DISPLAY", default)]
    pub program_type_name: Option<String>,
    #[serde(rename = "SYXSLBMC", default)]
    pub applicable_student_types: Option<String>,
    #[serde(rename = "ZDXF", default)]
    pub minimum_credits: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgramSection {
    #[serde(rename = "WID")]
    pub id: String,
    #[serde(rename = "MC")]
    pub name: String,
    #[serde(rename = "MRZ", default)]
    pub content: Option<String>,
    #[serde(rename = "SM", default)]
    pub description: Option<String>,
    #[serde(rename = "LX", default)]
    pub section_type: Option<Value>,
    #[serde(rename = "PX", default)]
    pub order: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgramCourse {
    #[serde(rename = "FADM")]
    pub program_id: String,
    #[serde(rename = "KCDM")]
    pub id: String,
    #[serde(rename = "KCMC")]
    pub name: String,
    #[serde(rename = "KCLBDM", default)]
    pub category_id: Option<String>,
    #[serde(rename = "KCLBDM_DISPLAY", default)]
    pub category_name: Option<String>,
    #[serde(rename = "KKDWDM", default)]
    pub department_id: Option<String>,
    #[serde(rename = "KKDWDM_DISPLAY", default)]
    pub department_name: Option<String>,
    #[serde(rename = "ZXS", default)]
    pub hours: Option<Value>,
    #[serde(rename = "XF", default)]
    pub credits: Option<Value>,
    #[serde(rename = "KKXQDM", default)]
    pub term_id: Option<String>,
    #[serde(rename = "KKXQ", default)]
    pub term_name: Option<String>,
    #[serde(rename = "SFBX", default)]
    pub required_id: Option<Value>,
    #[serde(rename = "SFBX_DISPLAY", default)]
    pub required_name: Option<String>,
    #[serde(rename = "BZ", default)]
    pub remark: Option<String>,
    #[serde(rename = "DXZXSMC", default)]
    pub multiple_choice_group: Option<String>,
    #[serde(rename = "ZYFXDM", default)]
    pub major_direction_id: Option<String>,
    #[serde(rename = "ZYFXDM_DISPLAY", default)]
    pub major_direction_name: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgramCreditRequirements {
    #[serde(rename = "falxzxfyqResults", default)]
    pub programs: Vec<GraduateTrainingProgramSummary>,
    #[serde(rename = "falxdykclbxfyqResults", default)]
    pub categories: Vec<GraduateTrainingProgramCategoryRequirement>,
    #[serde(rename = "falxdykclbxfyqhbzResults", default)]
    pub groups: Vec<GraduateTrainingProgramCreditGroup>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgramCategoryRequirement {
    #[serde(rename = "FADM")]
    pub program_id: String,
    #[serde(rename = "KCLBDM", default)]
    pub category_id: Option<String>,
    #[serde(rename = "KCLBDM_DISPLAY", default)]
    pub category_name: Option<String>,
    #[serde(rename = "ZDXF", default)]
    pub minimum_credits: Option<Value>,
    #[serde(rename = "HBZWID", default)]
    pub group_id: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GraduateTrainingProgramCreditGroup {
    #[serde(rename = "WID")]
    pub id: String,
    #[serde(rename = "FADM")]
    pub program_id: String,
    #[serde(rename = "ZDXF", default)]
    pub minimum_credits: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct ProgramAssignment {
    #[serde(rename = "FADM")]
    program_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct EhallEnvelope<T> {
    datas: HashMap<String, EhallRows<T>>,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct EhallRows<T> {
    #[serde(rename = "totalSize", default)]
    total_size: u64,
    #[serde(default)]
    rows: Vec<T>,
}

pub async fn get_my_training_program(client: &common::Client) -> Result<GraduateTrainingProgram> {
    let assignment =
        post_rows::<ProgramAssignment>(client, PROGRAM_ASSIGNMENT_URL, &[], ASSIGNMENT_ACTION)
            .await?
            .rows
            .into_iter()
            .next()
            .ok_or_else(|| {
                anyhow!("the current graduate student does not have an assigned training program")
            })?;
    let program_id = assignment.program_id;
    let course_setting = serde_json::to_string(&[json!({
        "name": "FADM",
        "value": program_id,
    })])
    .context("failed to serialize graduate training program course query")?;

    let sections_form = [("DM", program_id.as_str())];
    let summary = get_summary(client, &program_id);
    let sections = post_rows(
        client,
        PROGRAM_SECTIONS_URL,
        &sections_form,
        SECTIONS_ACTION,
    );
    let courses = list_all_courses(client, &course_setting);
    let credit_requirements = get_credit_requirements(client, &program_id);
    let (summary, sections, courses, credit_requirements) =
        tokio::try_join!(summary, sections, courses, credit_requirements)?;

    Ok(GraduateTrainingProgram {
        summary,
        sections: sections.rows,
        courses,
        credit_requirements,
    })
}

async fn get_summary(
    client: &common::Client,
    program_id: &str,
) -> Result<GraduateTrainingProgramSummary> {
    post_rows(
        client,
        PROGRAM_SUMMARY_URL,
        &[("DM", program_id)],
        SUMMARY_ACTION,
    )
    .await?
    .rows
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("graduate training program {program_id} was not found"))
}

async fn list_all_courses(
    client: &common::Client,
    course_setting: &str,
) -> Result<Vec<GraduateTrainingProgramCourse>> {
    let mut page_number = 1_u64;
    let page_size = 200_u64;
    let mut courses = Vec::new();

    loop {
        let page_number_value = page_number.to_string();
        let page_size_value = page_size.to_string();
        let form = [
            ("setting", course_setting),
            ("pageNumber", page_number_value.as_str()),
            ("pageSize", page_size_value.as_str()),
        ];
        let page = post_rows(client, PROGRAM_COURSES_URL, &form, COURSES_ACTION).await?;
        let total_size = page.total_size;
        let row_count = page.rows.len();
        courses.extend(page.rows);

        if total_size == 0 || courses.len() as u64 >= total_size || row_count == 0 {
            break;
        }
        page_number += 1;
    }

    Ok(courses)
}

async fn get_credit_requirements(
    client: &common::Client,
    program_id: &str,
) -> Result<GraduateTrainingProgramCreditRequirements> {
    let body = client
        .post(CREDIT_REQUIREMENTS_URL)
        .form(&[("FADM", program_id)])
        .send()
        .await
        .context("failed to request graduate training program credit requirements")?
        .error_for_status()
        .context("graduate training program credit requirements returned an error status")?
        .text()
        .await
        .context("failed to read graduate training program credit requirements response")?;
    let requirements: Vec<GraduateTrainingProgramCreditRequirements> =
        serde_json::from_str(&body).with_context(|| {
            format!(
                "failed to parse graduate training program credit requirements as JSON: response starts with {:?}",
                body.chars().take(200).collect::<String>()
            )
        })?;

    Ok(requirements.into_iter().next().unwrap_or_default())
}

async fn post_rows<T>(
    client: &common::Client,
    url: &str,
    form: &[(&str, &str)],
    action: &str,
) -> Result<EhallRows<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let body = client
        .post(url)
        .form(form)
        .send()
        .await
        .with_context(|| format!("failed to request {url}"))?
        .error_for_status()
        .with_context(|| format!("{url} returned an error status"))?
        .text()
        .await
        .with_context(|| format!("failed to read {url} response body"))?;
    let envelope: EhallEnvelope<T> = serde_json::from_str(&body).with_context(|| {
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
