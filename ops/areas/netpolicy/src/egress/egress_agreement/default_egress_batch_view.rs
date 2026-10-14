use super::*;

use acme_kit::text;

/// How screens and exports show a default egress batch.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressBatchView {
    pub code: String,
    pub label: String,
    pub fingerprint_text: String,
    pub discount: String,
    pub visible_text: String,
    pub region: String,
    pub archived: String,
    pub name: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_default_egress_batch_view(item: &DefaultEgressBatch) -> DefaultEgressBatchView {
    DefaultEgressBatchView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        fingerprint_text: item.fingerprint.clone(),
        discount: format!("{:.2}", item.discount),
        visible_text: format!("{}", item.visible),
        region: item.region.clone(),
        archived: format!("{}", item.archived),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_egress_batch_views(items: &[DefaultEgressBatch]) -> Vec<DefaultEgressBatchView> {
    let mut views: Vec<DefaultEgressBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_egress_batch_view(item));
    }
    views
}

pub fn sum_default_egress_batch_view(view: &DefaultEgressBatchView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_discount + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_egress_batch("code-35", "label-626");
        let view = to_default_egress_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_egress_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
