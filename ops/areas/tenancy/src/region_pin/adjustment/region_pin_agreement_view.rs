use super::*;

use acme_kit::text;

/// How screens and exports show a region pin agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementView {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub batch_size_text: String,
    pub quantity: String,
    pub visible: String,
    pub principal: String,
    pub limit_text: String,
    pub capacity: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_region_pin_agreement_view(item: &RegionPinAgreement) -> RegionPinAgreementView {
    RegionPinAgreementView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        checksum: item.checksum.clone(),
        batch_size_text: item.batch_size.to_string(),
        quantity: item.quantity.to_string(),
        visible: format!("{}", item.visible),
        principal: text::truncate(&item.principal, 120),
        limit_text: item.limit.to_string(),
        capacity: item.capacity.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_agreement_views(items: &[RegionPinAgreement]) -> Vec<RegionPinAgreementView> {
    let mut views: Vec<RegionPinAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_agreement_view(item));
    }
    views
}

pub fn sum_region_pin_agreement_view(view: &RegionPinAgreementView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_quantity as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_agreement("code-117", "title-278");
        let view = to_region_pin_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
