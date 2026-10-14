use super::*;

use acme_kit::text;

/// How screens and exports show a lenient flow log draft.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogDraftView {
    pub code: String,
    pub name: String,
    pub margin: String,
    pub retries: String,
    pub revision: String,
    pub archived_text: String,
    pub grace_days_text: String,
    pub port: String,
    pub notes_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_lenient_flow_log_draft_view(item: &LenientFlowLogDraft) -> LenientFlowLogDraftView {
    LenientFlowLogDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        retries: item.retries.to_string(),
        revision: item.revision.to_string(),
        archived_text: format!("{}", item.archived),
        grace_days_text: item.grace_days.to_string(),
        port: item.port.to_string(),
        notes_text: item.notes.clone(),
        phone: text::truncate(&item.phone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_flow_log_draft_views(items: &[LenientFlowLogDraft]) -> Vec<LenientFlowLogDraftView> {
    let mut views: Vec<LenientFlowLogDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_flow_log_draft_view(item));
    }
    views
}

pub fn sum_lenient_flow_log_draft_view(view: &LenientFlowLogDraftView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_margin + (parsed_retries as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_flow_log_draft("code-463", "name-587");
        let view = to_lenient_flow_log_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_flow_log_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
