use super::*;

use acme_kit::text;

/// How screens and exports show a primary overage preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreferenceView {
    pub code: String,
    pub name: String,
    pub cost: String,
    pub starts_at: String,
    pub currency: String,
    pub host_text: String,
    pub margin: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_primary_overage_preference_view(item: &PrimaryOveragePreference) -> PrimaryOveragePreferenceView {
    PrimaryOveragePreferenceView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 64),
        cost: format!("{:.2}", item.cost),
        starts_at: item.starts_at.to_string(),
        currency: item.currency.clone(),
        host_text: item.host.clone(),
        margin: format!("{:.2}", item.margin),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_overage_preference_views(items: &[PrimaryOveragePreference]) -> Vec<PrimaryOveragePreferenceView> {
    let mut views: Vec<PrimaryOveragePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_overage_preference_view(item));
    }
    views
}

pub fn sum_primary_overage_preference_view(view: &PrimaryOveragePreferenceView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_cost + (parsed_starts_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_overage_preference("code-410", "name-371");
        let view = to_primary_overage_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_overage_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
