use super::*;

use acme_kit::text;

/// How screens and exports show a rolling timeline plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePlanView {
    pub code: String,
    pub name_text: String,
    pub ratio: String,
    pub updated_at: String,
    pub phone: String,
    pub threshold: String,
    pub sequence: String,
    pub key_id: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_rolling_timeline_plan_view(item: &RollingTimelinePlan) -> RollingTimelinePlanView {
    RollingTimelinePlanView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        ratio: format!("{:.2}", item.ratio),
        updated_at: item.updated_at.to_string(),
        phone: item.phone.clone(),
        threshold: format!("{:.2}", item.threshold),
        sequence: item.sequence.to_string(),
        key_id: item.key_id.clone(),
        region_text: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_timeline_plan_views(items: &[RollingTimelinePlan]) -> Vec<RollingTimelinePlanView> {
    let mut views: Vec<RollingTimelinePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_timeline_plan_view(item));
    }
    views
}

pub fn sum_rolling_timeline_plan_view(view: &RollingTimelinePlanView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ratio + (parsed_updated_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_timeline_plan("code-362", "name-127");
        let view = to_rolling_timeline_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_timeline_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
