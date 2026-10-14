use super::*;

use acme_kit::text;

/// How screens and exports show a manual isolation target.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTargetView {
    pub code: String,
    pub label: String,
    pub budget: String,
    pub channel: String,
    pub starts_at: String,
    pub weight: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_manual_isolation_target_view(item: &ManualIsolationTarget) -> ManualIsolationTargetView {
    ManualIsolationTargetView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        budget: format!("{:.2}", item.budget),
        channel: text::truncate(&item.channel, 64),
        starts_at: item.starts_at.to_string(),
        weight: format!("{:.2}", item.weight),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_isolation_target_views(items: &[ManualIsolationTarget]) -> Vec<ManualIsolationTargetView> {
    let mut views: Vec<ManualIsolationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_isolation_target_view(item));
    }
    views
}

pub fn sum_manual_isolation_target_view(view: &ManualIsolationTargetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_budget + (parsed_starts_at as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_isolation_target("code-734", "label-933");
        let view = to_manual_isolation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_isolation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
