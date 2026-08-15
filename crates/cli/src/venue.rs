use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use ddddocr::BBox;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Subcommand)]
pub enum VenueCommand {
    /// 列出前台展示的场馆和可预约项目。
    List(ListVenuesOptions),
    /// 查看某个场地预约项目详情。
    Site {
        /// 场地项目名称或 ID，例如“健身房跑步机”。
        site: String,
        /// 校区名称，用于场地项目重名时消歧。
        #[arg(long)]
        campus: Option<String>,
        /// 场馆名称，用于场地项目重名时消歧。
        #[arg(long)]
        venue: Option<String>,
    },
    /// 查看某个日期的场地和时段预约状态。
    Status(StatusOptions),
    /// 一步完成预约：查状态、验证码识别、提交和免费付款确认。
    Book(BookOptions),
    /// 提交前获取订单确认信息。
    #[command(name = "order-info", hide = true)]
    OrderInfo(OrderInfoOptions),
    /// 获取点选文字验证码图片和词序。
    #[command(hide = true)]
    Captcha {
        /// 前端 clientUid；不传则生成 point-nju-cli。
        #[arg(long, default_value = "point-nju-cli")]
        client_uid: String,
        /// 保存验证码原图。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 使用 ddddocr 目标检测和裁剪识别输出点选坐标。
        #[arg(long)]
        ocr: bool,
    },
    /// 校验点选文字验证码坐标。
    #[command(name = "check-captcha", hide = true)]
    CheckCaptcha(CheckCaptchaOptions),
    /// 提交预约订单。
    #[command(hide = true)]
    Submit(SubmitOptions),
    /// 免费支付/确认订单。
    #[command(hide = true)]
    Pay {
        /// 订单 tradeNo。
        trade_no: String,
    },
    /// 列出我的预约记录。
    Orders(ListOrdersOptions),
    /// 查看预约详情。
    Order {
        /// 预约记录 id。
        order_id: u64,
    },
    /// 取消预约。
    Cancel(CancelOptions),
}

#[derive(Debug, Args)]
pub struct ListVenuesOptions {
    /// 校区名称或 ID，例如“仙林”“鼓楼”。
    #[arg(long)]
    campus: Option<String>,
    /// 运动名称或 ID，例如“跑步”“羽毛球”。
    #[arg(long, alias = "sport-type")]
    sport: Option<String>,
    /// 输出后端原始 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct StatusOptions {
    /// 场地项目名称或 ID，例如“健身房跑步机”。
    site: String,
    /// 查询日期，格式 YYYY-MM-DD。
    #[arg(long)]
    date: String,
    /// 校区名称，用于场地项目重名时消歧。
    #[arg(long)]
    campus: Option<String>,
    /// 场馆名称，用于场地项目重名时消歧。
    #[arg(long)]
    venue: Option<String>,
    /// 请求订单参数；提交预约前通常需要。
    #[arg(long, hide = true)]
    has_reserve_info: bool,
    /// 显示不可预约和已占用的时间段。
    #[arg(long)]
    include_unbookable: bool,
    /// 输出后端原始 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct SlotOptions {
    /// 场地 spaceId。
    #[arg(long)]
    space_id: String,
    /// 时段 timeId。
    #[arg(long)]
    time_id: String,
    /// 分组场地 ID。
    #[arg(long)]
    venue_space_group_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct OrderInfoOptions {
    /// 场地项目 ID。
    #[arg(long)]
    venue_site_id: u64,
    /// 预约日期，格式 YYYY-MM-DD。
    #[arg(long)]
    date: String,
    /// 周起始日期，通常取 status 响应的 reservationDateList 第一项。
    #[arg(long)]
    week_start_date: String,
    /// 预约类型；普通预约可不传。
    #[arg(long, hide = true)]
    reservation_type: Option<i64>,
    /// 场地 spaceId。
    #[arg(long)]
    space_id: String,
    /// 时段 timeId。
    #[arg(long)]
    time_id: String,
    /// 分组场地 ID。
    #[arg(long, hide = true)]
    venue_space_group_id: Option<String>,
    /// status 响应里的 token。
    #[arg(long)]
    token: Option<String>,
    /// 输出后端原始 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct BookOptions {
    /// 场地项目名称，例如“健身房跑步机”。
    #[arg(long)]
    site: Option<String>,
    /// 场地项目 ID。调试用；日常优先用 --site。
    #[arg(long, hide = true)]
    venue_site_id: Option<u64>,
    /// 预约日期，格式 YYYY-MM-DD。
    #[arg(long)]
    date: String,
    /// 校区名称，用于场地项目重名时消歧。
    #[arg(long)]
    campus: Option<String>,
    /// 场馆名称，用于场地项目重名时消歧。
    #[arg(long)]
    venue: Option<String>,
    /// 周起始日期；不传则使用状态接口返回的第一天。
    #[arg(long)]
    week_start_date: Option<String>,
    /// 预约类型；普通预约可不传。
    #[arg(long, hide = true)]
    reservation_type: Option<i64>,
    /// 场地名称，例如“6号跑步机”。
    #[arg(long)]
    space: Option<String>,
    /// 场地 spaceId。调试用；日常优先用 --space。
    #[arg(long, hide = true)]
    space_id: Option<String>,
    /// 时段，例如“21:30”或“21:30-22:00”。
    #[arg(long)]
    time: Option<String>,
    /// 时段 timeId。调试用；日常优先用 --time。
    #[arg(long, hide = true)]
    time_id: Option<String>,
    /// 分组场地 ID。
    #[arg(long, hide = true)]
    venue_space_group_id: Option<String>,
    /// 订单价格；不传时尝试从订单信息提取，提取不到则按免费场地传 0。
    #[arg(long, hide = true)]
    order_price: Option<f64>,
    /// 保存验证码原图，用于调试 OCR。
    #[arg(long)]
    captcha_output: Option<PathBuf>,
    /// 前端点击坐标 X，用于生成 orderPin。
    #[arg(long, default_value_t = 0, hide = true)]
    client_x: i64,
    /// 前端点击坐标 Y，用于生成 orderPin。
    #[arg(long, default_value_t = 0, hide = true)]
    client_y: i64,
    /// 提交成功后不自动发送免费付款确认请求。
    #[arg(long)]
    no_pay: bool,
    /// 输出完整中间结果 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct CheckCaptchaOptions {
    /// captcha/get 返回的 token。
    #[arg(long)]
    token: String,
    /// captcha/get 返回的 secretKey。
    #[arg(long)]
    secret_key: Option<String>,
    /// 点选坐标 JSON，例如 '[{"x":93,"y":56},{"x":160,"y":80},{"x":230,"y":72}]'。
    #[arg(long)]
    points_json: String,
}

#[derive(Debug, Args)]
pub struct SubmitOptions {
    /// 场地项目 ID。
    #[arg(long)]
    venue_site_id: u64,
    /// 预约日期，格式 YYYY-MM-DD。
    #[arg(long)]
    date: String,
    /// 周起始日期，通常取 status 响应的 reservationDateList 第一项。
    #[arg(long)]
    week_start_date: String,
    /// 预约类型；普通预约可不传。
    #[arg(long)]
    reservation_type: Option<i64>,
    /// 场地 spaceId。
    #[arg(long)]
    space_id: String,
    /// 时段 timeId。
    #[arg(long)]
    time_id: String,
    /// 分组场地 ID。
    #[arg(long)]
    venue_space_group_id: Option<String>,
    /// 订单价格；免费场地传 0。
    #[arg(long, default_value_t = 0.0)]
    order_price: f64,
    /// captcha/get 返回的 token。
    #[arg(long)]
    captcha_token: Option<String>,
    /// captcha/get 返回的 secretKey。
    #[arg(long)]
    captcha_secret_key: Option<String>,
    /// 点选坐标 JSON。传入后会生成 captchaVerification。
    #[arg(long)]
    captcha_points_json: Option<String>,
    /// captcha/check 后已经得到的 captchaVerification。
    #[arg(long)]
    captcha_verification: Option<String>,
    /// 前端点击坐标 X，用于生成 orderPin。
    #[arg(long, default_value_t = 0)]
    client_x: i64,
    /// 前端点击坐标 Y，用于生成 orderPin。
    #[arg(long, default_value_t = 0)]
    client_y: i64,
    /// status 响应里的 token。
    #[arg(long)]
    token: Option<String>,
    /// 提交成功后自动发免费付款请求。
    #[arg(long)]
    pay: bool,
}

#[derive(Debug, Args)]
pub struct ListOrdersOptions {
    /// 页码，从 1 开始。
    #[arg(long, default_value_t = 1)]
    page: u64,
    /// 每页数量。
    #[arg(long, default_value_t = 20)]
    size: u64,
    /// 输出后端原始 JSON。
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct CancelOptions {
    /// 预约 tradeNo。
    trade_no: String,
    /// 取消备注。
    #[arg(long, default_value = "")]
    remark: String,
}

pub async fn handle(command: VenueCommand, client: &common::Client) -> Result<()> {
    let session = venue::prepare_session(client)
        .await
        .context("failed to prepare venue session; try running `nju-cli login` again")?;

    match command {
        VenueCommand::List(options) => {
            let campus = resolve_campus(options.campus.as_deref())?;
            let sport = resolve_sport(options.sport.as_deref())?;
            let data = venue::list_campus_venue_info(&session, sport)
                .await
                .context("failed to list venue info")?;
            if options.json {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                print_venue_list(&data, campus)?;
            }
        }
        VenueCommand::Site {
            site,
            campus,
            venue,
        } => {
            let venue_site_id =
                resolve_venue_site_id(&session, &site, campus.as_deref(), venue.as_deref()).await?;
            let data = venue::get_venue_site(&session, venue_site_id)
                .await
                .with_context(|| format!("failed to get venue site {venue_site_id}"))?;
            print_site(&data)?;
        }
        VenueCommand::Status(options) => {
            let venue_site_id = resolve_venue_site_id(
                &session,
                &options.site,
                options.campus.as_deref(),
                options.venue.as_deref(),
            )
            .await?;
            let data = venue::get_reservation_day(
                &session,
                venue_site_id,
                &options.date,
                options.has_reserve_info,
            )
            .await
            .context("failed to get reservation status")?;
            if options.json {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                print_status(&data, options.include_unbookable);
            }
        }
        VenueCommand::Book(options) => {
            let venue_site_id = resolve_book_site_id(&session, &options).await?;
            let status = venue::get_reservation_day(&session, venue_site_id, &options.date, true)
                .await
                .context("failed to get reservation status")?;
            let slot = resolve_slot(&status, &options.date, &options)?;
            ensure_slot_free(&status, &options.date, &slot.space_id, &slot.time_id)?;

            let token = status
                .get("token")
                .and_then(Value::as_str)
                .map(str::to_string)
                .context("reservation status response did not include token")?;
            let week_start_date = options
                .week_start_date
                .clone()
                .or_else(|| {
                    status
                        .get("reservationDateList")
                        .and_then(Value::as_array)
                        .and_then(|items| items.first())
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_else(|| options.date.clone());

            let reservation_order_json = reservation_order_json(
                slot.space_id.clone(),
                slot.time_id.clone(),
                slot.venue_space_group_id.clone(),
            )?;
            let captcha_solution = solve_captcha(&session, options.captcha_output.as_ref())
                .await
                .context("failed to solve captcha")?;
            let check = venue::check_captcha(
                &session,
                &captcha_solution.token,
                captcha_solution.secret_key.as_deref(),
                &captcha_solution.points_json,
            )
            .await
            .context("failed to check captcha")?;
            ensure_captcha_success(&check)?;
            let captcha_verification = venue::captcha_verification(
                &captcha_solution.token,
                captcha_solution.secret_key.as_deref(),
                &captcha_solution.points_json,
            )?;

            let submit_request = venue::SubmitOrderRequest {
                venue_site_id,
                reservation_date: options.date.clone(),
                week_start_date,
                reservation_order_json,
                reservation_type: options.reservation_type,
                order_price: Some(options.order_price.unwrap_or(0.0)),
                order_pin: Some(venue::order_pin(options.client_x, options.client_y)?),
                captcha_verification: Some(captcha_verification),
                captcha_token: Some(captcha_solution.token.clone()),
                token: Some(token),
            };
            let submit = venue::submit_order(&session, &submit_request)
                .await
                .context("failed to submit reservation order")?;

            let pay = if options.no_pay {
                None
            } else {
                let trade_no = submit
                    .trade_no
                    .as_deref()
                    .ok_or_else(|| anyhow!("submit response did not include tradeNo"))?;
                Some(
                    venue::pay_order(&session, trade_no)
                        .await
                        .with_context(|| format!("failed to pay order {trade_no}"))?,
                )
            };

            if options.json {
                let result = serde_json::json!({
                    "status": status,
                    "captchaImage": options.captcha_output,
                    "captchaRecognition": captcha_solution.recognition,
                    "captchaCheck": check,
                    "submit": submit,
                    "pay": pay,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let result = serde_json::json!({
                    "status": "booked",
                    "tradeNo": submit.trade_no,
                    "orderId": submit.id,
                    "captchaImage": options.captcha_output,
                    "paid": pay.is_some(),
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        VenueCommand::OrderInfo(options) => {
            let request = venue::OrderInfoRequest {
                venue_site_id: options.venue_site_id,
                reservation_date: options.date,
                week_start_date: options.week_start_date,
                reservation_order_json: reservation_order_json(
                    options.space_id,
                    options.time_id,
                    options.venue_space_group_id,
                )?,
                reservation_type: options.reservation_type,
                school_type: None,
                token: options.token,
            };
            let data = venue::get_order_info(&session, &request)
                .await
                .context("failed to get order info")?;
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        VenueCommand::Captcha {
            client_uid,
            output,
            ocr,
        } => {
            let data = venue::get_captcha(&session, &client_uid)
                .await
                .context("failed to get captcha")?;
            if let Some(output) = output {
                save_captcha_image(&data, &output)?;
                println!("captchaImage\t{}", output.display());
            }
            if ocr {
                let recognition = recognize_captcha_points(&data)?;
                println!("{}", serde_json::to_string_pretty(&recognition)?);
                if let Some(points_json) = recognition.points_json() {
                    println!("pointsJson\t{points_json}");
                }
            }
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        VenueCommand::CheckCaptcha(options) => {
            let data = venue::check_captcha(
                &session,
                &options.token,
                options.secret_key.as_deref(),
                &options.points_json,
            )
            .await
            .context("failed to check captcha")?;
            let verification = venue::captcha_verification(
                &options.token,
                options.secret_key.as_deref(),
                &options.points_json,
            )?;
            println!("{}", serde_json::to_string_pretty(&data)?);
            println!("captchaVerification\t{verification}");
        }
        VenueCommand::Submit(options) => {
            let captcha_verification = match (
                options.captcha_verification,
                options.captcha_token.as_deref(),
                options.captcha_secret_key.as_deref(),
                options.captcha_points_json.as_deref(),
            ) {
                (Some(value), _, _, _) => Some(value),
                (None, Some(token), secret_key, Some(points_json)) => {
                    Some(venue::captcha_verification(token, secret_key, points_json)?)
                }
                (None, _, _, _) => None,
            };
            let request = venue::SubmitOrderRequest {
                venue_site_id: options.venue_site_id,
                reservation_date: options.date,
                week_start_date: options.week_start_date,
                reservation_order_json: reservation_order_json(
                    options.space_id,
                    options.time_id,
                    options.venue_space_group_id,
                )?,
                reservation_type: options.reservation_type,
                order_price: Some(options.order_price),
                order_pin: Some(venue::order_pin(options.client_x, options.client_y)?),
                captcha_verification,
                captcha_token: options.captcha_token,
                token: options.token,
            };
            let data = venue::submit_order(&session, &request)
                .await
                .context("failed to submit reservation order")?;

            println!("{}", serde_json::to_string_pretty(&data)?);
            if options.pay {
                let trade_no = data
                    .trade_no
                    .as_deref()
                    .ok_or_else(|| anyhow!("submit response did not include tradeNo"))?;
                let pay = venue::pay_order(&session, trade_no)
                    .await
                    .with_context(|| format!("failed to pay order {trade_no}"))?;
                println!("{}", serde_json::to_string_pretty(&pay)?);
            }
        }
        VenueCommand::Pay { trade_no } => {
            let data = venue::pay_order(&session, &trade_no)
                .await
                .with_context(|| format!("failed to pay order {trade_no}"))?;
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        VenueCommand::Orders(options) => {
            let data = venue::list_orders(&session, options.page.saturating_sub(1), options.size)
                .await
                .context("failed to list orders")?;
            if options.json {
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                print_orders(&data);
            }
        }
        VenueCommand::Order { order_id } => {
            let data = venue::get_order(&session, order_id)
                .await
                .with_context(|| format!("failed to get order {order_id}"))?;
            print_order_detail(&data)?;
        }
        VenueCommand::Cancel(options) => {
            let data = venue::cancel_order(&session, &options.trade_no, &options.remark)
                .await
                .with_context(|| format!("failed to cancel order {}", options.trade_no))?;
            let result = serde_json::json!({
                "status": "cancelled",
                "tradeNo": options.trade_no,
                "response": data,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn reservation_order_json(
    space_id: String,
    time_id: String,
    venue_space_group_id: Option<String>,
) -> Result<String> {
    venue::reservation_order_json(&[venue::ReservationItem {
        space_id,
        time_id,
        venue_space_group_id,
    }])
}

#[derive(Debug, Clone)]
struct SlotSelection {
    space_id: String,
    time_id: String,
    venue_space_group_id: Option<String>,
}

struct CaptchaSolution {
    token: String,
    secret_key: Option<String>,
    recognition: CaptchaRecognition,
    points_json: String,
}

const CAMPUS_ALIASES: &[(&str, u64)] = &[
    ("仙林", 51),
    ("仙林校区", 51),
    ("鼓楼", 146),
    ("鼓楼校区", 146),
    ("浦口", 155),
    ("浦口校区", 155),
    ("苏州", 156),
    ("苏州校区", 156),
];

const SPORT_ALIASES: &[(&str, u64)] = &[
    ("乒乓球", 150),
    ("兵乓球", 150),
    ("羽毛球", 151),
    ("健身", 152),
    ("跑步", 153),
    ("网球", 154),
    ("篮球", 155),
    ("排球", 156),
    ("足球", 157),
    ("游泳", 158),
    ("其他", 159),
];

fn resolve_campus(value: Option<&str>) -> Result<Option<u64>> {
    resolve_alias(value, CAMPUS_ALIASES, "校区")
}

fn resolve_sport(value: Option<&str>) -> Result<Option<u64>> {
    resolve_alias(value, SPORT_ALIASES, "运动")
}

fn resolve_alias(value: Option<&str>, aliases: &[(&str, u64)], label: &str) -> Result<Option<u64>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if let Ok(id) = value.parse::<u64>() {
        return Ok(Some(id));
    }

    let normalized = normalize_name(value);
    let matches = aliases
        .iter()
        .filter(|(name, _)| {
            normalize_name(name) == normalized || normalize_name(name).contains(&normalized)
        })
        .map(|(_, id)| *id)
        .collect::<HashSet<_>>();
    match matches.len() {
        0 => Err(anyhow!("unknown {label}: {value}")),
        1 => Ok(matches.into_iter().next()),
        _ => Err(anyhow!("ambiguous {label}: {value}")),
    }
}

async fn resolve_book_site_id(
    session: &venue::VenueSession<'_>,
    options: &BookOptions,
) -> Result<u64> {
    if let Some(id) = options.venue_site_id {
        return Ok(id);
    }
    let site = options
        .site
        .as_deref()
        .context("missing --site; use a site name like --site 健身房跑步机")?;
    resolve_venue_site_id(
        session,
        site,
        options.campus.as_deref(),
        options.venue.as_deref(),
    )
    .await
}

async fn resolve_venue_site_id(
    session: &venue::VenueSession<'_>,
    site: &str,
    campus: Option<&str>,
    venue_name: Option<&str>,
) -> Result<u64> {
    if let Ok(id) = site.parse::<u64>() {
        return Ok(id);
    }

    let campus_id = resolve_campus(campus)?;
    let data = venue::list_campus_venue_info(session, None)
        .await
        .context("failed to load venue site name mapping")?;
    let sites = flatten_venue_sites(&data);
    let normalized_site = normalize_name(site);
    let normalized_venue = venue_name.map(normalize_name);

    let matches = sites
        .into_iter()
        .filter(|candidate| {
            let site_match = normalize_name(&candidate.site_name) == normalized_site
                || normalize_name(&candidate.site_name).contains(&normalized_site);
            let campus_match = campus_id
                .map(|id| candidate.campus_id == Some(id))
                .unwrap_or(true);
            let venue_match = normalized_venue
                .as_ref()
                .map(|venue| normalize_name(&candidate.venue_name).contains(venue))
                .unwrap_or(true);
            site_match && campus_match && venue_match
        })
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => Err(anyhow!("no venue site matched: {site}")),
        [candidate] => Ok(candidate.id),
        _ => Err(anyhow!(
            "ambiguous venue site {site}; add --campus or --venue. candidates: {}",
            matches
                .iter()
                .map(|item| format!(
                    "{} / {} / {}",
                    item.campus_name, item.venue_name, item.site_name
                ))
                .collect::<Vec<_>>()
                .join("; ")
        )),
    }
}

#[derive(Debug)]
struct VenueSiteCandidate {
    id: u64,
    campus_id: Option<u64>,
    campus_name: String,
    venue_name: String,
    site_name: String,
}

fn flatten_venue_sites(data: &venue::CampusVenueInfo) -> Vec<VenueSiteCandidate> {
    data.venue_site_info
        .values()
        .flat_map(|sites| sites.iter())
        .map(|site| VenueSiteCandidate {
            id: site.id,
            campus_id: site.campus_id,
            campus_name: site.campus_name.clone(),
            venue_name: site.venue_name.clone(),
            site_name: site.site_name.clone(),
        })
        .collect()
}

fn resolve_slot(status: &Value, date: &str, options: &BookOptions) -> Result<SlotSelection> {
    let spaces = status
        .get("reservationDateSpaceInfo")
        .and_then(|items| items.get(date))
        .and_then(Value::as_array)
        .with_context(|| format!("reservation status did not include date {date}"))?;
    let space = match options.space_id.as_deref() {
        Some(space_id) => spaces
            .iter()
            .find(|space| value_id_matches(space.get("id"), space_id))
            .with_context(|| format!("reservation status did not include spaceId {space_id}"))?,
        None => {
            let space_name = options
                .space
                .as_deref()
                .context("missing --space; use a space name like --space 6号跑步机")?;
            find_by_name_or_id(spaces, space_name, "id", "spaceName", "场地")?
        }
    };

    let time_id = match options.time_id.as_deref() {
        Some(time_id) => time_id.to_string(),
        None => {
            let time = options
                .time
                .as_deref()
                .context("missing --time; use a time like --time 21:30 or --time 21:30-22:00")?;
            resolve_time_id(status, time)?
        }
    };

    Ok(SlotSelection {
        space_id: value_id_string(space.get("id")).context("space did not include id")?,
        time_id,
        venue_space_group_id: options.venue_space_group_id.clone().or_else(|| {
            space
                .get("venueSpaceGroupId")
                .and_then(|value| value_id_string(Some(value)))
        }),
    })
}

fn resolve_time_id(status: &Value, time: &str) -> Result<String> {
    let times = status
        .get("spaceTimeInfo")
        .and_then(Value::as_array)
        .context("reservation status did not include spaceTimeInfo")?;
    if time.parse::<u64>().is_ok() {
        return Ok(time.to_string());
    }
    let normalized = normalize_name(time);
    let matches = times
        .iter()
        .filter(|item| {
            let begin = value_string(item, "beginTime");
            let end = value_string(item, "endTime");
            normalize_name(&begin) == normalized
                || normalize_name(&format!("{begin}-{end}")) == normalized
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(anyhow!("no time slot matched: {time}")),
        [item] => value_id_string(item.get("id")).context("time slot did not include id"),
        _ => Err(anyhow!("ambiguous time slot: {time}")),
    }
}

fn find_by_name_or_id<'a>(
    items: &'a [Value],
    input: &str,
    id_key: &str,
    name_key: &str,
    label: &str,
) -> Result<&'a Value> {
    if input.parse::<u64>().is_ok() {
        return items
            .iter()
            .find(|item| value_id_matches(item.get(id_key), input))
            .with_context(|| format!("{label} id not found: {input}"));
    }
    let normalized = normalize_name(input);
    let matches = items
        .iter()
        .filter(|item| normalize_name(&value_string(item, name_key)) == normalized)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(anyhow!("{label} not found: {input}")),
        [item] => Ok(item),
        _ => Err(anyhow!("ambiguous {label}: {input}")),
    }
}

fn ensure_slot_free(status: &Value, date: &str, space_id: &str, time_id: &str) -> Result<()> {
    let spaces = status
        .get("reservationDateSpaceInfo")
        .and_then(|items| items.get(date))
        .and_then(Value::as_array)
        .with_context(|| format!("reservation status did not include date {date}"))?;

    let space = spaces
        .iter()
        .find(|space| value_id_matches(space.get("id"), space_id))
        .with_context(|| format!("reservation status did not include spaceId {space_id}"))?;
    let slot = space
        .get(time_id)
        .with_context(|| format!("reservation status did not include timeId {time_id}"))?;
    let status = slot
        .get("reservationStatus")
        .and_then(Value::as_i64)
        .context("slot did not include reservationStatus")?;

    if status == 1 {
        return Ok(());
    }

    let space_name = space
        .get("spaceName")
        .and_then(Value::as_str)
        .unwrap_or(space_id);
    let start = slot
        .get("startDate")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let end = slot
        .get("endDate")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Err(anyhow!(
        "{space_name} {start}-{end} is not free: {}",
        reservation_status_title(status)
    ))
}

async fn solve_captcha(
    session: &venue::VenueSession<'_>,
    output: Option<&PathBuf>,
) -> Result<CaptchaSolution> {
    let mut last_missing_words = Vec::new();
    for _ in 0..5 {
        let captcha = venue::get_captcha(session, "point-nju-cli")
            .await
            .context("failed to get captcha")?;
        let token = captcha
            .get("repData")
            .and_then(|rep_data| rep_data.get("token"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .context("captcha response did not include token")?;
        let secret_key = captcha
            .get("repData")
            .and_then(|rep_data| rep_data.get("secretKey"))
            .and_then(Value::as_str)
            .map(str::to_string);
        if let Some(output) = output {
            save_captcha_image(&captcha, output)?;
        }

        let recognition = recognize_captcha_points(&captcha)?;
        if let Some(points_json) = recognition.points_json() {
            return Ok(CaptchaSolution {
                token,
                secret_key,
                recognition,
                points_json,
            });
        }
        last_missing_words = recognition.missing_words;
    }

    Err(anyhow!(
        "captcha OCR did not recognize all requested words after retries: missing {:?}",
        last_missing_words
    ))
}

fn ensure_captcha_success(check: &Value) -> Result<()> {
    let success = check
        .get("repCode")
        .and_then(Value::as_str)
        .map(|code| code == "0000")
        .unwrap_or(false);
    if success {
        Ok(())
    } else {
        Err(anyhow!(
            "captcha check failed: {}",
            check
                .get("repMsg")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
        ))
    }
}

fn value_id_matches(value: Option<&Value>, expected: &str) -> bool {
    let Some(value) = value else {
        return false;
    };
    value.as_str() == Some(expected)
        || value
            .as_u64()
            .map(|id| id.to_string() == expected)
            .unwrap_or(false)
}

fn value_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn value_id_string(value: Option<&Value>) -> Option<String> {
    let value = value?;
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| value.as_u64().map(|id| id.to_string()))
        .or_else(|| value.as_i64().map(|id| id.to_string()))
}

fn normalize_name(value: &str) -> String {
    value
        .trim()
        .replace('（', "(")
        .replace('）', ")")
        .replace(' ', "")
        .to_lowercase()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptchaRecognition {
    word_list: Vec<String>,
    boxes: Vec<CaptchaBoxRecognition>,
    points: Vec<CaptchaPoint>,
    missing_words: Vec<String>,
}

impl CaptchaRecognition {
    fn points_json(&self) -> Option<String> {
        if self.points.len() != self.word_list.len() {
            return None;
        }

        serde_json::to_string(&self.points).ok()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptchaBoxRecognition {
    bbox: CaptchaBBox,
    text: String,
    center: CaptchaPoint,
}

#[derive(Debug, Serialize)]
struct CaptchaBBox {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

impl From<BBox> for CaptchaBBox {
    fn from(value: BBox) -> Self {
        Self {
            x1: value.x1,
            y1: value.y1,
            x2: value.x2,
            y2: value.y2,
        }
    }
}

#[derive(Debug, Serialize)]
struct CaptchaPoint {
    x: u32,
    y: u32,
}

fn recognize_captcha_points(data: &Value) -> Result<CaptchaRecognition> {
    let image = captcha_image(data)?;
    let word_list = data
        .get("repData")
        .and_then(|rep_data| rep_data.get("wordList"))
        .and_then(Value::as_array)
        .context("captcha response did not include wordList")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("captcha wordList included a non-string item"))
        })
        .collect::<Result<Vec<_>>>()?;

    let detector = ddddocr::ddddocr_detection().context("failed to initialize ddddocr detector")?;
    let boxes = detector
        .detection(&image)
        .context("failed to detect captcha words with ddddocr")?;

    let classifier =
        ddddocr::ddddocr_classification().context("failed to initialize ddddocr classifier")?;
    let mut recognized = classifier
        .classification_bbox(&image, &boxes)
        .context("failed to classify detected captcha boxes with ddddocr")?
        .into_iter()
        .map(|(bbox, text)| CaptchaBoxRecognition {
            center: CaptchaPoint {
                x: (bbox.x1 + bbox.x2) / 2,
                y: (bbox.y1 + bbox.y2) / 2,
            },
            bbox: bbox.into(),
            text: text.trim().to_string(),
        })
        .collect::<Vec<_>>();

    recognized.sort_by_key(|item| (item.bbox.y1, item.bbox.x1));

    let mut used = HashSet::new();
    let mut points = Vec::new();
    let mut missing_words = Vec::new();
    for word in &word_list {
        let Some((index, item)) = recognized
            .iter()
            .enumerate()
            .find(|(index, item)| !used.contains(index) && item.text.contains(word))
        else {
            missing_words.push(word.clone());
            continue;
        };
        used.insert(index);
        points.push(CaptchaPoint {
            x: item.center.x,
            y: item.center.y,
        });
    }

    Ok(CaptchaRecognition {
        word_list,
        boxes: recognized,
        points,
        missing_words,
    })
}

fn save_captcha_image(data: &Value, output: &PathBuf) -> Result<()> {
    let image = captcha_image(data)?;

    fs::write(output, image).with_context(|| format!("failed to write {}", output.display()))
}

fn captcha_image(data: &Value) -> Result<Vec<u8>> {
    let base64 = data
        .get("repData")
        .and_then(|rep_data| rep_data.get("originalImageBase64"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("captcha response did not include originalImageBase64"))?;

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64)
        .context("failed to decode captcha image base64")
}

fn print_venue_list(data: &venue::CampusVenueInfo, campus_id: Option<u64>) -> Result<()> {
    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for site in flatten_venue_sites(data).into_iter().filter(|site| {
        campus_id
            .map(|id| site.campus_id == Some(id))
            .unwrap_or(true)
    }) {
        grouped
            .entry(site.campus_name)
            .or_default()
            .push(format!("{}（{}）", site.site_name, site.venue_name));
    }

    for (index, (campus, mut sites)) in grouped.into_iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!("## {campus}");
        sites.sort();
        sites.dedup();
        for site in sites {
            println!("- {site}");
        }
    }
    Ok(())
}

fn print_site(data: &Value) -> Result<()> {
    println!(
        "## {} {}",
        value_string(data, "venueName"),
        value_string(data, "siteName")
    );
    println!("校区：{}", value_string(data, "campusName"));
    println!(
        "开放时间：{}-{}",
        value_string(data, "openStartDate"),
        value_string(data, "openEndDate")
    );
    if let Some(count) = data.get("reservationSpaceCount").and_then(Value::as_u64) {
        println!("可预约场地数：{count}");
    }
    if let Some(count) = data.get("spaceCount").and_then(Value::as_u64) {
        println!("场地数：{count}");
    }
    let telephone = value_string(data, "siteTelephone");
    if !telephone.is_empty() {
        println!("联系电话：{telephone}");
    }
    Ok(())
}

fn print_status(data: &Value, include_unbookable: bool) {
    let Some(times) = data.get("spaceTimeInfo").and_then(Value::as_array) else {
        println!("{data}");
        return;
    };
    let Some(date_spaces) = data
        .get("reservationDateSpaceInfo")
        .and_then(Value::as_object)
    else {
        println!("{data}");
        return;
    };

    let mut lines = Vec::new();
    for (date, spaces) in date_spaces {
        let Some(spaces) = spaces.as_array() else {
            continue;
        };
        for space in spaces {
            let space_name = space
                .get("spaceName")
                .and_then(Value::as_str)
                .unwrap_or_default();
            for time in times {
                let time_id = time.get("id").map(Value::to_string).unwrap_or_default();
                let begin = time
                    .get("beginTime")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let end = time
                    .get("endTime")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let status = space
                    .get(&time_id)
                    .and_then(|slot| slot.get("reservationStatus"))
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                if !include_unbookable && status != 1 {
                    continue;
                }
                let text = if include_unbookable {
                    format!("{begin}-{end} {}", reservation_status_title(status))
                } else {
                    format!("{begin}-{end}")
                };
                lines.push((date.clone(), space_name.to_string(), text));
            }
        }
    }

    let mut grouped = BTreeMap::<(String, String), Vec<String>>::new();
    for (date, space, text) in lines {
        grouped.entry((date, space)).or_default().push(text);
    }
    for ((_date, space), slots) in grouped {
        println!("{space}：{}", slots.join(", "));
    }
}

fn print_orders(data: &Value) {
    let Some(items) = data.get("content").and_then(Value::as_array) else {
        println!("{data}");
        return;
    };

    for item in items {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            item.get("id").map(Value::to_string).unwrap_or_default(),
            item.get("tradeNo")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            item.get("campusName")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            item.get("venueName")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            item.get("siteName")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            item.get("reservationDateDetail")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            order_status_title(
                item.get("orderStatus")
                    .and_then(Value::as_i64)
                    .unwrap_or_default()
            ),
        );
    }
}

fn print_order_detail(data: &Value) -> Result<()> {
    let order_info = data.get("orderInfo").unwrap_or(data);
    let venue_info = data.get("venueInfoBean").unwrap_or(data);
    let summary = serde_json::json!({
        "tradeNo": order_info.get("tradeNo").and_then(Value::as_str),
        "status": order_info.get("orderStatus").and_then(Value::as_i64).map(order_status_title),
        "campus": value_string(venue_info, "campusName"),
        "venue": value_string(venue_info, "venueName"),
        "site": value_string(venue_info, "siteName"),
        "date": value_string(order_info, "reservationDate"),
        "time": value_string(order_info, "reservationDateDetail"),
        "amount": order_info.get("orderAmount").or_else(|| order_info.get("totalAmount")),
    });
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn reservation_status_title(status: i64) -> &'static str {
    match status {
        1 => "空闲",
        2 => "不可用",
        3 => "未支付",
        4 => "已被占用",
        _ => "未知",
    }
}

fn order_status_title(status: i64) -> &'static str {
    match status {
        1 => "有效",
        2 => "已取消",
        _ => "未知",
    }
}
