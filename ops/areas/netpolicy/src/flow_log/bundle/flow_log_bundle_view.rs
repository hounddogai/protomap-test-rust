use super::*;

use acme_kit::text;

/// How screens and exports show a flow log bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBundleView {
    pub code_text: String,
    pub label: String,
    pub region: String,
    pub minimum: String,
    pub channel: String,
    pub burst: String,
    pub notes: String,
    pub phone: String,
    pub capacity: String,
    pub key_id: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_flow_log_bundle_view(item: &FlowLogBundle) -> FlowLogBundleView {
    FlowLogBundleView {
        code_text: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        region: item.region.clone(),
        minimum: format!("{:.2}", item.minimum),
        channel: item.channel.clone(),
        burst: item.burst.to_string(),
        notes: item.notes.clone(),
        phone: item.phone.clone(),
        capacity: item.capacity.to_string(),
        key_id: item.key_id.clone(),
        unit: text::truncate(&item.unit, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_bundle_views(items: &[FlowLogBundle]) -> Vec<FlowLogBundleView> {
    let mut views: Vec<FlowLogBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_bundle_view(item));
    }
    views
}

pub fn sum_flow_log_bundle_view(view: &FlowLogBundleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_minimum + (parsed_burst as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_bundle("code-70", "label-77");
        let view = to_flow_log_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
