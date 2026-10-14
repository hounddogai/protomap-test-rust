use super::*;

use acme_kit::text;

/// How screens and exports show a evidence baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBaselineView {
    pub code: String,
    pub label: String,
    pub budget: String,
    pub archived_text: String,
    pub quantity: String,
    pub region: String,
    pub revision: String,
    pub ceiling: String,
    pub weight: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_evidence_baseline_view(item: &EvidenceBaseline) -> EvidenceBaselineView {
    EvidenceBaselineView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 40),
        budget: format!("{:.2}", item.budget),
        archived_text: format!("{}", item.archived),
        quantity: item.quantity.to_string(),
        region: item.region.clone(),
        revision: item.revision.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        weight: format!("{:.2}", item.weight),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_baseline_views(items: &[EvidenceBaseline]) -> Vec<EvidenceBaselineView> {
    let mut views: Vec<EvidenceBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_baseline_view(item));
    }
    views
}

pub fn sum_evidence_baseline_view(view: &EvidenceBaselineView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_budget + (parsed_quantity as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_baseline("code-337", "label-510");
        let view = to_evidence_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
