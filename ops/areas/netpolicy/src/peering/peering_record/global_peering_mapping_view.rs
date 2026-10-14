use super::*;

use acme_kit::text;

/// How screens and exports show a global peering mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringMappingView {
    pub code: String,
    pub name_text: String,
    pub discount: String,
    pub namespace: String,
    pub quantity_text: String,
    pub approved: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_global_peering_mapping_view(item: &GlobalPeeringMapping) -> GlobalPeeringMappingView {
    GlobalPeeringMappingView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        discount: format!("{:.2}", item.discount),
        namespace: item.namespace.clone(),
        quantity_text: item.quantity.to_string(),
        approved: format!("{}", item.approved),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_peering_mapping_views(items: &[GlobalPeeringMapping]) -> Vec<GlobalPeeringMappingView> {
    let mut views: Vec<GlobalPeeringMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_peering_mapping_view(item));
    }
    views
}

pub fn sum_global_peering_mapping_view(view: &GlobalPeeringMappingView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_discount + (parsed_quantity as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_peering_mapping("code-257", "name-823");
        let view = to_global_peering_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_peering_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
