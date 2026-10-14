use super::*;

use acme_kit::text;

/// How screens and exports show a global plan draft.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanDraftView {
    pub code: String,
    pub title: String,
    pub active: String,
    pub floor_value: String,
    pub cipher_text: String,
    pub reviewer: String,
    pub burst_text: String,
    pub tenant_id_text: String,
    pub phone_text: String,
    pub expires_at: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_global_plan_draft_view(item: &GlobalPlanDraft) -> GlobalPlanDraftView {
    GlobalPlanDraftView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        active: format!("{}", item.active),
        floor_value: format!("{:.2}", item.floor_value),
        cipher_text: item.cipher.clone(),
        reviewer: item.reviewer.clone(),
        burst_text: item.burst.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        phone_text: item.phone.clone(),
        expires_at: item.expires_at.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_plan_draft_views(items: &[GlobalPlanDraft]) -> Vec<GlobalPlanDraftView> {
    let mut views: Vec<GlobalPlanDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_plan_draft_view(item));
    }
    views
}

pub fn sum_global_plan_draft_view(view: &GlobalPlanDraftView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_floor_value + (parsed_burst as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_plan_draft("code-586", "title-399");
        let view = to_global_plan_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_plan_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
