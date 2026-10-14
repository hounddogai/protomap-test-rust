use super::*;

use acme_kit::text;

/// How screens and exports show a segment estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEstimateView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub name: String,
    pub contact_email: String,
    pub amount: String,
    pub currency: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_segment_estimate_view(item: &SegmentEstimate) -> SegmentEstimateView {
    SegmentEstimateView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        name: text::truncate(&item.name, 80),
        contact_email: text::truncate(&item.contact_email, 120),
        amount: format!("{:.2}", item.amount),
        currency: item.currency.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_estimate_views(items: &[SegmentEstimate]) -> Vec<SegmentEstimateView> {
    let mut views: Vec<SegmentEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_estimate_view(item));
    }
    views
}

pub fn sum_segment_estimate_view(view: &SegmentEstimateView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_estimate("code-890", "label-449");
        let view = to_segment_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
