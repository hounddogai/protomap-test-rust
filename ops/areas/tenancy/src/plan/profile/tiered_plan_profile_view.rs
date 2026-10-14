use super::*;

use acme_kit::text;

/// How screens and exports show a tiered plan profile.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanProfileView {
    pub code: String,
    pub label: String,
    pub revision_text: String,
    pub weight: String,
    pub fingerprint: String,
    pub phone: String,
    pub owner_id: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_tiered_plan_profile_view(item: &TieredPlanProfile) -> TieredPlanProfileView {
    TieredPlanProfileView {
        code: item.code.clone(),
        label: item.label.clone(),
        revision_text: item.revision.to_string(),
        weight: format!("{:.2}", item.weight),
        fingerprint: item.fingerprint.clone(),
        phone: item.phone.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_plan_profile_views(items: &[TieredPlanProfile]) -> Vec<TieredPlanProfileView> {
    let mut views: Vec<TieredPlanProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_plan_profile_view(item));
    }
    views
}

pub fn sum_tiered_plan_profile_view(view: &TieredPlanProfileView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_revision as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_plan_profile("code-272", "label-602");
        let view = to_tiered_plan_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_plan_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
