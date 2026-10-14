use super::*;

use acme_kit::text;

/// How screens and exports show a slo estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SloEstimateView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub minimum: String,
    pub starts_at: String,
    pub discount: String,
    pub reviewer_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_slo_estimate_view(item: &SloEstimate) -> SloEstimateView {
    SloEstimateView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        amount: format!("{:.2}", item.amount),
        minimum: format!("{:.2}", item.minimum),
        starts_at: item.starts_at.to_string(),
        discount: format!("{:.2}", item.discount),
        reviewer_text: item.reviewer.clone(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_estimate_views(items: &[SloEstimate]) -> Vec<SloEstimateView> {
    let mut views: Vec<SloEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_estimate_view(item));
    }
    views
}

pub fn sum_slo_estimate_view(view: &SloEstimateView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_amount + parsed_minimum + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_estimate("code-26", "name-839");
        let view = to_slo_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
