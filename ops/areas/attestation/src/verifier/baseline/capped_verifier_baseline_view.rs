use super::*;

use acme_kit::text;

/// How screens and exports show a capped verifier baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierBaselineView {
    pub code: String,
    pub title: String,
    pub amount: String,
    pub floor_value_text: String,
    pub audience_text: String,
    pub weight_text: String,
    pub reviewer: String,
    pub starts_at_text: String,
    pub archived: String,
    pub fingerprint: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_capped_verifier_baseline_view(item: &CappedVerifierBaseline) -> CappedVerifierBaselineView {
    CappedVerifierBaselineView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount: format!("{:.2}", item.amount),
        floor_value_text: format!("{:.2}", item.floor_value),
        audience_text: text::truncate(&item.audience, 120),
        weight_text: format!("{:.2}", item.weight),
        reviewer: item.reviewer.clone(),
        starts_at_text: item.starts_at.to_string(),
        archived: format!("{}", item.archived),
        fingerprint: item.fingerprint.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_verifier_baseline_views(items: &[CappedVerifierBaseline]) -> Vec<CappedVerifierBaselineView> {
    let mut views: Vec<CappedVerifierBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_verifier_baseline_view(item));
    }
    views
}

pub fn sum_capped_verifier_baseline_view(view: &CappedVerifierBaselineView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_amount + parsed_floor_value + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_verifier_baseline("code-371", "title-369");
        let view = to_capped_verifier_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_verifier_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
