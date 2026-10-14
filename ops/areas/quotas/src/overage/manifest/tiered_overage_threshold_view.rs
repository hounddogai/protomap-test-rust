use super::*;

use acme_kit::text;

/// How screens and exports show a tiered overage threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdView {
    pub code: String,
    pub name_text: String,
    pub cost: String,
    pub rate_text: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_tiered_overage_threshold_view(item: &TieredOverageThreshold) -> TieredOverageThresholdView {
    TieredOverageThresholdView {
        code: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 120),
        cost: format!("{:.2}", item.cost),
        rate_text: format!("{:.2}", item.rate),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_overage_threshold_views(items: &[TieredOverageThreshold]) -> Vec<TieredOverageThresholdView> {
    let mut views: Vec<TieredOverageThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_overage_threshold_view(item));
    }
    views
}

pub fn sum_tiered_overage_threshold_view(view: &TieredOverageThresholdView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_cost + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_overage_threshold("code-46", "name-195");
        let view = to_tiered_overage_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_overage_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
