use super::*;

use acme_kit::text;

/// How screens and exports show a pending data export tier.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDataExportTierView {
    pub code: String,
    pub title: String,
    pub margin: String,
    pub timezone: String,
    pub quantity_text: String,
    pub discount: String,
    pub unit_text: String,
    pub description: String,
    pub tenant_id_text: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_pending_data_export_tier_view(item: &PendingDataExportTier) -> PendingDataExportTierView {
    PendingDataExportTierView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        timezone: item.timezone.clone(),
        quantity_text: item.quantity.to_string(),
        discount: format!("{:.2}", item.discount),
        unit_text: item.unit.clone(),
        description: text::truncate(&item.description, 120),
        tenant_id_text: item.tenant_id.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_data_export_tier_views(items: &[PendingDataExportTier]) -> Vec<PendingDataExportTierView> {
    let mut views: Vec<PendingDataExportTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_data_export_tier_view(item));
    }
    views
}

pub fn sum_pending_data_export_tier_view(view: &PendingDataExportTierView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_margin + (parsed_quantity as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_data_export_tier("code-107", "title-841");
        let view = to_pending_data_export_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_data_export_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
