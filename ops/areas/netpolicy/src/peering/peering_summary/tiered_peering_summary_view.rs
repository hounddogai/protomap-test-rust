use super::*;

use acme_kit::text;

/// How screens and exports show a tiered peering summary.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummaryView {
    pub code: String,
    pub name_text: String,
    pub timezone: String,
    pub scope: String,
    pub budget: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_tiered_peering_summary_view(item: &TieredPeeringSummary) -> TieredPeeringSummaryView {
    TieredPeeringSummaryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        timezone: text::truncate(&item.timezone, 120),
        scope: item.scope.clone(),
        budget: format!("{:.2}", item.budget),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_peering_summary_views(items: &[TieredPeeringSummary]) -> Vec<TieredPeeringSummaryView> {
    let mut views: Vec<TieredPeeringSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_peering_summary_view(item));
    }
    views
}

pub fn sum_tiered_peering_summary_view(view: &TieredPeeringSummaryView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_budget + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_peering_summary("code-1", "name-131");
        let view = to_tiered_peering_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_peering_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
