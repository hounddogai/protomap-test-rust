use super::*;

use acme_kit::text;

/// How screens and exports show a plan review.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanReviewView {
    pub code_text: String,
    pub title: String,
    pub weight: String,
    pub audience: String,
    pub currency: String,
    pub source: String,
    pub cluster: String,
    pub priority: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_plan_review_view(item: &PlanReview) -> PlanReviewView {
    PlanReviewView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        weight: format!("{:.2}", item.weight),
        audience: item.audience.clone(),
        currency: item.currency.clone(),
        source: item.source.clone(),
        cluster: text::truncate(&item.cluster, 120),
        priority: item.priority.to_string(),
        country_text: text::truncate(&item.country, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_review_views(items: &[PlanReview]) -> Vec<PlanReviewView> {
    let mut views: Vec<PlanReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_review_view(item));
    }
    views
}

pub fn sum_plan_review_view(view: &PlanReviewView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_review("code-611", "title-745");
        let view = to_plan_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
