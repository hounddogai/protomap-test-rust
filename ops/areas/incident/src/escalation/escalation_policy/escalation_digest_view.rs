use super::*;

use acme_kit::text;

/// How screens and exports show a escalation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationDigestView {
    pub code_text: String,
    pub title: String,
    pub sequence: String,
    pub revision: String,
    pub label_text: String,
    pub ttl_seconds_text: String,
    pub score_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_escalation_digest_view(item: &EscalationDigest) -> EscalationDigestView {
    EscalationDigestView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        revision: item.revision.to_string(),
        label_text: item.label.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        score_text: format!("{:.2}", item.score),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_digest_views(items: &[EscalationDigest]) -> Vec<EscalationDigestView> {
    let mut views: Vec<EscalationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_digest_view(item));
    }
    views
}

pub fn sum_escalation_digest_view(view: &EscalationDigestView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_revision as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_digest("code-17", "title-922");
        let view = to_escalation_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
