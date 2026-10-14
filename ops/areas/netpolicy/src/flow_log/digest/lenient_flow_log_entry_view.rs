use super::*;

/// How screens and exports show a lenient flow log entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryView {
    pub code_text: String,
    pub label: String,
    pub discount: String,
    pub window_days: String,
    pub limit: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_lenient_flow_log_entry_view(item: &LenientFlowLogEntry) -> LenientFlowLogEntryView {
    LenientFlowLogEntryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        window_days: item.window_days.to_string(),
        limit: item.limit.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_flow_log_entry_views(items: &[LenientFlowLogEntry]) -> Vec<LenientFlowLogEntryView> {
    let mut views: Vec<LenientFlowLogEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_flow_log_entry_view(item));
    }
    views
}

pub fn sum_lenient_flow_log_entry_view(view: &LenientFlowLogEntryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_discount + (parsed_window_days as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_flow_log_entry("code-380", "label-231");
        let view = to_lenient_flow_log_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_flow_log_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
