use std::time::Duration;

use anyhow::{Context, Result, anyhow, ensure};
use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, prelude::IndexedRandom};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::{request, utils};

const CAPTCHA_INIT_URL: &str =
    "https://authserver.nju.edu.cn/authserver/common/toSliderCaptcha.htl";
const CAPTCHA_OPEN_URL: &str =
    "https://authserver.nju.edu.cn/authserver/common/openSliderCaptcha.htl";
const CAPTCHA_VERIFY_URL: &str =
    "https://authserver.nju.edu.cn/authserver/common/verifySliderCaptcha.htl";
const MAX_CAPTCHA_ATTEMPTS: usize = 5;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptchaImages {
    big_image: String,
    small_image: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptchaResult {
    error_code: i64,
    error_msg: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptchaTrack {
    canvas_length: u32,
    move_length: f64,
    tracks: Vec<TrackPoint>,
}

#[derive(Debug, Serialize)]
struct TrackPoint {
    a: Number,
    b: i32,
    c: u64,
}

pub(crate) async fn verify_slider_captcha(client: &Client) -> Result<()> {
    let mut last_error = None;

    for attempt in 1..=MAX_CAPTCHA_ATTEMPTS {
        match verify_once(client).await {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error.context(format!(
                    "NJU auth slider captcha attempt {attempt}/{MAX_CAPTCHA_ATTEMPTS} failed"
                )));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("NJU auth slider captcha failed")))
}

async fn verify_once(client: &Client) -> Result<()> {
    request::send(
        client.get(CAPTCHA_INIT_URL),
        "initialize NJU auth slider captcha",
    )
    .await?;

    let images: CaptchaImages = request::send(
        client.get(CAPTCHA_OPEN_URL),
        "request NJU auth slider captcha images",
    )
    .await?
    .json()
    .await
    .context("failed to parse NJU auth slider captcha images")?;

    let big_image = decode_image("bigImage", &images.big_image)?;
    let small_image = decode_image("smallImage", &images.small_image)?;
    let track = solve_captcha(&big_image, &small_image)?;
    let wait = track.tracks.iter().map(|point| point.c).sum();
    let plaintext = serde_json::to_string(&track)
        .context("failed to serialize NJU auth slider captcha track")?;
    let key = utils::safe_secure(&small_image)?;
    let sign = utils::encrypt(&plaintext, key)?;

    tokio::time::sleep(Duration::from_millis(wait)).await;

    let result: CaptchaResult = request::send(
        client.post(CAPTCHA_VERIFY_URL).form(&[("sign", sign)]),
        "verify NJU auth slider captcha",
    )
    .await?
    .json()
    .await
    .context("failed to parse NJU auth slider captcha verification response")?;

    ensure!(
        result.error_msg == "success",
        "NJU auth slider captcha was rejected: errorCode={}, errorMsg={}",
        result.error_code,
        result.error_msg
    );

    Ok(())
}

fn decode_image(name: &str, encoded: &str) -> Result<Vec<u8>> {
    general_purpose::STANDARD
        .decode(encoded)
        .with_context(|| format!("failed to decode NJU auth slider captcha {name}"))
}

fn solve_captcha(big_image: &[u8], small_image: &[u8]) -> Result<CaptchaTrack> {
    let position = ddddocr::slide_match(small_image, big_image)
        .context("failed to locate NJU auth slider captcha gap")?;
    let background_width = image::load_from_memory(big_image)
        .context("failed to decode NJU auth slider captcha background")?
        .width();
    ensure!(
        background_width > 0,
        "NJU auth slider captcha has zero width"
    );

    let moved_offset = f64::from(position.x1) * (280.0 / f64::from(background_width));
    generate_track(moved_offset)
}

fn generate_track(moved_offset: f64) -> Result<CaptchaTrack> {
    ensure!(
        moved_offset.is_finite() && moved_offset >= 0.0,
        "invalid NJU auth slider captcha offset: {moved_offset}"
    );

    let mut rng = rand::rng();
    let sample_count = rng.random_range(19..=22);
    let mut tracks = Vec::with_capacity(sample_count);
    let mut previous_x = 0_i64;

    for index in 1..=sample_count {
        let progress = index as f64 / sample_count as f64;
        let mut eased_progress = progress * progress * (3.0 - 2.0 * progress);
        let (x, serialized_x) = if index < sample_count {
            eased_progress += rng.random_range(-0.004..0.004);
            let x = (moved_offset * eased_progress)
                .round()
                .clamp(previous_x as f64, moved_offset) as i64;
            (x, Number::from(x))
        } else {
            let x = moved_offset.round() as i64;
            let serialized = Number::from_f64(moved_offset)
                .context("failed to serialize NJU auth slider captcha offset")?;
            (x, serialized)
        };

        let interval = if index <= 2 {
            rng.random_range(45..=85)
        } else if index == sample_count {
            rng.random_range(400..=700)
        } else if index >= sample_count - 2 {
            rng.random_range(70..=160)
        } else {
            rng.random_range(28..=38)
        };
        let vertical_offset = if index == sample_count {
            0
        } else {
            *[-1, 0, 0, 0, 1]
                .choose(&mut rng)
                .expect("vertical offset choices are not empty")
        };

        tracks.push(TrackPoint {
            a: serialized_x,
            b: vertical_offset,
            c: interval,
        });
        previous_x = x;
    }

    Ok(CaptchaTrack {
        canvas_length: 280,
        move_length: moved_offset,
        tracks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_track_has_expected_shape() -> Result<()> {
        let moved_offset = 123.75;
        let track = generate_track(moved_offset)?;

        assert_eq!(track.canvas_length, 280);
        assert_eq!(track.move_length, moved_offset);
        assert!((19..=22).contains(&track.tracks.len()));
        assert_eq!(
            track.tracks.last().and_then(|point| point.a.as_f64()),
            Some(moved_offset)
        );
        assert_eq!(track.tracks.last().map(|point| point.b), Some(0));

        let positions = track
            .tracks
            .iter()
            .map(|point| point.a.as_f64().context("track position is not a number"))
            .collect::<Result<Vec<_>>>()?;
        assert!(positions.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(track.tracks.iter().all(|point| (-1..=1).contains(&point.b)));

        Ok(())
    }
}
