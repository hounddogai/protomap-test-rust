use super::*;

use acme_kit::text;

/// How screens and exports show a weighted alert entry.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertEntryView {
    pub code: String,
    pub title_text: String,
    pub maximum: String,
    pub tenant_id_text: String,
    pub attempts: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_weighted_alert_entry_view(item: &WeightedAlertEntry) -> WeightedAlertEntryView {
    WeightedAlertEntryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        tenant_id_text: item.tenant_id.clone(),
        attempts: item.attempts.to_string(),
        country_text: text::truncate(&item.country, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_alert_entry_views(items: &[WeightedAlertEntry]) -> Vec<WeightedAlertEntryView> {
    let mut views: Vec<WeightedAlertEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_alert_entry_view(item));
    }
    views
}

pub fn sum_weighted_alert_entry_view(view: &WeightedAlertEntryView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_maximum + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_alert_entry("code-599", "title-666");
        let view = to_weighted_alert_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_alert_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
