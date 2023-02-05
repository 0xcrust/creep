#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::Result;
use serde_json::json;
use std::path::Path;
use thirtyfour::prelude::*;
use thirtyfour::{extensions::cdp::ChromeDevTools, By, DesiredCapabilities, WebDriver};
use tokio::time::{sleep, Duration};

mod stealth;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let x = "dance";
    log::info!("x => {}", x);

    let mut opts = DesiredCapabilities::chrome();
    log::info!("opts: {:?}\n\n", opts);
    opts.add_chrome_arg("start-maximized")?;
    opts.add_chrome_option("excludeSwitches", ["enable-automation"])?;
    opts.add_chrome_option("useAutomationExtension", false)?;
    log::info!("opts: {:?}\n\n", opts);

    let driver = WebDriver::new("http://localhost:9515", opts).await?;
    stealth::activate_stealth(&driver, None, None, None, None, None, None, None, None).await?;

    let ua = driver
        .execute(r#"return navigator.userAgent;"#, vec![])
        .await?;
    println!("user agent: {:?}", ua);
    let url = "https://bot.sannysoft.com";
    driver.goto(url).await?;

    let chrome = ChromeDevTools::new(driver.handle.clone());
    let metrics = chrome.execute_cdp("Page.getLayoutMetrics").await?;
    let content_size = metrics.get("contentSize").unwrap();
    let width = content_size.get("width").unwrap();
    log::info!("width: {:?}", width);
    let height = content_size.get("height").unwrap();
    log::info!("height: {:?}", height);
    chrome
        .execute_cdp_with_params(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "mobile": false,
                "width": width,
                "height": height,
                "deviceScaleFactor": 1,
                "screenOrientation": {
                    "angle": 0,
                    "type": "portraitPrimary"
                }
            }),
        )
        .await?;

    driver.screenshot(Path::new("result.png")).await?;
    driver.quit().await?;

    Ok(())
}
