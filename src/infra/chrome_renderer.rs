use crate::domain::render::DiagramRenderer;
use headless_chrome::{Browser, LaunchOptions};

pub struct ChromeRenderer {
    chromium_path: Option<String>,
}

impl ChromeRenderer {
    pub fn new(chromium_path: Option<String>) -> Self {
        Self { chromium_path }
    }
}

impl DiagramRenderer for ChromeRenderer {
    fn render_to_png(&self, xml: &str, scale: f64) -> anyhow::Result<Vec<u8>> {
        let mut builder = LaunchOptions::default_builder();
        if let Some(ref path) = self.chromium_path {
            builder = builder.path(Some(std::path::PathBuf::from(path)));
        }
        let options = builder
            .headless(true)
            .sandbox(false)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build launch options: {}", e))?;

        let browser = Browser::new(options)?;
        let tab = browser.new_tab()?;

        // Load draw.io's export3.html renderer
        tab.navigate_to("https://app.diagrams.net/export3.html")?;
        tab.wait_until_navigated()?;

        // Wait for mxUtils to be available
        tab.wait_for_element("body")?;

        // Execute rendering via draw.io's render() function
        let escaped_xml = xml.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            r#"
            (() => {{
                const doc = mxUtils.parseXml(`{}`);
                const dup = doc.documentElement.cloneNode(false);
                let child = doc.documentElement.firstChild;
                while (child) {{
                    if (child.nodeType === Node.ELEMENT_NODE) {{
                        dup.appendChild(child);
                        break;
                    }}
                    child = child.nextSibling;
                }}
                render({{
                    xml: dup.outerHTML,
                    format: 'png',
                    w: 0, h: 0,
                    border: 0,
                    bg: 'none',
                    scale: {},
                }});
            }})()
            "#,
            escaped_xml, scale,
        );
        tab.evaluate(&js, false)?;

        // Wait for rendering to complete
        tab.wait_for_element("#LoadingComplete")?;

        // Take full-page screenshot as PNG
        let screenshot = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;

        Ok(screenshot)
    }
}
