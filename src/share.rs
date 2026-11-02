//! Share links: a finished export goes to jaeswift.xyz/daisycap/<handle>/<video>
//! and the link goes on the clipboard. The server (server/share) keeps each
//! handle's token as a hash, so only the Mac that claimed a handle can upload
//! to it or take a video down.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

pub const API: &str = "https://jaeswift.xyz/daisycap/api/share";
pub const SITE: &str = "https://jaeswift.xyz/daisycap";

/// The same names the server refuses as handles.
const RESERVED: [&str; 9] = ["api", "daisycap", "admin", "www", "wp-admin", "static", "assets", "login", "downloads"];

/// A handle is the `<user>` in a link: 3 to 30 letters, digits and single hyphens.
pub fn valid_handle(h: &str) -> bool {
    let b = h.as_bytes();
    (3..=30).contains(&b.len())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
        && !h.starts_with('-')
        && !h.ends_with('-')
        && !h.contains("--")
        && !RESERVED.contains(&h)
}

/// The page for a shared video.
pub fn page_url(handle: &str, slug: &str) -> String {
    format!("{SITE}/{handle}/{slug}")
}

/// What the video's page shows about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Details {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub duration_ms: f64,
}

/// A video that is up.
#[derive(Clone, Debug, PartialEq)]
pub struct Shared {
    pub handle: String,
    pub slug: String,
    pub url: String,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(120))
        .build()
}

#[derive(Deserialize, Default)]
struct Reply {
    token: Option<String>,
    slug: Option<String>,
    url: Option<String>,
}

/// The server's answer, or its reason for refusing, as a message a person can read.
fn reply(res: Result<ureq::Response, ureq::Error>) -> Result<Reply> {
    match res {
        Ok(r) => Ok(serde_json::from_str(&r.into_string()?)?),
        Err(ureq::Error::Status(code, r)) => {
            let msg = r
                .into_string()
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v["error"].as_str().map(str::to_owned))
                .unwrap_or_else(|| "the server refused it".into());
            bail!("{msg} (HTTP {code})")
        }
        Err(e) => bail!("couldn't reach jaeswift.xyz: {e}"),
    }
}

/// Claim a handle. A new handle comes back with its token, the only time the
/// server sends one. An existing handle is confirmed with its token.
pub fn claim(handle: &str, token: Option<&str>) -> Result<Option<String>> {
    let mut req = agent().post(&format!("{API}/handle.php")).set("Content-Type", "application/json");
    if let Some(t) = token {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }
    let body = serde_json::json!({ "handle": handle }).to_string();
    Ok(reply(req.send_string(&body))?.token)
}

/// Upload an export (MP4, MOV or GIF) under a claimed handle. `sent` counts the
/// bytes as they go; setting `cancel` stops the upload.
pub fn upload_video(handle: &str, token: &str, d: &Details, file: &Path, sent: &AtomicU64, cancel: &AtomicBool) -> Result<Shared> {
    let len = std::fs::metadata(file)?.len();
    let body = Counted { inner: File::open(file)?, sent, cancel };
    let kind = if file.extension().is_some_and(|e| e.eq_ignore_ascii_case("gif")) { "image/gif" } else { "video/mp4" };
    let res = agent()
        .post(&format!("{API}/upload.php"))
        .timeout(Duration::from_secs(4 * 3600))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Share-Handle", handle)
        .set("X-Share-Kind", "video")
        .set("X-Share-Title", &encode(&d.title))
        .set("X-Share-Width", &d.width.to_string())
        .set("X-Share-Height", &d.height.to_string())
        .set("X-Share-Duration-Ms", &(d.duration_ms.round().max(0.0) as u64).to_string())
        .set("Content-Type", kind)
        .set("Content-Length", &len.to_string())
        .send(body);
    let r = reply(res)?;
    Ok(Shared {
        handle: handle.to_owned(),
        slug: r.slug.context("the server didn't say where the video went")?,
        url: r.url.context("the server didn't say where the video went")?,
    })
}

/// Attach a JPEG poster to one of the handle's videos.
pub fn upload_poster(handle: &str, token: &str, slug: &str, jpeg: &[u8]) -> Result<()> {
    let res = agent()
        .post(&format!("{API}/upload.php"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Share-Handle", handle)
        .set("X-Share-Kind", "poster")
        .set("X-Share-Slug", slug)
        .set("Content-Type", "image/jpeg")
        .send_bytes(jpeg);
    reply(res).map(|_| ())
}

/// Take one of the handle's videos down.
pub fn remove(handle: &str, token: &str, slug: &str) -> Result<()> {
    let body = serde_json::json!({ "handle": handle, "slug": slug }).to_string();
    let res = agent()
        .post(&format!("{API}/delete.php"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .send_string(&body);
    reply(res).map(|_| ())
}

/// Percent-encode a header value: every UTF-8 byte outside the unreserved set.
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// A file that counts the bytes read from it, and fails once cancelled.
struct Counted<'a> {
    inner: File,
    sent: &'a AtomicU64,
    cancel: &'a AtomicBool,
}

impl Read for Counted<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(io::Error::other("cancelled"));
        }
        let n = self.inner.read(buf)?;
        self.sent.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_follow_the_server_rules() {
        for ok in ["jae", "my-link", "dc-check-1", "a1b"] {
            assert!(valid_handle(ok), "{ok}");
        }
        for bad in ["ab", "Jae", "-jae", "jae-", "my--link", "api", "daisycap", "downloads", "has space"] {
            assert!(!valid_handle(bad), "{bad}");
        }
        assert!(!valid_handle(&"x".repeat(31)), "too long");
    }

    #[test]
    fn links_name_the_handle_and_the_video() {
        assert_eq!(page_url("jae", "hello-8bbbbf"), "https://jaeswift.xyz/daisycap/jae/hello-8bbbbf");
    }

    #[test]
    fn titles_are_percent_encoded_for_headers() {
        assert_eq!(encode("Hi there"), "Hi%20there");
        assert_eq!(encode("ü"), "%C3%BC");
        assert_eq!(encode("a-b_c.d~e"), "a-b_c.d~e");
    }
}
