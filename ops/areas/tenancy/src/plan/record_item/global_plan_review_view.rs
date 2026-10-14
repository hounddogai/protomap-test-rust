use super::*;

use acme_kit::text;

/// How screens and exports show a global plan review.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewView {
    pub code: String,
    pub name: String,
    pub active_text: String,
    pub version: String,
    pub sequence_text: String,
    pub duration_minutes_text: String,
    pub maximum: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_global_plan_review_view(item: &GlobalPlanReview) -> GlobalPlanReviewView {
    GlobalPlanReviewView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        active_text: format!("{}", item.active),
        version: item.version.to_string(),
        sequence_text: item.sequence.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        maximum: format!("{:.2}", item.maximum),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_plan_review_views(items: &[GlobalPlanReview]) -> Vec<GlobalPlanReviewView> {
    let mut views: Vec<GlobalPlanReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_plan_review_view(item));
    }
    views
}

pub fn sum_global_plan_review_view(view: &GlobalPlanReviewView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_version as f64 + (parsed_sequence as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_plan_review("code-67", "name-423");
        let view = to_global_plan_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_plan_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
