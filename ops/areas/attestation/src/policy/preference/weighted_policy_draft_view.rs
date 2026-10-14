use super::*;

use acme_kit::text;

/// How screens and exports show a weighted policy draft.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftView {
    pub code: String,
    pub label: String,
    pub ratio: String,
    pub scope_text: String,
    pub priority: String,
    pub margin: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_weighted_policy_draft_view(item: &WeightedPolicyDraft) -> WeightedPolicyDraftView {
    WeightedPolicyDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        ratio: format!("{:.2}", item.ratio),
        scope_text: item.scope.clone(),
        priority: item.priority.to_string(),
        margin: format!("{:.2}", item.margin),
        contact_email: text::truncate(&item.contact_email, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_policy_draft_views(items: &[WeightedPolicyDraft]) -> Vec<WeightedPolicyDraftView> {
    let mut views: Vec<WeightedPolicyDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_policy_draft_view(item));
    }
    views
}

pub fn sum_weighted_policy_draft_view(view: &WeightedPolicyDraftView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ratio + (parsed_priority as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_policy_draft("code-223", "label-686");
        let view = to_weighted_policy_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_policy_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
