use super::*;

use acme_kit::text;

/// How screens and exports show a rolling alert slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertSlotView {
    pub code: String,
    pub name_text: String,
    pub country_text: String,
    pub host: String,
    pub region: String,
    pub checksum: String,
    pub retries_text: String,
    pub key_id_text: String,
    pub maximum_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_rolling_alert_slot_view(item: &RollingAlertSlot) -> RollingAlertSlotView {
    RollingAlertSlotView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        country_text: item.country.clone(),
        host: text::truncate(&item.host, 40),
        region: text::truncate(&item.region, 80),
        checksum: text::truncate(&item.checksum, 80),
        retries_text: item.retries.to_string(),
        key_id_text: item.key_id.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_alert_slot_views(items: &[RollingAlertSlot]) -> Vec<RollingAlertSlotView> {
    let mut views: Vec<RollingAlertSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_alert_slot_view(item));
    }
    views
}

pub fn sum_rolling_alert_slot_view(view: &RollingAlertSlotView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_retries as f64 + parsed_maximum + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_alert_slot("code-123", "name-108");
        let view = to_rolling_alert_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_alert_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
