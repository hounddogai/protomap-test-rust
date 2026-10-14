use super::*;

use acme_kit::text;

/// How screens and exports show a action item score.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScoreView {
    pub code: String,
    pub title: String,
    pub threshold_text: String,
    pub cluster_text: String,
    pub tenant_id_text: String,
    pub fingerprint: String,
    pub updated_at_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_action_item_score_view(item: &ActionItemScore) -> ActionItemScoreView {
    ActionItemScoreView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        threshold_text: format!("{:.2}", item.threshold),
        cluster_text: text::truncate(&item.cluster, 40),
        tenant_id_text: item.tenant_id.clone(),
        fingerprint: item.fingerprint.clone(),
        updated_at_text: item.updated_at.to_string(),
        audience: text::truncate(&item.audience, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_score_views(items: &[ActionItemScore]) -> Vec<ActionItemScoreView> {
    let mut views: Vec<ActionItemScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_score_view(item));
    }
    views
}

pub fn sum_action_item_score_view(view: &ActionItemScoreView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_threshold + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_score("code-730", "title-793");
        let view = to_action_item_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
