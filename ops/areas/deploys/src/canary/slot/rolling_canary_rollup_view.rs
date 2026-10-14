use super::*;

use acme_kit::text;

/// How screens and exports show a rolling canary rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryRollupView {
    pub code: String,
    pub title: String,
    pub audience_text: String,
    pub label: String,
    pub ttl_seconds_text: String,
    pub timezone: String,
    pub window_days: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_rolling_canary_rollup_view(item: &RollingCanaryRollup) -> RollingCanaryRollupView {
    RollingCanaryRollupView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        audience_text: item.audience.clone(),
        label: text::truncate(&item.label, 40),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        timezone: item.timezone.clone(),
        window_days: item.window_days.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_canary_rollup_views(items: &[RollingCanaryRollup]) -> Vec<RollingCanaryRollupView> {
    let mut views: Vec<RollingCanaryRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_canary_rollup_view(item));
    }
    views
}

pub fn sum_rolling_canary_rollup_view(view: &RollingCanaryRollupView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_canary_rollup("code-88", "title-485");
        let view = to_rolling_canary_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_canary_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
