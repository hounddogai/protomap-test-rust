use super::*;

use acme_kit::text;

/// How screens and exports show a batched allowlist draft.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistDraftView {
    pub code_text: String,
    pub name: String,
    pub weight: String,
    pub quota_units: String,
    pub channel: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_batched_allowlist_draft_view(item: &BatchedAllowlistDraft) -> BatchedAllowlistDraftView {
    BatchedAllowlistDraftView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        weight: format!("{:.2}", item.weight),
        quota_units: item.quota_units.to_string(),
        channel: item.channel.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_allowlist_draft_views(items: &[BatchedAllowlistDraft]) -> Vec<BatchedAllowlistDraftView> {
    let mut views: Vec<BatchedAllowlistDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_allowlist_draft_view(item));
    }
    views
}

pub fn sum_batched_allowlist_draft_view(view: &BatchedAllowlistDraftView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_weight + (parsed_quota_units as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_allowlist_draft("code-423", "name-605");
        let view = to_batched_allowlist_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_allowlist_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
