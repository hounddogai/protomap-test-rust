use super::*;

/// How screens and exports show a flow log claim.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaimView {
    pub code: String,
    pub label_text: String,
    pub created_at: String,
    pub burst: String,
    pub contact_email: String,
    pub country: String,
    pub namespace_text: String,
    pub approved: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_flow_log_claim_view(item: &FlowLogClaim) -> FlowLogClaimView {
    FlowLogClaimView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        created_at: item.created_at.to_string(),
        burst: item.burst.to_string(),
        contact_email: item.contact_email.clone(),
        country: item.country.clone(),
        namespace_text: item.namespace.clone(),
        approved: format!("{}", item.approved),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_claim_views(items: &[FlowLogClaim]) -> Vec<FlowLogClaimView> {
    let mut views: Vec<FlowLogClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_claim_view(item));
    }
    views
}

pub fn sum_flow_log_claim_view(view: &FlowLogClaimView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_burst as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_claim("code-26", "label-582");
        let view = to_flow_log_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
