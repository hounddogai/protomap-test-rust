use super::*;

use acme_kit::text;

/// How screens and exports show a release tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTierView {
    pub code_text: String,
    pub title: String,
    pub quantity: String,
    pub floor_value: String,
    pub duration_minutes: String,
    pub burst: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_release_tier_view(item: &ReleaseTier) -> ReleaseTierView {
    ReleaseTierView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        quantity: item.quantity.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        duration_minutes: item.duration_minutes.to_string(),
        burst: item.burst.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_tier_views(items: &[ReleaseTier]) -> Vec<ReleaseTierView> {
    let mut views: Vec<ReleaseTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_tier_view(item));
    }
    views
}

pub fn sum_release_tier_view(view: &ReleaseTierView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_floor_value + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_tier("code-45", "title-853");
        let view = to_release_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
