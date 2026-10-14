use super::*;

use acme_kit::text;

/// How screens and exports show a flow log batch.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBatchView {
    pub code: String,
    pub label: String,
    pub unit_text: String,
    pub country: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_flow_log_batch_view(item: &FlowLogBatch) -> FlowLogBatchView {
    FlowLogBatchView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        unit_text: item.unit.clone(),
        country: item.country.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_batch_views(items: &[FlowLogBatch]) -> Vec<FlowLogBatchView> {
    let mut views: Vec<FlowLogBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_batch_view(item));
    }
    views
}

pub fn sum_flow_log_batch_view(view: &FlowLogBatchView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_batch("code-557", "label-401");
        let view = to_flow_log_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
