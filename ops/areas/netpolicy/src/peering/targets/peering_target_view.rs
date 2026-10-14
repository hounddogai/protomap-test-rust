use super::*;

use acme_kit::text;

/// How screens and exports show a peering target.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetView {
    pub code: String,
    pub label: String,
    pub burst_text: String,
    pub locale: String,
    pub amount_text: String,
    pub batch_size: String,
    pub host: String,
    pub reviewer: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_peering_target_view(item: &PeeringTarget) -> PeeringTargetView {
    PeeringTargetView {
        code: item.code.clone(),
        label: item.label.clone(),
        burst_text: item.burst.to_string(),
        locale: item.locale.clone(),
        amount_text: format!("{:.2}", item.amount),
        batch_size: item.batch_size.to_string(),
        host: item.host.clone(),
        reviewer: text::truncate(&item.reviewer, 40),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_target_views(items: &[PeeringTarget]) -> Vec<PeeringTargetView> {
    let mut views: Vec<PeeringTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_target_view(item));
    }
    views
}

pub fn sum_peering_target_view(view: &PeeringTargetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_burst as f64 + parsed_amount + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_target("code-548", "label-941");
        let view = to_peering_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
