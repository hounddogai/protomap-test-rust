use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled delegation score.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationScoreView {
    pub code_text: String,
    pub label: String,
    pub currency: String,
    pub channel_text: String,
    pub rotation_days_text: String,
    pub ratio: String,
    pub owner_id_text: String,
    pub description: String,
    pub timezone_text: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_scheduled_delegation_score_view(item: &ScheduledDelegationScore) -> ScheduledDelegationScoreView {
    ScheduledDelegationScoreView {
        code_text: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 120),
        currency: text::truncate(&item.currency, 40),
        channel_text: text::truncate(&item.channel, 64),
        rotation_days_text: item.rotation_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        owner_id_text: item.owner_id.clone(),
        description: text::truncate(&item.description, 120),
        timezone_text: item.timezone.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_delegation_score_views(items: &[ScheduledDelegationScore]) -> Vec<ScheduledDelegationScoreView> {
    let mut views: Vec<ScheduledDelegationScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_delegation_score_view(item));
    }
    views
}

pub fn sum_scheduled_delegation_score_view(view: &ScheduledDelegationScoreView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_ratio + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_delegation_score("code-928", "label-443");
        let view = to_scheduled_delegation_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_delegation_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
