use super::*;

use acme_kit::text;

/// How screens and exports show a flow log cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleView {
    pub code: String,
    pub label: String,
    pub attempts: String,
    pub minimum: String,
    pub external_ref: String,
    pub cipher_text: String,
    pub rate: String,
    pub retries: String,
    pub owner_id_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_flow_log_cycle_view(item: &FlowLogCycle) -> FlowLogCycleView {
    FlowLogCycleView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 40),
        attempts: item.attempts.to_string(),
        minimum: format!("{:.2}", item.minimum),
        external_ref: item.external_ref.clone(),
        cipher_text: item.cipher.clone(),
        rate: format!("{:.2}", item.rate),
        retries: item.retries.to_string(),
        owner_id_text: item.owner_id.clone(),
        summary: text::truncate(&item.summary, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_cycle_views(items: &[FlowLogCycle]) -> Vec<FlowLogCycleView> {
    let mut views: Vec<FlowLogCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_cycle_view(item));
    }
    views
}

pub fn sum_flow_log_cycle_view(view: &FlowLogCycleView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_minimum + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_cycle("code-607", "label-695");
        let view = to_flow_log_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
