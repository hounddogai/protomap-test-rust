use super::*;

/// How screens and exports show a pending federation override.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationOverrideView {
    pub code_text: String,
    pub label_text: String,
    pub margin: String,
    pub quota_units: String,
    pub reviewer: String,
    pub quantity: String,
    pub external_ref_text: String,
    pub grace_days: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_pending_federation_override_view(item: &PendingFederationOverride) -> PendingFederationOverrideView {
    PendingFederationOverrideView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        quota_units: item.quota_units.to_string(),
        reviewer: item.reviewer.clone(),
        quantity: item.quantity.to_string(),
        external_ref_text: item.external_ref.clone(),
        grace_days: item.grace_days.to_string(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_federation_override_views(items: &[PendingFederationOverride]) -> Vec<PendingFederationOverrideView> {
    let mut views: Vec<PendingFederationOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_federation_override_view(item));
    }
    views
}

pub fn sum_pending_federation_override_view(view: &PendingFederationOverrideView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_margin + (parsed_quota_units as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_federation_override("code-629", "label-495");
        let view = to_pending_federation_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_federation_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
