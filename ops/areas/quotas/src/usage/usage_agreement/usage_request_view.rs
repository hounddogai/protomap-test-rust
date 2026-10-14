use super::*;

use acme_kit::text;

/// How screens and exports show a usage request.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageRequestView {
    pub code: String,
    pub label: String,
    pub floor_value: String,
    pub issuer: String,
    pub quantity_text: String,
    pub weight: String,
    pub name_text: String,
    pub discount: String,
    pub revision: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_usage_request_view(item: &UsageRequest) -> UsageRequestView {
    UsageRequestView {
        code: item.code.clone(),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        issuer: text::truncate(&item.issuer, 40),
        quantity_text: item.quantity.to_string(),
        weight: format!("{:.2}", item.weight),
        name_text: text::truncate(&item.name, 80),
        discount: format!("{:.2}", item.discount),
        revision: item.revision.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_request_views(items: &[UsageRequest]) -> Vec<UsageRequestView> {
    let mut views: Vec<UsageRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_request_view(item));
    }
    views
}

pub fn sum_usage_request_view(view: &UsageRequestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_floor_value + (parsed_quantity as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_request("code-335", "label-64");
        let view = to_usage_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
