use super::*;

use acme_kit::text;

/// How screens and exports show a strict alert preference.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertPreferenceView {
    pub code: String,
    pub name_text: String,
    pub category: String,
    pub budget_text: String,
    pub burst: String,
    pub external_ref_text: String,
    pub rate: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_strict_alert_preference_view(item: &StrictAlertPreference) -> StrictAlertPreferenceView {
    StrictAlertPreferenceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        category: text::truncate(&item.category, 40),
        budget_text: format!("{:.2}", item.budget),
        burst: item.burst.to_string(),
        external_ref_text: item.external_ref.clone(),
        rate: format!("{:.2}", item.rate),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_alert_preference_views(items: &[StrictAlertPreference]) -> Vec<StrictAlertPreferenceView> {
    let mut views: Vec<StrictAlertPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_alert_preference_view(item));
    }
    views
}

pub fn sum_strict_alert_preference_view(view: &StrictAlertPreferenceView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_budget + (parsed_burst as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_alert_preference("code-343", "name-872");
        let view = to_strict_alert_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_alert_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
