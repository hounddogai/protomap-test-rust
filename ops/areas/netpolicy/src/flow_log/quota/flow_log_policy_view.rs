use super::*;

use acme_kit::text;

/// How screens and exports show a flow log policy.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyView {
    pub code_text: String,
    pub title: String,
    pub cluster: String,
    pub ratio: String,
    pub reviewer_text: String,
    pub checksum: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_flow_log_policy_view(item: &FlowLogPolicy) -> FlowLogPolicyView {
    FlowLogPolicyView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cluster: item.cluster.clone(),
        ratio: format!("{:.2}", item.ratio),
        reviewer_text: item.reviewer.clone(),
        checksum: item.checksum.clone(),
        locale: text::truncate(&item.locale, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_policy_views(items: &[FlowLogPolicy]) -> Vec<FlowLogPolicyView> {
    let mut views: Vec<FlowLogPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_policy_view(item));
    }
    views
}

pub fn sum_flow_log_policy_view(view: &FlowLogPolicyView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_policy("code-832", "title-967");
        let view = to_flow_log_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
