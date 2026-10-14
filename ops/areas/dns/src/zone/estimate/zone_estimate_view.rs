use super::*;

use acme_kit::text;

/// How screens and exports show a zone estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateView {
    pub code_text: String,
    pub title: String,
    pub key_id: String,
    pub currency: String,
    pub sequence: String,
    pub window_days: String,
    pub budget: String,
    pub owner_id: String,
    pub threshold: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_zone_estimate_view(item: &ZoneEstimate) -> ZoneEstimateView {
    ZoneEstimateView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        key_id: item.key_id.clone(),
        currency: item.currency.clone(),
        sequence: item.sequence.to_string(),
        window_days: item.window_days.to_string(),
        budget: format!("{:.2}", item.budget),
        owner_id: text::truncate(&item.owner_id, 80),
        threshold: format!("{:.2}", item.threshold),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_estimate_views(items: &[ZoneEstimate]) -> Vec<ZoneEstimateView> {
    let mut views: Vec<ZoneEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_estimate_view(item));
    }
    views
}

pub fn sum_zone_estimate_view(view: &ZoneEstimateView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_window_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_estimate("code-329", "title-215");
        let view = to_zone_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
