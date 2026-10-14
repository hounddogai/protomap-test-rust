use super::*;

/// How screens and exports show a peering agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAgreementView {
    pub code_text: String,
    pub name_text: String,
    pub pinned: String,
    pub priority: String,
    pub budget_text: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_peering_agreement_view(item: &PeeringAgreement) -> PeeringAgreementView {
    PeeringAgreementView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        pinned: format!("{}", item.pinned),
        priority: item.priority.to_string(),
        budget_text: format!("{:.2}", item.budget),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_agreement_views(items: &[PeeringAgreement]) -> Vec<PeeringAgreementView> {
    let mut views: Vec<PeeringAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_agreement_view(item));
    }
    views
}

pub fn sum_peering_agreement_view(view: &PeeringAgreementView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_priority as f64 + parsed_budget + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_agreement("code-793", "name-803");
        let view = to_peering_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
