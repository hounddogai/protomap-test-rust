use super::*;

use acme_kit::text;

/// How screens and exports show a pending redaction slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionSlotView {
    pub code_text: String,
    pub label_text: String,
    pub burst: String,
    pub timezone: String,
    pub revision_text: String,
    pub scope: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_pending_redaction_slot_view(item: &PendingRedactionSlot) -> PendingRedactionSlotView {
    PendingRedactionSlotView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        timezone: item.timezone.clone(),
        revision_text: item.revision.to_string(),
        scope: item.scope.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_redaction_slot_views(items: &[PendingRedactionSlot]) -> Vec<PendingRedactionSlotView> {
    let mut views: Vec<PendingRedactionSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_redaction_slot_view(item));
    }
    views
}

pub fn sum_pending_redaction_slot_view(view: &PendingRedactionSlotView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_redaction_slot("code-95", "label-706");
        let view = to_pending_redaction_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_redaction_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
