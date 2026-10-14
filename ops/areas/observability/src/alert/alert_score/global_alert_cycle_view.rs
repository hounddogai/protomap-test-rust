use super::*;

use acme_kit::text;

/// How screens and exports show a global alert cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleView {
    pub code: String,
    pub name_text: String,
    pub visible: String,
    pub timezone: String,
    pub ratio: String,
    pub key_id: String,
    pub quantity: String,
    pub currency_text: String,
    pub cluster: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_global_alert_cycle_view(item: &GlobalAlertCycle) -> GlobalAlertCycleView {
    GlobalAlertCycleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        visible: format!("{}", item.visible),
        timezone: item.timezone.clone(),
        ratio: format!("{:.2}", item.ratio),
        key_id: text::truncate(&item.key_id, 120),
        quantity: item.quantity.to_string(),
        currency_text: item.currency.clone(),
        cluster: item.cluster.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_alert_cycle_views(items: &[GlobalAlertCycle]) -> Vec<GlobalAlertCycleView> {
    let mut views: Vec<GlobalAlertCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_alert_cycle_view(item));
    }
    views
}

pub fn sum_global_alert_cycle_view(view: &GlobalAlertCycleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_ratio + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_alert_cycle("code-528", "name-836");
        let view = to_global_alert_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_alert_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
