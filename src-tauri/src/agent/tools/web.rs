//! Read-only web tools: SearXNG-backed search and HTTP page scraping. Ported from
//! the legacy `WebBrowserLayer`. Both are non-sensitive (no host side effects).

use async_trait::async_trait;
use serde_json::{Value, json};

use super::{Tool, ToolContext};

pub struct WebSearch;

#[async_trait]
impl Tool for WebSearch {
    fn name(&self) -> &'static str {
        "web_search"
    }
    fn description(&self) -> &'static str {
        "Search the web (via SearXNG) and return the top results as markdown."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        })
    }
    async fn run(&self, args: &Value, ctx: &ToolContext) -> Result<String, String> {
        let query = args["query"].as_str().unwrap_or("").trim();
        if query.is_empty() {
            return Err("`query` is required".into());
        }
        let Some(base) = ctx.searxng_url.as_deref().filter(|s| !s.is_empty()) else {
            return Err("No SearXNG URL configured (set searxng_url in settings).".into());
        };
        let url = format!(
            "{}/search?q={}&format=json",
            base.trim_end_matches('/'),
            urlencoding::encode(query)
        );
        let client = reqwest::Client::new();
        let json = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("search request failed: {e}"))?
            .json::<Value>()
            .await
            .map_err(|e| format!("search returned non-JSON: {e}"))?;

        let mut out = String::new();
        if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
            for (i, r) in results.iter().take(5).enumerate() {
                let title = r.get("title").and_then(|t| t.as_str()).unwrap_or("");
                let link = r.get("url").and_then(|u| u.as_str()).unwrap_or("");
                let content = r.get("content").and_then(|c| c.as_str()).unwrap_or("");
                out.push_str(&format!("{}. [{}]({})\n   {}\n\n", i + 1, title, link, content));
            }
        }
        if out.is_empty() {
            Ok("No results found. Try different search terms.".into())
        } else {
            Ok(out)
        }
    }
}

pub struct WebScrape;

#[async_trait]
impl Tool for WebScrape {
    fn name(&self) -> &'static str {
        "web_scrape"
    }
    fn description(&self) -> &'static str {
        "Fetch a URL and return its readable text content (HTML stripped, truncated)."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "url": { "type": "string" } },
            "required": ["url"]
        })
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let url = args["url"].as_str().unwrap_or("").trim();
        if url.is_empty() {
            return Err("`url` is required".into());
        }

        // Support browser-harness, camofox, cloakbrowser if present on command line
        let has_harness = std::process::Command::new("browser-harness")
            .arg("--help")
            .output()
            .is_ok();
        let has_camofox = std::process::Command::new("camofox")
            .arg("--help")
            .output()
            .is_ok();
        let has_cloak = std::process::Command::new("cloakbrowser")
            .arg("--help")
            .output()
            .is_ok();

        if has_harness {
            if let Ok(out) = std::process::Command::new("browser-harness")
                .arg("--url")
                .arg(url)
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout).into_owned();
                    return Ok(truncate_text(text));
                }
            }
        } else if has_camofox {
            if let Ok(out) = std::process::Command::new("camofox")
                .arg("--url")
                .arg(url)
                .arg("--text-only")
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout).into_owned();
                    return Ok(truncate_text(text));
                }
            }
        } else if has_cloak {
            if let Ok(out) = std::process::Command::new("cloakbrowser")
                .arg("--url")
                .arg(url)
                .arg("--text-only")
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout).into_owned();
                    return Ok(truncate_text(text));
                }
            }
        }

        // Fallback standard HTTP request
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36")
            .build()
            .map_err(|e| e.to_string())?;
        let html = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("scrape failed: {e}"))?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let text: String = strip_html_tags(&html)
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(truncate_text(text))
    }
}

fn truncate_text(text: String) -> String {
    if text.len() > 10_000 {
        format!("{}... [truncated]", &text[..10_000])
    } else {
        text
    }
}

/// Minimal HTML→text: drops tags and the contents of <script>/<style>.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script_style = false;
    let mut current_tag = String::new();

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
            current_tag.clear();
        } else if c == '>' {
            in_tag = false;
            let t = current_tag.to_lowercase();
            if t.starts_with("script") || t.starts_with("style") {
                in_script_style = true;
            }
            if t.starts_with("/script") || t.starts_with("/style") {
                in_script_style = false;
            }
        } else if in_tag {
            current_tag.push(c);
        } else if !in_script_style {
            result.push(c);
        }
    }
    result
}
