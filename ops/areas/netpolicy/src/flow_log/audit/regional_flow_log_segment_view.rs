use super::*;

use acme_kit::text;

/// How screens and exports show a regional flow log segment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFlowLogSegmentView {
    pub code: String,
    pub name: String,
    pub budget_text: String,
    pub timezone_text: String,
    pub window_days: String,
    pub key_id: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_regional_flow_log_segment_view(item: &RegionalFlowLogSegment) -> RegionalFlowLogSegmentView {
    RegionalFlowLogSegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        timezone_text: text::truncate(&item.timezone, 80),
        window_days: item.window_days.to_string(),
        key_id: text::truncate(&item.key_id, 120),
        audience_text: text::truncate(&item.audience, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_flow_log_segment_views(items: &[RegionalFlowLogSegment]) -> Vec<RegionalFlowLogSegmentView> {
    let mut views: Vec<RegionalFlowLogSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_flow_log_segment_view(item));
    }
    views
}

pub fn sum_regional_flow_log_segment_view(view: &RegionalFlowLogSegmentView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_budget + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_flow_log_segment("code-131", "name-566");
        let view = to_regional_flow_log_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_flow_log_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
