use super::*;

use acme_kit::text;

/// How screens and exports show a legacy verifier estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierEstimateView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub rotation_days_text: String,
    pub starts_at: String,
    pub window_days: String,
    pub threshold_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_legacy_verifier_estimate_view(item: &LegacyVerifierEstimate) -> LegacyVerifierEstimateView {
    LegacyVerifierEstimateView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 64),
        currency: item.currency.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        starts_at: item.starts_at.to_string(),
        window_days: item.window_days.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_verifier_estimate_views(items: &[LegacyVerifierEstimate]) -> Vec<LegacyVerifierEstimateView> {
    let mut views: Vec<LegacyVerifierEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_verifier_estimate_view(item));
    }
    views
}

pub fn sum_legacy_verifier_estimate_view(view: &LegacyVerifierEstimateView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_starts_at as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_verifier_estimate("code-97", "label-200");
        let view = to_legacy_verifier_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_verifier_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
