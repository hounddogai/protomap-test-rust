use super::*;

use acme_kit::text;

/// How screens and exports show a fallback failover draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraftView {
    pub code: String,
    pub label_text: String,
    pub pinned: String,
    pub visible: String,
    pub country_text: String,
    pub key_id: String,
    pub active_text: String,
    pub phone_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_fallback_failover_draft_view(item: &FallbackFailoverDraft) -> FallbackFailoverDraftView {
    FallbackFailoverDraftView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        pinned: format!("{}", item.pinned),
        visible: format!("{}", item.visible),
        country_text: item.country.clone(),
        key_id: item.key_id.clone(),
        active_text: format!("{}", item.active),
        phone_text: text::truncate(&item.phone, 80),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_failover_draft_views(items: &[FallbackFailoverDraft]) -> Vec<FallbackFailoverDraftView> {
    let mut views: Vec<FallbackFailoverDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_failover_draft_view(item));
    }
    views
}

pub fn sum_fallback_failover_draft_view(view: &FallbackFailoverDraftView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_failover_draft("code-750", "label-895");
        let view = to_fallback_failover_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_failover_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
