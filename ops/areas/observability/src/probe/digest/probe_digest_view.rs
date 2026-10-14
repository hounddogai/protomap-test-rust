use super::*;

/// How screens and exports show a probe digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDigestView {
    pub code: String,
    pub name: String,
    pub category_text: String,
    pub grace_days: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_probe_digest_view(item: &ProbeDigest) -> ProbeDigestView {
    ProbeDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        category_text: item.category.clone(),
        grace_days: item.grace_days.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_digest_views(items: &[ProbeDigest]) -> Vec<ProbeDigestView> {
    let mut views: Vec<ProbeDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_digest_view(item));
    }
    views
}

pub fn sum_probe_digest_view(view: &ProbeDigestView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_budget)
}
