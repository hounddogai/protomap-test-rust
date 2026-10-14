use super::*;

use acme_kit::text;

/// How screens and exports show a batched failover review.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReviewView {
    pub code: String,
    pub name: String,
    pub rate_text: String,
    pub label: String,
    pub ratio_text: String,
    pub host: String,
    pub cost_text: String,
    pub approved: String,
    pub scope: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_batched_failover_review_view(item: &BatchedFailoverReview) -> BatchedFailoverReviewView {
    BatchedFailoverReviewView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        rate_text: format!("{:.2}", item.rate),
        label: item.label.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        host: item.host.clone(),
        cost_text: format!("{:.2}", item.cost),
        approved: format!("{}", item.approved),
        scope: item.scope.clone(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_failover_review_views(items: &[BatchedFailoverReview]) -> Vec<BatchedFailoverReviewView> {
    let mut views: Vec<BatchedFailoverReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_failover_review_view(item));
    }
    views
}

pub fn sum_batched_failover_review_view(view: &BatchedFailoverReviewView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_rate + parsed_ratio + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_failover_review("code-936", "name-571");
        let view = to_batched_failover_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_failover_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
