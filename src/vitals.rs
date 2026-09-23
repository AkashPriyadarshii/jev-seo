//! PageSpeed vitals for the crawl homepage. Free Google endpoint, no key.
//! Keyless quota is small; failures degrade to None and the report says so.
//! Lab numbers only unless Chrome field data exists for the URL.

use serde::{Deserialize, Serialize};

/// Lab thresholds from Google guidance. Engineering picks, not ranking law.
pub const LCP_MS: u64 = 2500;
/// CLS stored as thousandths (100 = 0.10) to stay integer-friendly.
pub const CLS_MILLI: u64 = 100;
pub const INP_MS: u64 = 200;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vitals {
    #[serde(default)]
    pub lcp_ms: Option<u64>,
    #[serde(default)]
    pub cls_milli: Option<u64>,
    #[serde(default)]
    pub inp_ms: Option<u64>,
    #[serde(default)]
    pub score: Option<u32>,
    #[serde(default)]
    pub field: bool,
}

fn num_at(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for k in path {
        cur = cur.get(*k)?;
    }
    cur.as_f64()
}

/// Fetch homepage vitals. Returns None (not Err) when PageSpeed is
/// unreachable or rate-limited so crawls keep working offline.
pub fn fetch_home_vitals(raw_url: &str) -> Option<Vitals> {
    let u = crate::paths::reject_private_url(raw_url).ok()?;
    if u.scheme() != "https" && u.scheme() != "http" {
        return None;
    }
    let api = format!(
        "https://www.googleapis.com/pagespeedonline/v5/runPagespeed?url={}&strategy=mobile&category=performance",
        url::form_urlencoded::byte_serialize(u.as_str().as_bytes()).collect::<String>()
    );
    let resp = ureq::get(&api)
        .set("User-Agent", crate::crawl::CRAWL_UA)
        .timeout(std::time::Duration::from_secs(12))
        .call()
        .ok()?;
    let v: serde_json::Value = resp.into_json().ok()?;
    let audits = v.get("lighthouseResult")?.get("audits")?;
    let finite = |x: f64| x.is_finite() && x >= 0.0;
    let lcp = num_at(audits, &["largest-contentful-paint", "numericValue"])
        .filter(|x| finite(*x))
        .map(|x| x as u64);
    let cls = num_at(audits, &["cumulative-layout-shift", "numericValue"])
        .filter(|x| finite(*x))
        .map(|x| (x * 1000.0).round() as u64);
    let inp = num_at(audits, &["interaction-to-next-paint", "numericValue"])
        .filter(|x| finite(*x))
        .map(|x| x as u64);
    let score = v
        .get("lighthouseResult")?
        .get("categories")?
        .get("performance")?
        .get("score")?
        .as_f64()
        .map(|s| (s * 100.0).round() as u32);
    let field = v
        .get("loadingExperience")
        .and_then(|le| le.get("metrics"))
        .and_then(|m| m.as_object())
        .map(|m| !m.is_empty())
        .unwrap_or(false);
    if lcp.is_none() && cls.is_none() && inp.is_none() && score.is_none() {
        return None;
    }
    Some(Vitals { lcp_ms: lcp, cls_milli: cls, inp_ms: inp, score, field })
}

pub fn cls_display(milli: u64) -> String {
    format!("{}.{:03}", milli / 1000, milli % 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_match_google_guidance() {
        assert_eq!(LCP_MS, 2500);
        assert_eq!(CLS_MILLI, 100);
        assert_eq!(INP_MS, 200);
    }

    #[test]
    fn cls_display_formats_thousandths() {
        assert_eq!(cls_display(120), "0.120");
        assert_eq!(cls_display(0), "0.000");
    }

    #[test]
    fn private_urls_refuse_before_network() {
        assert!(fetch_home_vitals("http://127.0.0.1/").is_none());
        assert!(fetch_home_vitals("not a url").is_none());
    }

    #[test]
    fn vitals_default_is_empty() {
        let v = Vitals::default();
        assert!(v.lcp_ms.is_none() && !v.field);
        let back: Vitals = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
        assert!(back.score.is_none());
    }
}
