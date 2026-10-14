use super::*;

use acme_kit::text;

/// How screens and exports show a tiered zone plan.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlanView {
    pub code: String,
    pub title_text: String,
    pub owner_id: String,
    pub duration_minutes: String,
    pub created_at: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_tiered_zone_plan_view(item: &TieredZonePlan) -> TieredZonePlanView {
    TieredZonePlanView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        owner_id: item.owner_id.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        created_at: item.created_at.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_zone_plan_views(items: &[TieredZonePlan]) -> Vec<TieredZonePlanView> {
    let mut views: Vec<TieredZonePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_zone_plan_view(item));
    }
    views
}

pub fn sum_tiered_zone_plan_view(view: &TieredZonePlanView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_created_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_zone_plan("code-418", "title-152");
        let view = to_tiered_zone_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_zone_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
