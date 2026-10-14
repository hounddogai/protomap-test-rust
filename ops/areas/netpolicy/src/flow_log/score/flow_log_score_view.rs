use super::*;

use acme_kit::text;

/// How screens and exports show a flow log score.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreView {
    pub code: String,
    pub title_text: String,
    pub starts_at: String,
    pub segment: String,
    pub tenant_id: String,
    pub locale: String,
    pub retries_text: String,
    pub quantity: String,
    pub sequence_text: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_flow_log_score_view(item: &FlowLogScore) -> FlowLogScoreView {
    FlowLogScoreView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        starts_at: item.starts_at.to_string(),
        segment: item.segment.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        locale: item.locale.clone(),
        retries_text: item.retries.to_string(),
        quantity: item.quantity.to_string(),
        sequence_text: item.sequence.to_string(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_score_views(items: &[FlowLogScore]) -> Vec<FlowLogScoreView> {
    let mut views: Vec<FlowLogScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_score_view(item));
    }
    views
}

pub fn sum_flow_log_score_view(view: &FlowLogScoreView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_retries as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_score("code-521", "title-282");
        let view = to_flow_log_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
