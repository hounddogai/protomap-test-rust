use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive binding milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingMilestoneView {
    pub code_text: String,
    pub title: String,
    pub approved: String,
    pub ratio: String,
    pub ceiling: String,
    pub audience: String,
    pub currency_text: String,
    pub channel_text: String,
    pub issuer: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_adaptive_binding_milestone_view(item: &AdaptiveBindingMilestone) -> AdaptiveBindingMilestoneView {
    AdaptiveBindingMilestoneView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        ratio: format!("{:.2}", item.ratio),
        ceiling: format!("{:.2}", item.ceiling),
        audience: item.audience.clone(),
        currency_text: item.currency.clone(),
        channel_text: text::truncate(&item.channel, 80),
        issuer: item.issuer.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_binding_milestone_views(items: &[AdaptiveBindingMilestone]) -> Vec<AdaptiveBindingMilestoneView> {
    let mut views: Vec<AdaptiveBindingMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_binding_milestone_view(item));
    }
    views
}

pub fn sum_adaptive_binding_milestone_view(view: &AdaptiveBindingMilestoneView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ratio + parsed_ceiling + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_binding_milestone("code-828", "title-709");
        let view = to_adaptive_binding_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_binding_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
