use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy template.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateView {
    pub code_text: String,
    pub label: String,
    pub window_days_text: String,
    pub capacity: String,
    pub key_id_text: String,
    pub port: String,
    pub duration_minutes_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_batched_policy_template_view(item: &BatchedPolicyTemplate) -> BatchedPolicyTemplateView {
    BatchedPolicyTemplateView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        window_days_text: item.window_days.to_string(),
        capacity: item.capacity.to_string(),
        key_id_text: item.key_id.clone(),
        port: item.port.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_template_views(items: &[BatchedPolicyTemplate]) -> Vec<BatchedPolicyTemplateView> {
    let mut views: Vec<BatchedPolicyTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_template_view(item));
    }
    views
}

pub fn sum_batched_policy_template_view(view: &BatchedPolicyTemplateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_capacity as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_policy_template("code-80", "label-452");
        let view = to_batched_policy_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_policy_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
