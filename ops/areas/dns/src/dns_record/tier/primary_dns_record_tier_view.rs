use super::*;

use acme_kit::text;

/// How screens and exports show a primary dns record tier.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordTierView {
    pub code_text: String,
    pub label: String,
    pub quantity_text: String,
    pub approved: String,
    pub visible_text: String,
    pub issuer: String,
    pub margin_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_primary_dns_record_tier_view(item: &PrimaryDnsRecordTier) -> PrimaryDnsRecordTierView {
    PrimaryDnsRecordTierView {
        code_text: text::truncate(&item.code, 80),
        label: item.label.clone(),
        quantity_text: item.quantity.to_string(),
        approved: format!("{}", item.approved),
        visible_text: format!("{}", item.visible),
        issuer: text::truncate(&item.issuer, 80),
        margin_text: format!("{:.2}", item.margin),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_dns_record_tier_views(items: &[PrimaryDnsRecordTier]) -> Vec<PrimaryDnsRecordTierView> {
    let mut views: Vec<PrimaryDnsRecordTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_dns_record_tier_view(item));
    }
    views
}

pub fn sum_primary_dns_record_tier_view(view: &PrimaryDnsRecordTierView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_dns_record_tier("code-844", "label-992");
        let view = to_primary_dns_record_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_dns_record_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
