use super::*;

use acme_kit::text;

/// How screens and exports show a regional reservation mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationMappingView {
    pub code_text: String,
    pub name: String,
    pub maximum_text: String,
    pub tenant_id: String,
    pub port: String,
    pub weight: String,
    pub namespace: String,
    pub ttl_seconds: String,
    pub cost: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_regional_reservation_mapping_view(item: &RegionalReservationMapping) -> RegionalReservationMappingView {
    RegionalReservationMappingView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        tenant_id: item.tenant_id.clone(),
        port: item.port.to_string(),
        weight: format!("{:.2}", item.weight),
        namespace: item.namespace.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        cost: format!("{:.2}", item.cost),
        key_id: text::truncate(&item.key_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_reservation_mapping_views(
    items: &[RegionalReservationMapping],
) -> Vec<RegionalReservationMappingView> {
    let mut views: Vec<RegionalReservationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_reservation_mapping_view(item));
    }
    views
}

pub fn sum_regional_reservation_mapping_view(view: &RegionalReservationMappingView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_maximum + (parsed_port as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_reservation_mapping("code-479", "name-291");
        let view = to_regional_reservation_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_reservation_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
