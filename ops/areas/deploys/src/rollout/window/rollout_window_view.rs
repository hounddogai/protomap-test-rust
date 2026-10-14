use super::*;

use acme_kit::text;

/// How screens and exports show a rollout window.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutWindowView {
    pub code: String,
    pub name_text: String,
    pub minimum: String,
    pub principal_text: String,
    pub country_text: String,
    pub capacity: String,
    pub description_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_rollout_window_view(item: &RolloutWindow) -> RolloutWindowView {
    RolloutWindowView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        principal_text: item.principal.clone(),
        country_text: item.country.clone(),
        capacity: item.capacity.to_string(),
        description_text: text::truncate(&item.description, 80),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_window_views(items: &[RolloutWindow]) -> Vec<RolloutWindowView> {
    let mut views: Vec<RolloutWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_window_view(item));
    }
    views
}

pub fn sum_rollout_window_view(view: &RolloutWindowView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_minimum + (parsed_capacity as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_window("code-340", "name-978");
        let view = to_rollout_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
