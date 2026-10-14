use super::*;

use acme_kit::text;

/// How screens and exports show a flow log digest.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDigestView {
    pub code: String,
    pub label_text: String,
    pub host: String,
    pub phone_text: String,
    pub segment: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_flow_log_digest_view(item: &FlowLogDigest) -> FlowLogDigestView {
    FlowLogDigestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        host: text::truncate(&item.host, 64),
        phone_text: item.phone.clone(),
        segment: item.segment.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_digest_views(items: &[FlowLogDigest]) -> Vec<FlowLogDigestView> {
    let mut views: Vec<FlowLogDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_digest_view(item));
    }
    views
}

pub fn sum_flow_log_digest_view(view: &FlowLogDigestView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_digest("code-890", "label-344");
        let view = to_flow_log_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
