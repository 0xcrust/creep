use anyhow::Result;
use serde_json::json;
use std::path::Path;
use thirtyfour::{extensions::cdp::ChromeDevTools, DesiredCapabilities, WebDriver};

use stealth;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut opts = DesiredCapabilities::chrome();

    opts.add_chrome_arg("start-maximized")?;
    opts.add_chrome_option("excludeSwitches", ["enable-automation"])?;
    opts.add_chrome_option("useAutomationExtension", false)?;

    let driver = WebDriver::new("http://localhost:9515", opts).await?;
    stealth::activate(&driver, None, None, None, None, None, None, None, None).await?;

    let ua = driver
        .execute(r#"return navigator.userAgent;"#, vec![])
        .await?;
    log::info!("user agent: {:?}", ua);

    let url = "https://arh.antoinevastel.com/bots/areyouheadless";
    driver.goto(url).await?;

    let chrome = ChromeDevTools::new(driver.handle.clone());

    let metrics = chrome.execute_cdp("Page.getLayoutMetrics").await?;
    let content_size = metrics.get("contentSize").unwrap();
    let width = content_size.get("width").unwrap();
    let height = content_size.get("height").unwrap();

    log::info!("width: {:?}", width);
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

    driver
        .screenshot(Path::new("examples/results/hidden.png"))
        .await?;
    driver.quit().await?;

    Ok(())
}
