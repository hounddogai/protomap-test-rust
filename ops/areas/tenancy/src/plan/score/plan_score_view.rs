use super::*;

use acme_kit::text;

/// How screens and exports show a plan score.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanScoreView {
    pub code: String,
    pub name: String,
    pub created_at: String,
    pub ratio: String,
    pub maximum: String,
    pub active: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_plan_score_view(item: &PlanScore) -> PlanScoreView {
    PlanScoreView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        created_at: item.created_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        maximum: format!("{:.2}", item.maximum),
        active: format!("{}", item.active),
        namespace: text::truncate(&item.namespace, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_score_views(items: &[PlanScore]) -> Vec<PlanScoreView> {
    let mut views: Vec<PlanScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_score_view(item));
    }
    views
}

pub fn sum_plan_score_view(view: &PlanScoreView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_ratio + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_score("code-407", "name-896");
        let view = to_plan_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
