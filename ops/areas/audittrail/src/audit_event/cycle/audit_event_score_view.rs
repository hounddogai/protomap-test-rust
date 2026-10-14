use super::*;

use acme_kit::text;

/// How screens and exports show a audit event score.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScoreView {
    pub code_text: String,
    pub title: String,
    pub category: String,
    pub limit: String,
    pub source: String,
    pub amount: String,
    pub capacity_text: String,
    pub currency: String,
    pub reviewer_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_audit_event_score_view(item: &AuditEventScore) -> AuditEventScoreView {
    AuditEventScoreView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        category: item.category.clone(),
        limit: item.limit.to_string(),
        source: item.source.clone(),
        amount: format!("{:.2}", item.amount),
        capacity_text: item.capacity.to_string(),
        currency: item.currency.clone(),
        reviewer_text: item.reviewer.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_score_views(items: &[AuditEventScore]) -> Vec<AuditEventScoreView> {
    let mut views: Vec<AuditEventScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_score_view(item));
    }
    views
}

pub fn sum_audit_event_score_view(view: &AuditEventScoreView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_limit as f64 + parsed_amount + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_score("code-946", "title-76");
        let view = to_audit_event_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
