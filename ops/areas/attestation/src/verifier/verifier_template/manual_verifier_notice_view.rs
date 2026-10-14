use super::*;

use acme_kit::text;

/// How screens and exports show a manual verifier notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierNoticeView {
    pub code: String,
    pub title_text: String,
    pub contact_email: String,
    pub starts_at: String,
    pub score: String,
    pub segment: String,
    pub rate: String,
    pub summary: String,
    pub floor_value_text: String,
    pub weight: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_manual_verifier_notice_view(item: &ManualVerifierNotice) -> ManualVerifierNoticeView {
    ManualVerifierNoticeView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        contact_email: item.contact_email.clone(),
        starts_at: item.starts_at.to_string(),
        score: format!("{:.2}", item.score),
        segment: item.segment.clone(),
        rate: format!("{:.2}", item.rate),
        summary: item.summary.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        weight: format!("{:.2}", item.weight),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_verifier_notice_views(items: &[ManualVerifierNotice]) -> Vec<ManualVerifierNoticeView> {
    let mut views: Vec<ManualVerifierNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_verifier_notice_view(item));
    }
    views
}

pub fn sum_manual_verifier_notice_view(view: &ManualVerifierNoticeView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_score + parsed_rate)
}
