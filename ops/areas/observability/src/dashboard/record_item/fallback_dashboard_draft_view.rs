use super::*;

use acme_kit::text;

/// How screens and exports show a fallback dashboard draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardDraftView {
    pub code: String,
    pub label: String,
    pub capacity_text: String,
    pub reviewer: String,
    pub owner_id: String,
    pub batch_size: String,
    pub weight: String,
    pub ttl_seconds: String,
    pub margin: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_fallback_dashboard_draft_view(item: &FallbackDashboardDraft) -> FallbackDashboardDraftView {
    FallbackDashboardDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        reviewer: text::truncate(&item.reviewer, 40),
        owner_id: item.owner_id.clone(),
        batch_size: item.batch_size.to_string(),
        weight: format!("{:.2}", item.weight),
        ttl_seconds: item.ttl_seconds.to_string(),
        margin: format!("{:.2}", item.margin),
        host: text::truncate(&item.host, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_dashboard_draft_views(items: &[FallbackDashboardDraft]) -> Vec<FallbackDashboardDraftView> {
    let mut views: Vec<FallbackDashboardDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_dashboard_draft_view(item));
    }
    views
}

pub fn sum_fallback_dashboard_draft_view(view: &FallbackDashboardDraftView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_batch_size as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_dashboard_draft("code-904", "label-707");
        let view = to_fallback_dashboard_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_dashboard_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
