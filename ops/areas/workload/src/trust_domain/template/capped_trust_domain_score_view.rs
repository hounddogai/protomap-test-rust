use super::*;

/// How screens and exports show a capped trust domain score.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainScoreView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub score: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_capped_trust_domain_score_view(item: &CappedTrustDomainScore) -> CappedTrustDomainScoreView {
    CappedTrustDomainScoreView {
        code: item.code.clone(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        score: format!("{:.2}", item.score),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_trust_domain_score_views(items: &[CappedTrustDomainScore]) -> Vec<CappedTrustDomainScoreView> {
    let mut views: Vec<CappedTrustDomainScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_trust_domain_score_view(item));
    }
    views
}

pub fn sum_capped_trust_domain_score_view(view: &CappedTrustDomainScoreView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_amount + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_trust_domain_score("code-520", "label-89");
        let view = to_capped_trust_domain_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_trust_domain_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
