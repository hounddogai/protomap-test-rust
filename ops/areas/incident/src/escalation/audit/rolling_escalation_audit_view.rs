use super::*;

use acme_kit::text;

/// How screens and exports show a rolling escalation audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationAuditView {
    pub code_text: String,
    pub title: String,
    pub ratio: String,
    pub cipher_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_rolling_escalation_audit_view(item: &RollingEscalationAudit) -> RollingEscalationAuditView {
    RollingEscalationAuditView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        ratio: format!("{:.2}", item.ratio),
        cipher_text: item.cipher.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_escalation_audit_views(items: &[RollingEscalationAudit]) -> Vec<RollingEscalationAuditView> {
    let mut views: Vec<RollingEscalationAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_escalation_audit_view(item));
    }
    views
}

pub fn sum_rolling_escalation_audit_view(view: &RollingEscalationAuditView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ratio + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_escalation_audit("code-459", "title-986");
        let view = to_rolling_escalation_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_escalation_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
