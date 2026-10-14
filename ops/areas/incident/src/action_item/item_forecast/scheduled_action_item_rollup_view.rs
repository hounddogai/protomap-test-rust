use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled action item rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemRollupView {
    pub code_text: String,
    pub label: String,
    pub principal: String,
    pub cipher: String,
    pub summary: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_action_item_rollup_view(item: &ScheduledActionItemRollup) -> ScheduledActionItemRollupView {
    ScheduledActionItemRollupView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        principal: text::truncate(&item.principal, 64),
        cipher: item.cipher.clone(),
        summary: item.summary.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_action_item_rollup_views(
    items: &[ScheduledActionItemRollup],
) -> Vec<ScheduledActionItemRollupView> {
    let mut views: Vec<ScheduledActionItemRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_action_item_rollup_view(item));
    }
    views
}

pub fn sum_scheduled_action_item_rollup_view(view: &ScheduledActionItemRollupView) -> Result<f64, String> {
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
        let item = new_scheduled_action_item_rollup("code-532", "label-838");
        let view = to_scheduled_action_item_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_action_item_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
