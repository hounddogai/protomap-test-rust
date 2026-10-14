use super::*;

use acme_kit::text;

/// How screens and exports show a tiered plan draft.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanDraftView {
    pub code: String,
    pub label_text: String,
    pub region_text: String,
    pub principal: String,
    pub duration_minutes: String,
    pub grace_days_text: String,
    pub maximum: String,
    pub ttl_seconds: String,
    pub cluster_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_tiered_plan_draft_view(item: &TieredPlanDraft) -> TieredPlanDraftView {
    TieredPlanDraftView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        region_text: item.region.clone(),
        principal: item.principal.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        grace_days_text: item.grace_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        ttl_seconds: item.ttl_seconds.to_string(),
        cluster_text: item.cluster.clone(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_plan_draft_views(items: &[TieredPlanDraft]) -> Vec<TieredPlanDraftView> {
    let mut views: Vec<TieredPlanDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_plan_draft_view(item));
    }
    views
}

pub fn sum_tiered_plan_draft_view(view: &TieredPlanDraftView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_grace_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_plan_draft("code-540", "label-171");
        let view = to_tiered_plan_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_plan_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
