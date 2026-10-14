use super::*;

use acme_kit::text;

/// How screens and exports show a rollout bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBundleView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub budget: String,
    pub channel: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_rollout_bundle_view(item: &RolloutBundle) -> RolloutBundleView {
    RolloutBundleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        port: item.port.to_string(),
        budget: format!("{:.2}", item.budget),
        channel: item.channel.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_bundle_views(items: &[RolloutBundle]) -> Vec<RolloutBundleView> {
    let mut views: Vec<RolloutBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_bundle_view(item));
    }
    views
}

pub fn sum_rollout_bundle_view(view: &RolloutBundleView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_bundle("code-597", "label-655");
        let view = to_rollout_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
