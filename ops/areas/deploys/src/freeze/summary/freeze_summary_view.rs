use super::*;

use acme_kit::text;

/// How screens and exports show a freeze summary.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSummaryView {
    pub code: String,
    pub label: String,
    pub rate_text: String,
    pub host_text: String,
    pub quota_units: String,
    pub maximum: String,
    pub budget_text: String,
    pub floor_value: String,
    pub fingerprint_text: String,
    pub discount_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_freeze_summary_view(item: &FreezeSummary) -> FreezeSummaryView {
    FreezeSummaryView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        rate_text: format!("{:.2}", item.rate),
        host_text: item.host.clone(),
        quota_units: item.quota_units.to_string(),
        maximum: format!("{:.2}", item.maximum),
        budget_text: format!("{:.2}", item.budget),
        floor_value: format!("{:.2}", item.floor_value),
        fingerprint_text: item.fingerprint.clone(),
        discount_text: format!("{:.2}", item.discount),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_summary_views(items: &[FreezeSummary]) -> Vec<FreezeSummaryView> {
    let mut views: Vec<FreezeSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_summary_view(item));
    }
    views
}

pub fn sum_freeze_summary_view(view: &FreezeSummaryView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + (parsed_quota_units as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_summary("code-562", "label-440");
        let view = to_freeze_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
