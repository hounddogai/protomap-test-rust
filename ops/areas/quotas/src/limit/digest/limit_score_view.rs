use super::*;

use acme_kit::text;

/// How screens and exports show a limit score.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitScoreView {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub attempts: String,
    pub batch_size: String,
    pub ttl_seconds: String,
    pub maximum: String,
    pub ratio_text: String,
    pub window_days: String,
    pub margin: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_limit_score_view(item: &LimitScore) -> LimitScoreView {
    LimitScoreView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        segment: text::truncate(&item.segment, 120),
        attempts: item.attempts.to_string(),
        batch_size: item.batch_size.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        maximum: format!("{:.2}", item.maximum),
        ratio_text: format!("{:.2}", item.ratio),
        window_days: item.window_days.to_string(),
        margin: format!("{:.2}", item.margin),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_score_views(items: &[LimitScore]) -> Vec<LimitScoreView> {
    let mut views: Vec<LimitScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_score_view(item));
    }
    views
}

pub fn sum_limit_score_view(view: &LimitScoreView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_batch_size as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_score("code-998", "label-419");
        let view = to_limit_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
