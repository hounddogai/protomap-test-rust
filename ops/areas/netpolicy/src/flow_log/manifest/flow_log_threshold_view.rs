use super::*;

use acme_kit::text;

/// How screens and exports show a flow log threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdView {
    pub code: String,
    pub title_text: String,
    pub approved: String,
    pub reviewer: String,
    pub key_id_text: String,
    pub ttl_seconds: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_flow_log_threshold_view(item: &FlowLogThreshold) -> FlowLogThresholdView {
    FlowLogThresholdView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        approved: format!("{}", item.approved),
        reviewer: text::truncate(&item.reviewer, 40),
        key_id_text: item.key_id.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        locale: text::truncate(&item.locale, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_threshold_views(items: &[FlowLogThreshold]) -> Vec<FlowLogThresholdView> {
    let mut views: Vec<FlowLogThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_threshold_view(item));
    }
    views
}

pub fn sum_flow_log_threshold_view(view: &FlowLogThresholdView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_threshold("code-963", "title-3");
        let view = to_flow_log_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
