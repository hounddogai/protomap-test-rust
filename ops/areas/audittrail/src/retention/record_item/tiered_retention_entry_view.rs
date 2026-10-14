use super::*;

use acme_kit::text;

/// How screens and exports show a tiered retention entry.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionEntryView {
    pub code_text: String,
    pub label_text: String,
    pub weight_text: String,
    pub quantity_text: String,
    pub ratio: String,
    pub pinned: String,
    pub grace_days: String,
    pub audience: String,
    pub port: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_tiered_retention_entry_view(item: &TieredRetentionEntry) -> TieredRetentionEntryView {
    TieredRetentionEntryView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        weight_text: format!("{:.2}", item.weight),
        quantity_text: item.quantity.to_string(),
        ratio: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        audience: item.audience.clone(),
        port: item.port.to_string(),
        principal: text::truncate(&item.principal, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_retention_entry_views(items: &[TieredRetentionEntry]) -> Vec<TieredRetentionEntryView> {
    let mut views: Vec<TieredRetentionEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_retention_entry_view(item));
    }
    views
}

pub fn sum_tiered_retention_entry_view(view: &TieredRetentionEntryView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_weight + (parsed_quantity as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_retention_entry("code-844", "label-962");
        let view = to_tiered_retention_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_retention_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
