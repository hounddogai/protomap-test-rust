use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive allowlist draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistDraftView {
    pub code_text: String,
    pub label: String,
    pub timezone: String,
    pub rotation_days: String,
    pub currency: String,
    pub contact_email: String,
    pub cluster: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_adaptive_allowlist_draft_view(item: &AdaptiveAllowlistDraft) -> AdaptiveAllowlistDraftView {
    AdaptiveAllowlistDraftView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        rotation_days: item.rotation_days.to_string(),
        currency: item.currency.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        cluster: item.cluster.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_allowlist_draft_views(items: &[AdaptiveAllowlistDraft]) -> Vec<AdaptiveAllowlistDraftView> {
    let mut views: Vec<AdaptiveAllowlistDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_allowlist_draft_view(item));
    }
    views
}

pub fn sum_adaptive_allowlist_draft_view(view: &AdaptiveAllowlistDraftView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_allowlist_draft("code-138", "label-731");
        let view = to_adaptive_allowlist_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_allowlist_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
