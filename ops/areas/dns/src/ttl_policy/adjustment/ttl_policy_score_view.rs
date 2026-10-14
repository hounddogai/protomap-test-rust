use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy score.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScoreView {
    pub code_text: String,
    pub title: String,
    pub attempts_text: String,
    pub ttl_seconds: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_ttl_policy_score_view(item: &TtlPolicyScore) -> TtlPolicyScoreView {
    TtlPolicyScoreView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        attempts_text: item.attempts.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_score_views(items: &[TtlPolicyScore]) -> Vec<TtlPolicyScoreView> {
    let mut views: Vec<TtlPolicyScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_score_view(item));
    }
    views
}

pub fn sum_ttl_policy_score_view(view: &TtlPolicyScoreView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_ttl_seconds as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_score("code-354", "title-498");
        let view = to_ttl_policy_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
