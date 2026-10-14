use super::*;

use acme_kit::text;

/// How screens and exports show a fallback probe tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeTierView {
    pub code_text: String,
    pub name: String,
    pub discount_text: String,
    pub quota_units: String,
    pub active_text: String,
    pub expires_at_text: String,
    pub label_text: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_fallback_probe_tier_view(item: &FallbackProbeTier) -> FallbackProbeTierView {
    FallbackProbeTierView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        discount_text: format!("{:.2}", item.discount),
        quota_units: item.quota_units.to_string(),
        active_text: format!("{}", item.active),
        expires_at_text: item.expires_at.to_string(),
        label_text: text::truncate(&item.label, 120),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_probe_tier_views(items: &[FallbackProbeTier]) -> Vec<FallbackProbeTierView> {
    let mut views: Vec<FallbackProbeTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_probe_tier_view(item));
    }
    views
}

pub fn sum_fallback_probe_tier_view(view: &FallbackProbeTierView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_discount + (parsed_quota_units as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_probe_tier("code-882", "name-699");
        let view = to_fallback_probe_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_probe_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
