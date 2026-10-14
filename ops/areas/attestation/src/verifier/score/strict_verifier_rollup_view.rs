use super::*;

/// How screens and exports show a strict verifier rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictVerifierRollupView {
    pub code: String,
    pub name_text: String,
    pub country_text: String,
    pub expires_at_text: String,
    pub budget_text: String,
    pub created_at: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_strict_verifier_rollup_view(item: &StrictVerifierRollup) -> StrictVerifierRollupView {
    StrictVerifierRollupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        country_text: item.country.clone(),
        expires_at_text: item.expires_at.to_string(),
        budget_text: format!("{:.2}", item.budget),
        created_at: item.created_at.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_verifier_rollup_views(items: &[StrictVerifierRollup]) -> Vec<StrictVerifierRollupView> {
    let mut views: Vec<StrictVerifierRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_verifier_rollup_view(item));
    }
    views
}

pub fn sum_strict_verifier_rollup_view(view: &StrictVerifierRollupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_budget + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_verifier_rollup("code-967", "name-356");
        let view = to_strict_verifier_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_verifier_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
