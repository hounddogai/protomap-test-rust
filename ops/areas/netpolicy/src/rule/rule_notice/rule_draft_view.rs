use super::*;

use acme_kit::text;

/// How screens and exports show a rule draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraftView {
    pub code_text: String,
    pub name: String,
    pub margin: String,
    pub notes_text: String,
    pub version: String,
    pub pinned_text: String,
    pub threshold: String,
    pub segment: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_rule_draft_view(item: &RuleDraft) -> RuleDraftView {
    RuleDraftView {
        code_text: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        margin: format!("{:.2}", item.margin),
        notes_text: item.notes.clone(),
        version: item.version.to_string(),
        pinned_text: format!("{}", item.pinned),
        threshold: format!("{:.2}", item.threshold),
        segment: item.segment.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_draft_views(items: &[RuleDraft]) -> Vec<RuleDraftView> {
    let mut views: Vec<RuleDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_draft_view(item));
    }
    views
}

pub fn sum_rule_draft_view(view: &RuleDraftView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_margin + (parsed_version as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_draft("code-871", "name-706");
        let view = to_rule_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
