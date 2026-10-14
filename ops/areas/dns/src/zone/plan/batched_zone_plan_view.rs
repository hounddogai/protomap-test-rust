use super::*;

use acme_kit::text;

/// How screens and exports show a batched zone plan.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZonePlanView {
    pub code: String,
    pub label: String,
    pub updated_at_text: String,
    pub margin: String,
    pub category: String,
    pub timezone_text: String,
    pub key_id_text: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_batched_zone_plan_view(item: &BatchedZonePlan) -> BatchedZonePlanView {
    BatchedZonePlanView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        updated_at_text: item.updated_at.to_string(),
        margin: format!("{:.2}", item.margin),
        category: item.category.clone(),
        timezone_text: item.timezone.clone(),
        key_id_text: item.key_id.clone(),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_zone_plan_views(items: &[BatchedZonePlan]) -> Vec<BatchedZonePlanView> {
    let mut views: Vec<BatchedZonePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_zone_plan_view(item));
    }
    views
}

pub fn sum_batched_zone_plan_view(view: &BatchedZonePlanView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_zone_plan("code-202", "label-142");
        let view = to_batched_zone_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_zone_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
