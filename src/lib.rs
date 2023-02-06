use anyhow::Result;
use serde_json::{ json, Value };
use std::io::Read;
use thirtyfour::{ extensions::cdp::ChromeDevTools, prelude::* };

pub async fn activate_stealth(
    driver: &WebDriver,
    user_agent: Option<&str>,
    languages: Option<Vec<&str>>,
    vendor: Option<&str>,
    platform: Option<&str>,
    webgl_vendor: Option<&str>,
    renderer: Option<&str>,
    fix_hairline: Option<bool>,
    run_on_insecure_origins: Option<bool>,
) -> Result<()> {
    let chrome = ChromeDevTools::new(driver.handle.clone());

    with_utils(&chrome).await?;
    chrome_app(&chrome).await?;
    chrome_runtime(&chrome, &run_on_insecure_origins).await?;
    iframe_content_window(&chrome).await?;
    media_codecs(&chrome).await?;
    navigator_languages(&chrome, languages.clone()).await?;
    navigator_permissions(&chrome).await?;
    navigator_plugins(&chrome).await?;
    navigator_vendor(&chrome, &vendor).await?;
    navigator_webdriver(&chrome).await?;
    user_agent_override(&chrome, &user_agent, languages.clone(), &platform).await?;
    webgl_vendor_override(&chrome, &webgl_vendor, &renderer).await?;
    window_outerdimensions(&chrome).await?;

    let fix_hairline = fix_hairline.unwrap_or(true);
    if fix_hairline {
        hairline_fix(&chrome).await?;
    }

    Ok(())
}

async fn with_utils(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/utils.js", None).await?;
    Ok(())
}

async fn chrome_app(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/chrome.app.js", None).await?;
    Ok(())
}

async fn chrome_runtime(
    chrome: &ChromeDevTools,
    run_on_insecure_origins: &Option<bool>,
) -> Result<()> {
    let run_on_insecure_origins = run_on_insecure_origins.unwrap_or(false);

    evaluate_on_new_document(
        chrome,
        "js/chrome.runtime.js",
        Some(vec![json!(run_on_insecure_origins)]),
    )
    .await?;
    Ok(())
}

async fn iframe_content_window(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/iframe.contentWindow.js", None).await?;
    Ok(())
}

async fn media_codecs(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/media.codecs.js", None).await?;
    Ok(())
}

async fn navigator_languages(
    chrome: &ChromeDevTools, 
    languages: Option<Vec<&str>>
) -> Result<()> {
    let languages = languages.unwrap_or(vec!["en-US", "en"]);
    let args = languages
        .into_iter()
        .map(|x| json!(x))
        .collect::<Vec<Value>>();

    evaluate_on_new_document(chrome, "js/navigator.languages.js", Some(args)).await?;
    Ok(())
}

async fn navigator_permissions(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/navigator.permissions.js", None).await?;
    Ok(())
}

async fn navigator_plugins(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/navigator.plugins.js", None).await?;
    Ok(())
}

async fn navigator_vendor(
    chrome: &ChromeDevTools, 
    vendor: &Option<&str>
) -> Result<()> {
    let vendor = vendor.unwrap_or("Google Inc.");
    let args = json!(vendor);
    evaluate_on_new_document(chrome, "js/navigator.vendor.js", Some(vec![args])).await?;
    Ok(())
}

async fn navigator_webdriver(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/navigator.webdriver.js", None).await?;
    Ok(())
}

async fn user_agent_override(
    chrome: &ChromeDevTools,
    user_agent: &Option<&str>,
    languages: Option<Vec<&str>>,
    platform: &Option<&str>,
) -> Result<()> {
    let languages = languages.unwrap_or(vec!["en-US", "en"]);
    let ua_language = languages.join(",");

    let platform = platform.unwrap_or("None");

    let mut user_agent = match user_agent {
        Some(x) => x.to_string(),
        None => {
            let result = chrome.execute_cdp("Browser.getVersion").await?;
            result.get("userAgent").unwrap().to_string()
        }
    };

    user_agent = user_agent.replace("HeadlessChrome", "Chrome");
    user_agent = format!("({})", user_agent.trim());

    chrome
        .execute_cdp_with_params(
            "Network.setUserAgentOverride",
            json!({
                "userAgent": json!(user_agent),
                "acceptLanguage": ua_language,
                "platform": platform
            }),
        )
        .await?;

    Ok(())
}

async fn webgl_vendor_override(
    chrome: &ChromeDevTools,
    webgl_vendor: &Option<&str>,
    renderer: &Option<&str>,
) -> Result<()> {
    let webgl_vendor = webgl_vendor.unwrap_or("Intel Inc.");
    let renderer = renderer.unwrap_or("Intel Iris OpenGL Engine");

    evaluate_on_new_document(
        chrome,
        "js/webgl.vendor.js",
        Some(vec![json!(webgl_vendor), json!(renderer)]),
    )
    .await?;
    Ok(())
}

async fn window_outerdimensions(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/window.outerdimensions.js", None).await?;
    Ok(())
}

async fn hairline_fix(chrome: &ChromeDevTools) -> Result<()> {
    evaluate_on_new_document(chrome, "js/hairline.fix.js", None).await?;
    Ok(())
}

async fn evaluate_on_new_document(
    chrome: &ChromeDevTools,
    path: &str,
    args: Option<Vec<Value>>,
) -> Result<()> {
    let js = read_file(path).expect(&format!("failed reading {}", path));
    let code = evaluate(&js, &args)?;

    chrome
        .execute_cdp_with_params(
            "Page.addScriptToEvaluateOnNewDocument",
            json!({ "source": code }),
        )
        .await?;
    Ok(())
}

fn read_file(path: &str) -> Result<String> {
    let mut file = std::fs::File::open(path).expect(&format!("Failed reading {}", path));
    let mut result = String::new();
    file.read_to_string(&mut result)?;

    Ok(result)
}

fn evaluate(code: &str, maybe_args: &Option<Vec<Value>>) -> Result<String> {
    let args = match maybe_args {
        Some(args) => args
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(","),
        None => "".to_string(),
    };

    Ok(format!("({})({})", code, args))
}
