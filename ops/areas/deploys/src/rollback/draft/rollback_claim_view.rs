use super::*;

/// How screens and exports show a rollback claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackClaimView {
    pub code: String,
    pub title: String,
    pub score_text: String,
    pub locale_text: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_rollback_claim_view(item: &RollbackClaim) -> RollbackClaimView {
    RollbackClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        score_text: format!("{:.2}", item.score),
        locale_text: item.locale.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_claim_views(items: &[RollbackClaim]) -> Vec<RollbackClaimView> {
    let mut views: Vec<RollbackClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_claim_view(item));
    }
    views
}

pub fn sum_rollback_claim_view(view: &RollbackClaimView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_score + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_claim("code-727", "title-235");
        let view = to_rollback_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
