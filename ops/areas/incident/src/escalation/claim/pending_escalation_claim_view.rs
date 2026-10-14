use super::*;

use acme_kit::text;

/// How screens and exports show a pending escalation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaimView {
    pub code_text: String,
    pub name: String,
    pub budget_text: String,
    pub sequence: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_pending_escalation_claim_view(item: &PendingEscalationClaim) -> PendingEscalationClaimView {
    PendingEscalationClaimView {
        code_text: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 120),
        budget_text: format!("{:.2}", item.budget),
        sequence: item.sequence.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_escalation_claim_views(items: &[PendingEscalationClaim]) -> Vec<PendingEscalationClaimView> {
    let mut views: Vec<PendingEscalationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_escalation_claim_view(item));
    }
    views
}

pub fn sum_pending_escalation_claim_view(view: &PendingEscalationClaimView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_budget + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_escalation_claim("code-379", "name-5");
        let view = to_pending_escalation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_escalation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
