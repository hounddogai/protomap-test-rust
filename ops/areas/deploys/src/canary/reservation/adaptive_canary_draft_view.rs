use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive canary draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryDraftView {
    pub code: String,
    pub title: String,
    pub namespace_text: String,
    pub threshold: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_adaptive_canary_draft_view(item: &AdaptiveCanaryDraft) -> AdaptiveCanaryDraftView {
    AdaptiveCanaryDraftView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 80),
        namespace_text: item.namespace.clone(),
        threshold: format!("{:.2}", item.threshold),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_canary_draft_views(items: &[AdaptiveCanaryDraft]) -> Vec<AdaptiveCanaryDraftView> {
    let mut views: Vec<AdaptiveCanaryDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_canary_draft_view(item));
    }
    views
}

pub fn sum_adaptive_canary_draft_view(view: &AdaptiveCanaryDraftView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_threshold + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_canary_draft("code-320", "title-161");
        let view = to_adaptive_canary_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_canary_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
