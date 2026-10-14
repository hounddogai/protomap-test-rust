use super::*;

use acme_kit::text;

/// How screens and exports show a escalation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimView {
    pub code_text: String,
    pub title_text: String,
    pub duration_minutes_text: String,
    pub scope: String,
    pub attempts: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_escalation_claim_view(item: &EscalationClaim) -> EscalationClaimView {
    EscalationClaimView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        duration_minutes_text: item.duration_minutes.to_string(),
        scope: text::truncate(&item.scope, 64),
        attempts: item.attempts.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_claim_views(items: &[EscalationClaim]) -> Vec<EscalationClaimView> {
    let mut views: Vec<EscalationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_claim_view(item));
    }
    views
}

pub fn sum_escalation_claim_view(view: &EscalationClaimView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_attempts as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_claim("code-867", "title-165");
        let view = to_escalation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
