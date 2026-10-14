use super::*;

use acme_kit::text;

/// How screens and exports show a manual credential rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialRollupView {
    pub code_text: String,
    pub title: String,
    pub name_text: String,
    pub description: String,
    pub segment: String,
    pub cost: String,
    pub window_days_text: String,
    pub pinned: String,
    pub channel: String,
    pub capacity: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_manual_credential_rollup_view(item: &ManualCredentialRollup) -> ManualCredentialRollupView {
    ManualCredentialRollupView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 64),
        name_text: text::truncate(&item.name, 120),
        description: item.description.clone(),
        segment: item.segment.clone(),
        cost: format!("{:.2}", item.cost),
        window_days_text: item.window_days.to_string(),
        pinned: format!("{}", item.pinned),
        channel: item.channel.clone(),
        capacity: item.capacity.to_string(),
        unit: text::truncate(&item.unit, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_credential_rollup_views(items: &[ManualCredentialRollup]) -> Vec<ManualCredentialRollupView> {
    let mut views: Vec<ManualCredentialRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_credential_rollup_view(item));
    }
    views
}

pub fn sum_manual_credential_rollup_view(view: &ManualCredentialRollupView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
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
    Ok(parsed_cost + (parsed_window_days as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_credential_rollup("code-995", "title-932");
        let view = to_manual_credential_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_credential_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
