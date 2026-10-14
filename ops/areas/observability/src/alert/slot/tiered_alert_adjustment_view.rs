use super::*;

use acme_kit::text;

/// How screens and exports show a tiered alert adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAdjustmentView {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub rate_text: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_tiered_alert_adjustment_view(item: &TieredAlertAdjustment) -> TieredAlertAdjustmentView {
    TieredAlertAdjustmentView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        contact_email: item.contact_email.clone(),
        rate_text: format!("{:.2}", item.rate),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_alert_adjustment_views(items: &[TieredAlertAdjustment]) -> Vec<TieredAlertAdjustmentView> {
    let mut views: Vec<TieredAlertAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_alert_adjustment_view(item));
    }
    views
}

pub fn sum_tiered_alert_adjustment_view(view: &TieredAlertAdjustmentView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_alert_adjustment("code-328", "name-214");
        let view = to_tiered_alert_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_alert_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
