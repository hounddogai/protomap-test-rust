use super::*;

/// How screens and exports show a weighted flow log record.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogRecordView {
    pub code_text: String,
    pub title: String,
    pub timezone: String,
    pub retries: String,
    pub sequence_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_weighted_flow_log_record_view(item: &WeightedFlowLogRecord) -> WeightedFlowLogRecordView {
    WeightedFlowLogRecordView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        timezone: item.timezone.clone(),
        retries: item.retries.to_string(),
        sequence_text: item.sequence.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_flow_log_record_views(items: &[WeightedFlowLogRecord]) -> Vec<WeightedFlowLogRecordView> {
    let mut views: Vec<WeightedFlowLogRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_flow_log_record_view(item));
    }
    views
}

pub fn sum_weighted_flow_log_record_view(view: &WeightedFlowLogRecordView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_sequence as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_flow_log_record("code-993", "title-101");
        let view = to_weighted_flow_log_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_flow_log_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
