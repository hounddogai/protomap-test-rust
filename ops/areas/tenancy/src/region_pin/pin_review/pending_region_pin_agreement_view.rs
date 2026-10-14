use super::*;

use acme_kit::text;

/// How screens and exports show a pending region pin agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementView {
    pub code_text: String,
    pub title: String,
    pub notes: String,
    pub category: String,
    pub country: String,
    pub amount: String,
    pub batch_size: String,
    pub host: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_pending_region_pin_agreement_view(item: &PendingRegionPinAgreement) -> PendingRegionPinAgreementView {
    PendingRegionPinAgreementView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        notes: item.notes.clone(),
        category: item.category.clone(),
        country: text::truncate(&item.country, 64),
        amount: format!("{:.2}", item.amount),
        batch_size: item.batch_size.to_string(),
        host: item.host.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_region_pin_agreement_views(
    items: &[PendingRegionPinAgreement],
) -> Vec<PendingRegionPinAgreementView> {
    let mut views: Vec<PendingRegionPinAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_region_pin_agreement_view(item));
    }
    views
}

pub fn sum_pending_region_pin_agreement_view(view: &PendingRegionPinAgreementView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_amount + (parsed_batch_size as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_region_pin_agreement("code-507", "title-228");
        let view = to_pending_region_pin_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_region_pin_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
