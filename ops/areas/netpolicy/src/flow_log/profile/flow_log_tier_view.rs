use super::*;

/// How screens and exports show a flow log tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTierView {
    pub code: String,
    pub title_text: String,
    pub principal: String,
    pub created_at: String,
    pub approved: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_flow_log_tier_view(item: &FlowLogTier) -> FlowLogTierView {
    FlowLogTierView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        principal: item.principal.clone(),
        created_at: item.created_at.to_string(),
        approved: format!("{}", item.approved),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_tier_views(items: &[FlowLogTier]) -> Vec<FlowLogTierView> {
    let mut views: Vec<FlowLogTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_tier_view(item));
    }
    views
}

pub fn sum_flow_log_tier_view(view: &FlowLogTierView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_tier("code-58", "title-197");
        let view = to_flow_log_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
