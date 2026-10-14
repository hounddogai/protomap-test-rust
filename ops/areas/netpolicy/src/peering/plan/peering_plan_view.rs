use super::*;

use acme_kit::text;

/// How screens and exports show a peering plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPlanView {
    pub code_text: String,
    pub title: String,
    pub channel: String,
    pub audience_text: String,
    pub active: String,
    pub currency: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_peering_plan_view(item: &PeeringPlan) -> PeeringPlanView {
    PeeringPlanView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        channel: item.channel.clone(),
        audience_text: text::truncate(&item.audience, 40),
        active: format!("{}", item.active),
        currency: text::truncate(&item.currency, 120),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_plan_views(items: &[PeeringPlan]) -> Vec<PeeringPlanView> {
    let mut views: Vec<PeeringPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_plan_view(item));
    }
    views
}

pub fn sum_peering_plan_view(view: &PeeringPlanView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_plan("code-355", "title-514");
        let view = to_peering_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
