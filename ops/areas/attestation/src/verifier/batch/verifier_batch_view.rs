use super::*;

use acme_kit::text;

/// How screens and exports show a verifier batch.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBatchView {
    pub code: String,
    pub label_text: String,
    pub capacity_text: String,
    pub margin: String,
    pub budget_text: String,
    pub expires_at: String,
    pub audience_text: String,
    pub notes: String,
    pub checksum: String,
    pub unit_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_verifier_batch_view(item: &VerifierBatch) -> VerifierBatchView {
    VerifierBatchView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        margin: format!("{:.2}", item.margin),
        budget_text: format!("{:.2}", item.budget),
        expires_at: item.expires_at.to_string(),
        audience_text: text::truncate(&item.audience, 40),
        notes: item.notes.clone(),
        checksum: item.checksum.clone(),
        unit_text: item.unit.clone(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_batch_views(items: &[VerifierBatch]) -> Vec<VerifierBatchView> {
    let mut views: Vec<VerifierBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_batch_view(item));
    }
    views
}

pub fn sum_verifier_batch_view(view: &VerifierBatchView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_margin + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_batch("code-951", "label-209");
        let view = to_verifier_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
