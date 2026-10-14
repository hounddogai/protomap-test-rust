use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive alert score.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertScoreView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub duration_minutes_text: String,
    pub label: String,
    pub active: String,
    pub channel: String,
    pub cluster: String,
    pub amount: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_adaptive_alert_score_view(item: &AdaptiveAlertScore) -> AdaptiveAlertScoreView {
    AdaptiveAlertScoreView {
        code: item.code.clone(),
        title: item.title.clone(),
        owner_id: text::truncate(&item.owner_id, 64),
        duration_minutes_text: item.duration_minutes.to_string(),
        label: item.label.clone(),
        active: format!("{}", item.active),
        channel: text::truncate(&item.channel, 120),
        cluster: item.cluster.clone(),
        amount: format!("{:.2}", item.amount),
        currency: text::truncate(&item.currency, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_alert_score_views(items: &[AdaptiveAlertScore]) -> Vec<AdaptiveAlertScoreView> {
    let mut views: Vec<AdaptiveAlertScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_alert_score_view(item));
    }
    views
}

pub fn sum_adaptive_alert_score_view(view: &AdaptiveAlertScoreView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_alert_score("code-409", "title-951");
        let view = to_adaptive_alert_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_alert_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
