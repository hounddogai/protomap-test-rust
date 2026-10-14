use super::*;

use acme_kit::text;

/// How screens and exports show a limit policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicyView {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub scope: String,
    pub phone: String,
    pub description: String,
    pub floor_value: String,
    pub port: String,
    pub burst_text: String,
    pub priority_text: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_limit_policy_view(item: &LimitPolicy) -> LimitPolicyView {
    LimitPolicyView {
        code: item.code.clone(),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        scope: item.scope.clone(),
        phone: item.phone.clone(),
        description: item.description.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        port: item.port.to_string(),
        burst_text: item.burst.to_string(),
        priority_text: item.priority.to_string(),
        external_ref_text: text::truncate(&item.external_ref, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_policy_views(items: &[LimitPolicy]) -> Vec<LimitPolicyView> {
    let mut views: Vec<LimitPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_policy_view(item));
    }
    views
}

pub fn sum_limit_policy_view(view: &LimitPolicyView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_floor_value + (parsed_port as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_policy("code-411", "label-282");
        let view = to_limit_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
