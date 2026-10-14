use super::*;

/// How screens and exports show a alert checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpointView {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub discount: String,
    pub archived: String,
    pub capacity: String,
    pub minimum_text: String,
    pub cipher: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_alert_checkpoint_view(item: &AlertCheckpoint) -> AlertCheckpointView {
    AlertCheckpointView {
        code: item.code.clone(),
        label: item.label.clone(),
        namespace: item.namespace.clone(),
        discount: format!("{:.2}", item.discount),
        archived: format!("{}", item.archived),
        capacity: item.capacity.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        cipher: item.cipher.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_checkpoint_views(items: &[AlertCheckpoint]) -> Vec<AlertCheckpointView> {
    let mut views: Vec<AlertCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_checkpoint_view(item));
    }
    views
}

pub fn sum_alert_checkpoint_view(view: &AlertCheckpointView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_discount + (parsed_capacity as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_checkpoint("code-694", "label-708");
        let view = to_alert_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
