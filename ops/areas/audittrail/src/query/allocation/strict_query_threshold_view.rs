use super::*;

use acme_kit::text;

/// How screens and exports show a strict query threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdView {
    pub code: String,
    pub label: String,
    pub maximum_text: String,
    pub locale: String,
    pub quota_units: String,
    pub starts_at: String,
    pub floor_value: String,
    pub weight: String,
    pub batch_size_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_strict_query_threshold_view(item: &StrictQueryThreshold) -> StrictQueryThresholdView {
    StrictQueryThresholdView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        locale: item.locale.clone(),
        quota_units: item.quota_units.to_string(),
        starts_at: item.starts_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        weight: format!("{:.2}", item.weight),
        batch_size_text: item.batch_size.to_string(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_query_threshold_views(items: &[StrictQueryThreshold]) -> Vec<StrictQueryThresholdView> {
    let mut views: Vec<StrictQueryThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_query_threshold_view(item));
    }
    views
}

pub fn sum_strict_query_threshold_view(view: &StrictQueryThresholdView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_maximum + (parsed_quota_units as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_query_threshold("code-972", "label-656");
        let view = to_strict_query_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_query_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
