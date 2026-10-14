use super::*;

use acme_kit::text;

/// How screens and exports show a primary verifier target.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTargetView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub capacity_text: String,
    pub retries: String,
    pub priority: String,
    pub currency_text: String,
    pub region: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_primary_verifier_target_view(item: &PrimaryVerifierTarget) -> PrimaryVerifierTargetView {
    PrimaryVerifierTargetView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        capacity_text: item.capacity.to_string(),
        retries: item.retries.to_string(),
        priority: item.priority.to_string(),
        currency_text: text::truncate(&item.currency, 40),
        region: text::truncate(&item.region, 64),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_verifier_target_views(items: &[PrimaryVerifierTarget]) -> Vec<PrimaryVerifierTargetView> {
    let mut views: Vec<PrimaryVerifierTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_verifier_target_view(item));
    }
    views
}

pub fn sum_primary_verifier_target_view(view: &PrimaryVerifierTargetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_amount + (parsed_capacity as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_verifier_target("code-657", "name-40");
        let view = to_primary_verifier_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_verifier_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
