use std::{env, path::Path, time::Duration};

use anyhow::{Context, Result, anyhow};
use chromiumoxide::{
    Browser, BrowserConfig, Page,
    cdp::browser_protocol::{
        emulation::{ScreenOrientation, ScreenOrientationType, SetDeviceMetricsOverrideParams},
        network::{EnableParams, Headers, SetExtraHttpHeadersParams},
        page::{CaptureScreenshotFormat, Viewport},
    },
    page::ScreenshotParams,
};
use futures::StreamExt;
use tokio::time::{Instant, sleep, timeout};

use crate::request::ScreenshotRequest;

pub async fn take_screenshot(req: &ScreenshotRequest) -> Result<Vec<u8>> {
    let user_data_dir = tempfile::Builder::new()
        .prefix("screenshot-chrome-")
        .tempdir()
        .context("create temporary Chromium profile")?;
    let config = browser_config(req, user_data_dir.path())?;
    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .context("launch Chromium browser")?;

    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(error) = event {
                tracing::debug!(?error, "Chromium event handler reported an error");
            }
        }
    });

    let result = run_in_browser(&browser, req).await;

    if let Err(error) = browser.close().await {
        tracing::debug!(?error, "failed to close Chromium cleanly");
    }
    if let Err(error) = browser.wait().await {
        tracing::debug!(?error, "failed to wait for Chromium process");
    }
    handler_task.abort();

    result
}

async fn run_in_browser(browser: &Browser, req: &ScreenshotRequest) -> Result<Vec<u8>> {
    let page = browser
        .new_page("about:blank")
        .await
        .context("open Chromium page")?;
    let request_timeout = Duration::from_secs(req.timeout as u64);

    let result = timeout(request_timeout, capture_page(&page, req))
        .await
        .map_err(|_| anyhow!("timed out after {} seconds", req.timeout))?;

    if let Err(error) = page.close().await {
        tracing::debug!(?error, "failed to close Chromium page cleanly");
    }

    result
}

async fn capture_page(page: &Page, req: &ScreenshotRequest) -> Result<Vec<u8>> {
    configure_page(page, req).await?;

    page.goto(req.url.as_str())
        .await
        .with_context(|| format!("navigate to {}", req.url))?;

    if !req.wait_for.trim().is_empty() {
        wait_for_visible_selector(
            page,
            req.wait_for.trim(),
            Duration::from_secs(req.timeout as u64),
        )
        .await?;
    }

    if req.wait_time > 0 {
        sleep(Duration::from_millis(req.wait_time as u64)).await;
    }

    if req.full_page {
        full_page_screenshot(page, req).await
    } else if let Some(clip) = &req.clip {
        page.screenshot(
            screenshot_builder(req)
                .clip(Viewport {
                    x: clip.x,
                    y: clip.y,
                    width: clip.width,
                    height: clip.height,
                    scale: 1.0,
                })
                .build(),
        )
        .await
        .context("capture clipped screenshot")
    } else {
        page.screenshot(screenshot_builder(req).build())
            .await
            .context("capture viewport screenshot")
    }
}

async fn configure_page(page: &Page, req: &ScreenshotRequest) -> Result<()> {
    page.execute(EnableParams::default())
        .await
        .context("enable Chromium network domain")?;

    if !req.headers.is_empty() {
        let headers = serde_json::to_value(&req.headers).context("serialize request headers")?;
        page.execute(SetExtraHttpHeadersParams::new(Headers::new(headers)))
            .await
            .context("set extra request headers")?;
    }

    if !req.user_agent.trim().is_empty() {
        page.set_user_agent(req.user_agent.trim())
            .await
            .context("set user agent")?;
    }

    page.execute(device_metrics(req.width, req.height, req))
        .await
        .context("set device metrics")?;

    Ok(())
}

async fn wait_for_visible_selector(
    page: &Page,
    selector: &str,
    request_timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + request_timeout;

    loop {
        if let Ok(element) = page.find_element(selector).await {
            if let Ok(bounds) = element.bounding_box().await {
                if bounds.width > 0.0 && bounds.height > 0.0 {
                    return Ok(());
                }
            }
        }

        if Instant::now() >= deadline {
            return Err(anyhow!("wait_for selector `{selector}` timed out"));
        }

        sleep(Duration::from_millis(100)).await;
    }
}

async fn full_page_screenshot(page: &Page, req: &ScreenshotRequest) -> Result<Vec<u8>> {
    let metrics = page.layout_metrics().await.context("read layout metrics")?;
    let width = metrics.css_content_size.width.ceil();
    let height = metrics.css_content_size.height.ceil().min(16_384.0);

    page.execute(full_page_device_metrics(width as i64, height as i64))
        .await
        .context("set full-page viewport")?;

    page.screenshot(
        screenshot_builder(req)
            .capture_beyond_viewport(true)
            .clip(Viewport {
                x: 0.0,
                y: 0.0,
                width,
                height,
                scale: 1.0,
            })
            .build(),
    )
    .await
    .context("capture full-page screenshot")
}

fn screenshot_builder(req: &ScreenshotRequest) -> chromiumoxide::page::ScreenshotParamsBuilder {
    let mut builder = ScreenshotParams::builder().format(chrome_format(&req.format));

    if matches!(req.format.as_str(), "jpeg" | "jpg" | "webp") {
        builder = builder.quality(req.quality);
    }

    builder
}

fn device_metrics(
    width: i64,
    height: i64,
    req: &ScreenshotRequest,
) -> SetDeviceMetricsOverrideParams {
    let orientation = if req.landscape {
        ScreenOrientation::new(ScreenOrientationType::LandscapePrimary, 90)
    } else {
        ScreenOrientation::new(ScreenOrientationType::PortraitPrimary, 0)
    };

    SetDeviceMetricsOverrideParams::builder()
        .width(width)
        .height(height)
        .device_scale_factor(req.device_scale)
        .mobile(req.mobile)
        .screen_orientation(orientation)
        .build()
        .expect("device metrics are validated before Chromium is launched")
}

fn full_page_device_metrics(width: i64, height: i64) -> SetDeviceMetricsOverrideParams {
    SetDeviceMetricsOverrideParams::builder()
        .width(width)
        .height(height)
        .device_scale_factor(1.0)
        .mobile(false)
        .build()
        .expect("full-page metrics come from Chromium layout metrics")
}

fn chrome_format(format: &str) -> CaptureScreenshotFormat {
    match format {
        "jpeg" | "jpg" => CaptureScreenshotFormat::Jpeg,
        "webp" => CaptureScreenshotFormat::Webp,
        _ => CaptureScreenshotFormat::Png,
    }
}

fn browser_config(req: &ScreenshotRequest, user_data_dir: &Path) -> Result<BrowserConfig> {
    let mut builder = BrowserConfig::builder()
        .new_headless_mode()
        .no_sandbox()
        .window_size(req.width as u32, req.height as u32)
        .request_timeout(Duration::from_secs(req.timeout as u64))
        .launch_timeout(Duration::from_secs(req.timeout.min(60) as u64))
        .user_data_dir(user_data_dir)
        .args([
            "disable-gpu",
            "disable-software-rasterizer",
            "safebrowsing-disable-auto-update",
        ]);

    if let Some(executable) = chrome_executable() {
        builder = builder.chrome_executable(executable);
    }

    builder.build().map_err(anyhow::Error::msg)
}

fn chrome_executable() -> Option<String> {
    ["CHROME_BIN", "CHROMIUM_BIN"]
        .into_iter()
        .filter_map(|key| env::var(key).ok())
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
        .or_else(|| {
            env::var("CHROME_PATH")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty() && Path::new(value).is_file())
        })
}
