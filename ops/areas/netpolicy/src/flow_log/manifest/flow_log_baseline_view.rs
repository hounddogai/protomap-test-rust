use super::*;

use acme_kit::text;

/// How screens and exports show a flow log baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBaselineView {
    pub code: String,
    pub name_text: String,
    pub batch_size: String,
    pub margin: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_flow_log_baseline_view(item: &FlowLogBaseline) -> FlowLogBaselineView {
    FlowLogBaselineView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        margin: format!("{:.2}", item.margin),
        host: text::truncate(&item.host, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_baseline_views(items: &[FlowLogBaseline]) -> Vec<FlowLogBaselineView> {
    let mut views: Vec<FlowLogBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_baseline_view(item));
    }
    views
}

pub fn sum_flow_log_baseline_view(view: &FlowLogBaselineView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_baseline("code-458", "name-927");
        let view = to_flow_log_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
