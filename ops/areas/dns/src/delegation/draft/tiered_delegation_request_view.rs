use super::*;

use acme_kit::text;

/// How screens and exports show a tiered delegation request.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequestView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub port_text: String,
    pub country: String,
    pub minimum: String,
    pub title_text: String,
    pub external_ref: String,
    pub audience_text: String,
    pub ends_at: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_tiered_delegation_request_view(item: &TieredDelegationRequest) -> TieredDelegationRequestView {
    TieredDelegationRequestView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        port_text: item.port.to_string(),
        country: item.country.clone(),
        minimum: format!("{:.2}", item.minimum),
        title_text: item.title.clone(),
        external_ref: item.external_ref.clone(),
        audience_text: item.audience.clone(),
        ends_at: item.ends_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_delegation_request_views(items: &[TieredDelegationRequest]) -> Vec<TieredDelegationRequestView> {
    let mut views: Vec<TieredDelegationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_delegation_request_view(item));
    }
    views
}

pub fn sum_tiered_delegation_request_view(view: &TieredDelegationRequestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_port as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_delegation_request("code-522", "name-171");
        let view = to_tiered_delegation_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_delegation_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
