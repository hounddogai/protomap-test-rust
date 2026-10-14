use super::*;

/// How screens and exports show a tiered region pin agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinAgreementView {
    pub code: String,
    pub title_text: String,
    pub visible_text: String,
    pub cluster_text: String,
    pub attempts: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_tiered_region_pin_agreement_view(item: &TieredRegionPinAgreement) -> TieredRegionPinAgreementView {
    TieredRegionPinAgreementView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        visible_text: format!("{}", item.visible),
        cluster_text: item.cluster.clone(),
        attempts: item.attempts.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_region_pin_agreement_views(items: &[TieredRegionPinAgreement]) -> Vec<TieredRegionPinAgreementView> {
    let mut views: Vec<TieredRegionPinAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_region_pin_agreement_view(item));
    }
    views
}

pub fn sum_tiered_region_pin_agreement_view(view: &TieredRegionPinAgreementView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_attempts as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_region_pin_agreement("code-69", "title-351");
        let view = to_tiered_region_pin_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_region_pin_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
