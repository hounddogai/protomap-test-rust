use super::*;

use acme_kit::text;

/// How screens and exports show a nonce baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaselineView {
    pub code: String,
    pub title: String,
    pub ttl_seconds_text: String,
    pub fingerprint: String,
    pub budget: String,
    pub scope: String,
    pub threshold: String,
    pub description: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_nonce_baseline_view(item: &NonceBaseline) -> NonceBaselineView {
    NonceBaselineView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        budget: format!("{:.2}", item.budget),
        scope: text::truncate(&item.scope, 40),
        threshold: format!("{:.2}", item.threshold),
        description: item.description.clone(),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_baseline_views(items: &[NonceBaseline]) -> Vec<NonceBaselineView> {
    let mut views: Vec<NonceBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_baseline_view(item));
    }
    views
}

pub fn sum_nonce_baseline_view(view: &NonceBaselineView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_budget + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_baseline("code-940", "title-647");
        let view = to_nonce_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
