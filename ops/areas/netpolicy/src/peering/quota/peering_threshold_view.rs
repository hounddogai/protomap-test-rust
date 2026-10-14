use super::*;

use acme_kit::text;

/// How screens and exports show a peering threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdView {
    pub code: String,
    pub title: String,
    pub burst: String,
    pub key_id_text: String,
    pub quantity_text: String,
    pub window_days: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_peering_threshold_view(item: &PeeringThreshold) -> PeeringThresholdView {
    PeeringThresholdView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        burst: item.burst.to_string(),
        key_id_text: text::truncate(&item.key_id, 40),
        quantity_text: item.quantity.to_string(),
        window_days: item.window_days.to_string(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_threshold_views(items: &[PeeringThreshold]) -> Vec<PeeringThresholdView> {
    let mut views: Vec<PeeringThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_threshold_view(item));
    }
    views
}

pub fn sum_peering_threshold_view(view: &PeeringThresholdView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_quantity as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_threshold("code-365", "title-993");
        let view = to_peering_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
