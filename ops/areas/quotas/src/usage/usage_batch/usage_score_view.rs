use super::*;

use acme_kit::text;

/// How screens and exports show a usage score.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageScoreView {
    pub code_text: String,
    pub label_text: String,
    pub timezone: String,
    pub host: String,
    pub burst_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_usage_score_view(item: &UsageScore) -> UsageScoreView {
    UsageScoreView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        timezone: text::truncate(&item.timezone, 120),
        host: item.host.clone(),
        burst_text: item.burst.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_score_views(items: &[UsageScore]) -> Vec<UsageScoreView> {
    let mut views: Vec<UsageScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_score_view(item));
    }
    views
}

pub fn sum_usage_score_view(view: &UsageScoreView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_score("code-383", "label-121");
        let view = to_usage_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
