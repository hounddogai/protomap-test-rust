use super::*;

use acme_kit::text;

/// How screens and exports show a flow log allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAllocationView {
    pub code: String,
    pub title_text: String,
    pub channel: String,
    pub priority_text: String,
    pub country_text: String,
    pub reviewer: String,
    pub cipher_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_flow_log_allocation_view(item: &FlowLogAllocation) -> FlowLogAllocationView {
    FlowLogAllocationView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        channel: text::truncate(&item.channel, 120),
        priority_text: item.priority.to_string(),
        country_text: item.country.clone(),
        reviewer: text::truncate(&item.reviewer, 80),
        cipher_text: text::truncate(&item.cipher, 120),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_allocation_views(items: &[FlowLogAllocation]) -> Vec<FlowLogAllocationView> {
    let mut views: Vec<FlowLogAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_allocation_view(item));
    }
    views
}

pub fn sum_flow_log_allocation_view(view: &FlowLogAllocationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_allocation("code-320", "title-430");
        let view = to_flow_log_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
