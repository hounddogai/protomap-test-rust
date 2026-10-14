use super::*;

/// How screens and exports show a global alert rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertRollupView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub priority_text: String,
    pub version: String,
    pub quantity_text: String,
    pub limit_text: String,
    pub expires_at: String,
    pub threshold: String,
    pub ttl_seconds_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_global_alert_rollup_view(item: &GlobalAlertRollup) -> GlobalAlertRollupView {
    GlobalAlertRollupView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        priority_text: item.priority.to_string(),
        version: item.version.to_string(),
        quantity_text: item.quantity.to_string(),
        limit_text: item.limit.to_string(),
        expires_at: item.expires_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_alert_rollup_views(items: &[GlobalAlertRollup]) -> Vec<GlobalAlertRollupView> {
    let mut views: Vec<GlobalAlertRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_alert_rollup_view(item));
    }
    views
}

pub fn sum_global_alert_rollup_view(view: &GlobalAlertRollupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_alert_rollup("code-985", "title-473");
        let view = to_global_alert_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_alert_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
