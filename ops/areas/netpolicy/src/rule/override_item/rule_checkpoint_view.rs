use super::*;

use acme_kit::text;

/// How screens and exports show a rule checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpointView {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub namespace: String,
    pub retries_text: String,
    pub threshold_text: String,
    pub source: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_rule_checkpoint_view(item: &RuleCheckpoint) -> RuleCheckpointView {
    RuleCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        locale: item.locale.clone(),
        namespace: text::truncate(&item.namespace, 80),
        retries_text: item.retries.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        source: item.source.clone(),
        scope: text::truncate(&item.scope, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_checkpoint_views(items: &[RuleCheckpoint]) -> Vec<RuleCheckpointView> {
    let mut views: Vec<RuleCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_checkpoint_view(item));
    }
    views
}

pub fn sum_rule_checkpoint_view(view: &RuleCheckpointView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_retries as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_checkpoint("code-487", "title-91");
        let view = to_rule_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
