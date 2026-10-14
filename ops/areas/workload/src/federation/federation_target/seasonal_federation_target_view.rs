use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal federation target.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationTargetView {
    pub code: String,
    pub name_text: String,
    pub visible_text: String,
    pub description: String,
    pub title_text: String,
    pub retries_text: String,
    pub cluster: String,
    pub minimum: String,
    pub label: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_seasonal_federation_target_view(item: &SeasonalFederationTarget) -> SeasonalFederationTargetView {
    SeasonalFederationTargetView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        visible_text: format!("{}", item.visible),
        description: text::truncate(&item.description, 80),
        title_text: item.title.clone(),
        retries_text: item.retries.to_string(),
        cluster: text::truncate(&item.cluster, 80),
        minimum: format!("{:.2}", item.minimum),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_federation_target_views(items: &[SeasonalFederationTarget]) -> Vec<SeasonalFederationTargetView> {
    let mut views: Vec<SeasonalFederationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_federation_target_view(item));
    }
    views
}

pub fn sum_seasonal_federation_target_view(view: &SeasonalFederationTargetView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_retries as f64 + parsed_minimum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_federation_target("code-38", "name-915");
        let view = to_seasonal_federation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_federation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
