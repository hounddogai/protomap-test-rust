use super::*;

use acme_kit::text;

/// How screens and exports show a primary rule claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleClaimView {
    pub code_text: String,
    pub title_text: String,
    pub category: String,
    pub ttl_seconds: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_primary_rule_claim_view(item: &PrimaryRuleClaim) -> PrimaryRuleClaimView {
    PrimaryRuleClaimView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        category: text::truncate(&item.category, 64),
        ttl_seconds: item.ttl_seconds.to_string(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rule_claim_views(items: &[PrimaryRuleClaim]) -> Vec<PrimaryRuleClaimView> {
    let mut views: Vec<PrimaryRuleClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rule_claim_view(item));
    }
    views
}

pub fn sum_primary_rule_claim_view(view: &PrimaryRuleClaimView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rule_claim("code-536", "title-966");
        let view = to_primary_rule_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rule_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
