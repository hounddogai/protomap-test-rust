use super::*;

use acme_kit::text;

/// How screens and exports show a rollout policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPolicyView {
    pub code_text: String,
    pub label_text: String,
    pub floor_value_text: String,
    pub currency_text: String,
    pub margin: String,
    pub duration_minutes: String,
    pub retries_text: String,
    pub cost: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_rollout_policy_view(item: &RolloutPolicy) -> RolloutPolicyView {
    RolloutPolicyView {
        code_text: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        currency_text: item.currency.clone(),
        margin: format!("{:.2}", item.margin),
        duration_minutes: item.duration_minutes.to_string(),
        retries_text: item.retries.to_string(),
        cost: format!("{:.2}", item.cost),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_policy_views(items: &[RolloutPolicy]) -> Vec<RolloutPolicyView> {
    let mut views: Vec<RolloutPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_policy_view(item));
    }
    views
}

pub fn sum_rollout_policy_view(view: &RolloutPolicyView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_floor_value + parsed_margin + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_policy("code-134", "label-936");
        let view = to_rollout_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
