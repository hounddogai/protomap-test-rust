use super::*;

use acme_kit::text;

/// How screens and exports show a fallback plan target.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTargetView {
    pub code: String,
    pub name_text: String,
    pub phone: String,
    pub contact_email: String,
    pub namespace: String,
    pub burst: String,
    pub quantity: String,
    pub grace_days: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_fallback_plan_target_view(item: &FallbackPlanTarget) -> FallbackPlanTargetView {
    FallbackPlanTargetView {
        code: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 40),
        phone: text::truncate(&item.phone, 120),
        contact_email: item.contact_email.clone(),
        namespace: item.namespace.clone(),
        burst: item.burst.to_string(),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_plan_target_views(items: &[FallbackPlanTarget]) -> Vec<FallbackPlanTargetView> {
    let mut views: Vec<FallbackPlanTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_plan_target_view(item));
    }
    views
}

pub fn sum_fallback_plan_target_view(view: &FallbackPlanTargetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_quantity as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_plan_target("code-851", "name-559");
        let view = to_fallback_plan_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_plan_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
