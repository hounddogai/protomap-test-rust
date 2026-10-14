use super::*;

use acme_kit::text;

/// How screens and exports show a health check score.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScoreView {
    pub code: String,
    pub name: String,
    pub threshold: String,
    pub sequence: String,
    pub key_id: String,
    pub region: String,
    pub ends_at: String,
    pub retries: String,
    pub burst_text: String,
    pub principal: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_health_check_score_view(item: &HealthCheckScore) -> HealthCheckScoreView {
    HealthCheckScoreView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        threshold: format!("{:.2}", item.threshold),
        sequence: item.sequence.to_string(),
        key_id: item.key_id.clone(),
        region: text::truncate(&item.region, 40),
        ends_at: item.ends_at.to_string(),
        retries: item.retries.to_string(),
        burst_text: item.burst.to_string(),
        principal: item.principal.clone(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_score_views(items: &[HealthCheckScore]) -> Vec<HealthCheckScoreView> {
    let mut views: Vec<HealthCheckScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_score_view(item));
    }
    views
}

pub fn sum_health_check_score_view(view: &HealthCheckScoreView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_threshold + (parsed_sequence as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_score("code-372", "name-340");
        let view = to_health_check_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
