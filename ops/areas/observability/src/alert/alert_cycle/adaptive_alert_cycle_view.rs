use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive alert cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertCycleView {
    pub code_text: String,
    pub title: String,
    pub maximum_text: String,
    pub country: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_adaptive_alert_cycle_view(item: &AdaptiveAlertCycle) -> AdaptiveAlertCycleView {
    AdaptiveAlertCycleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        country: text::truncate(&item.country, 80),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_alert_cycle_views(items: &[AdaptiveAlertCycle]) -> Vec<AdaptiveAlertCycleView> {
    let mut views: Vec<AdaptiveAlertCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_alert_cycle_view(item));
    }
    views
}

pub fn sum_adaptive_alert_cycle_view(view: &AdaptiveAlertCycleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_maximum + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_alert_cycle("code-811", "title-845");
        let view = to_adaptive_alert_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_alert_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
