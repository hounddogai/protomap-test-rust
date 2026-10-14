use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive tenant draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftView {
    pub code: String,
    pub name_text: String,
    pub source: String,
    pub reviewer_text: String,
    pub expires_at: String,
    pub capacity: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_adaptive_tenant_draft_view(item: &AdaptiveTenantDraft) -> AdaptiveTenantDraftView {
    AdaptiveTenantDraftView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        source: item.source.clone(),
        reviewer_text: item.reviewer.clone(),
        expires_at: item.expires_at.to_string(),
        capacity: item.capacity.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_tenant_draft_views(items: &[AdaptiveTenantDraft]) -> Vec<AdaptiveTenantDraftView> {
    let mut views: Vec<AdaptiveTenantDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_tenant_draft_view(item));
    }
    views
}

pub fn sum_adaptive_tenant_draft_view(view: &AdaptiveTenantDraftView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_capacity as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_tenant_draft("code-320", "name-395");
        let view = to_adaptive_tenant_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_tenant_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
