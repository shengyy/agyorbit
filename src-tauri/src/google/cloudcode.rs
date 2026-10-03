//! Cloud Code endpoints Antigravity uses for quota and subscription tier.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// The endpoint Antigravity 2.19.1 passes to its language server
/// (`--cloud_code_endpoint`).
const BASE: &str = "https://daily-cloudcode-pa.googleapis.com/v1internal";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupKey {
    Gemini,
    Claude,
    Other,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    /// 0.0–1.0 share of the window still available.
    pub remaining: f64,
    pub reset_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaGroup {
    pub key: GroupKey,
    pub label: String,
    pub models: Vec<String>,
    pub five_hour: Option<QuotaWindow>,
    pub weekly: Option<QuotaWindow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanKind {
    Free,
    Pro,
    Ultra,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub kind: PlanKind,
    pub name: String,
}

#[derive(Deserialize)]
struct SummaryResponse {
    #[serde(default)]
    groups: Vec<RawGroup>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGroup {
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    buckets: Vec<RawBucket>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBucket {
    #[serde(default)]
    window: String,
    /// proto3 JSON omits zero values, so a missing fraction means exhausted.
    #[serde(default)]
    remaining_fraction: f64,
    #[serde(default)]
    reset_time: Option<DateTime<Utc>>,
}

async fn post<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    access_token: &str,
    method: &'static str,
    body: serde_json::Value,
) -> Result<T> {
    let response = http
        .post(format!("{BASE}:{method}"))
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        return Err(Error::Http {
            endpoint: method,
            status: status.as_u16(),
        });
    }
    Ok(response.json().await?)
}

/// The 5-hour and weekly buckets of every model group.
pub async fn quota(http: &reqwest::Client, access_token: &str) -> Result<Vec<QuotaGroup>> {
    let summary: SummaryResponse = post(
        http,
        access_token,
        "retrieveUserQuotaSummary",
        serde_json::json!({}),
    )
    .await?;
    Ok(summary.groups.into_iter().map(parse_group).collect())
}

fn parse_group(group: RawGroup) -> QuotaGroup {
    let lower = group.display_name.to_lowercase();
    let key = if lower.contains("gemini") {
        GroupKey::Gemini
    } else if lower.contains("claude") {
        GroupKey::Claude
    } else {
        GroupKey::Other
    };
    let label = match key {
        GroupKey::Gemini => "Gemini".to_owned(),
        GroupKey::Claude => "Claude".to_owned(),
        GroupKey::Other => group.display_name.clone(),
    };
    // "Models within this group: Claude Opus, Claude Sonnet, GPT-OSS"
    let models = group
        .description
        .split_once(':')
        .map(|(_, list)| {
            list.split(',')
                .map(|m| m.trim().to_owned())
                .filter(|m| !m.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let window = |name: &str| {
        group
            .buckets
            .iter()
            .find(|b| b.window == name)
            .map(|b| QuotaWindow {
                remaining: b.remaining_fraction.clamp(0.0, 1.0),
                reset_at: b.reset_time,
            })
    };
    QuotaGroup {
        key,
        label,
        models,
        five_hour: window("5h"),
        weekly: window("weekly"),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadResponse {
    #[serde(default)]
    paid_tier: Option<Tier>,
    #[serde(default)]
    current_tier: Option<Tier>,
}

#[derive(Deserialize)]
struct Tier {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
}

/// The Google AI subscription behind the account.
pub async fn plan(http: &reqwest::Client, access_token: &str) -> Result<Plan> {
    let load: LoadResponse = post(
        http,
        access_token,
        "loadCodeAssist",
        serde_json::json!({ "metadata": { "ideType": "ANTIGRAVITY" } }),
    )
    .await?;
    Ok(parse_plan(load))
}

fn parse_plan(load: LoadResponse) -> Plan {
    match load.paid_tier.or(load.current_tier) {
        Some(tier) if tier.id.contains("ultra") => Plan {
            kind: PlanKind::Ultra,
            name: tier.name,
        },
        Some(tier) if tier.id.contains("pro") => Plan {
            kind: PlanKind::Pro,
            name: tier.name,
        },
        _ => Plan {
            kind: PlanKind::Free,
            name: "Free".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quota_summary() {
        let raw = r#"{"groups":[
          {"displayName":"Gemini Models","description":"Models within this group: Gemini Flash, Gemini Pro",
           "buckets":[{"window":"weekly","remainingFraction":0.96,"resetTime":"2026-10-10T09:23:35Z"},
                      {"window":"5h","remainingFraction":0.5,"resetTime":"2026-10-03T14:23:35Z"}]},
          {"displayName":"Claude and GPT models","description":"Models within this group: Claude Opus, Claude Sonnet, GPT-OSS",
           "buckets":[{"window":"5h","resetTime":"2026-10-03T14:23:35Z"}]}]}"#;
        let summary: SummaryResponse = serde_json::from_str(raw).unwrap();
        let groups: Vec<_> = summary.groups.into_iter().map(parse_group).collect();
        assert_eq!(groups[0].key, GroupKey::Gemini);
        assert_eq!(groups[0].five_hour.as_ref().unwrap().remaining, 0.5);
        assert_eq!(groups[0].weekly.as_ref().unwrap().remaining, 0.96);
        assert_eq!(groups[1].key, GroupKey::Claude);
        assert_eq!(
            groups[1].models,
            ["Claude Opus", "Claude Sonnet", "GPT-OSS"]
        );
        // proto3 omitted the zero fraction
        assert_eq!(groups[1].five_hour.as_ref().unwrap().remaining, 0.0);
        assert!(groups[1].weekly.is_none());
    }

    #[test]
    fn parses_plan_tiers() {
        let pro: LoadResponse =
            serde_json::from_str(r#"{"currentTier":{"id":"free-tier"},"paidTier":{"id":"g1-pro-tier","name":"Google AI Pro"}}"#)
                .unwrap();
        assert_eq!(
            parse_plan(pro),
            Plan {
                kind: PlanKind::Pro,
                name: "Google AI Pro".into()
            }
        );
        let free: LoadResponse =
            serde_json::from_str(r#"{"currentTier":{"id":"free-tier","name":"Antigravity"}}"#)
                .unwrap();
        assert_eq!(parse_plan(free).kind, PlanKind::Free);
    }
}
