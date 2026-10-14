use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal rollout preference.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreferenceView {
    pub code: String,
    pub title: String,
    pub port: String,
    pub label: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_seasonal_rollout_preference_view(item: &SeasonalRolloutPreference) -> SeasonalRolloutPreferenceView {
    SeasonalRolloutPreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        port: item.port.to_string(),
        label: text::truncate(&item.label, 120),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_rollout_preference_views(items: &[SeasonalRolloutPreference]) -> Vec<SeasonalRolloutPreferenceView> {
    let mut views: Vec<SeasonalRolloutPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_rollout_preference_view(item));
    }
    views
}

pub fn sum_seasonal_rollout_preference_view(view: &SeasonalRolloutPreferenceView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_port as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_rollout_preference("code-133", "title-516");
        let view = to_seasonal_rollout_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_rollout_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
