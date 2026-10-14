use super::*;

use acme_kit::text;

/// How screens and exports show a weighted sink digest.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSinkDigestView {
    pub code: String,
    pub label: String,
    pub capacity: String,
    pub description: String,
    pub principal: String,
    pub attempts: String,
    pub budget_text: String,
    pub port: String,
    pub batch_size: String,
    pub summary_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_weighted_sink_digest_view(item: &WeightedSinkDigest) -> WeightedSinkDigestView {
    WeightedSinkDigestView {
        code: item.code.clone(),
        label: item.label.clone(),
        capacity: item.capacity.to_string(),
        description: item.description.clone(),
        principal: text::truncate(&item.principal, 64),
        attempts: item.attempts.to_string(),
        budget_text: format!("{:.2}", item.budget),
        port: item.port.to_string(),
        batch_size: item.batch_size.to_string(),
        summary_text: text::truncate(&item.summary, 40),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_sink_digest_views(items: &[WeightedSinkDigest]) -> Vec<WeightedSinkDigestView> {
    let mut views: Vec<WeightedSinkDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_sink_digest_view(item));
    }
    views
}

pub fn sum_weighted_sink_digest_view(view: &WeightedSinkDigestView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_attempts as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_sink_digest("code-94", "label-427");
        let view = to_weighted_sink_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_sink_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
