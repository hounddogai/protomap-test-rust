use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupView {
    pub code_text: String,
    pub name: String,
    pub duration_minutes: String,
    pub floor_value_text: String,
    pub version: String,
    pub window_days: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_allowlist_rollup_view(item: &AllowlistRollup) -> AllowlistRollupView {
    AllowlistRollupView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        version: item.version.to_string(),
        window_days: item.window_days.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_rollup_views(items: &[AllowlistRollup]) -> Vec<AllowlistRollupView> {
    let mut views: Vec<AllowlistRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_rollup_view(item));
    }
    views
}

pub fn sum_allowlist_rollup_view(view: &AllowlistRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_floor_value + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_rollup("code-164", "name-864");
        let view = to_allowlist_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
