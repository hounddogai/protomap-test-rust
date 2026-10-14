use super::*;

/// How screens and exports show a batched failover score.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverScoreView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub burst_text: String,
    pub label: String,
    pub contact_email: String,
    pub priority: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_batched_failover_score_view(item: &BatchedFailoverScore) -> BatchedFailoverScoreView {
    BatchedFailoverScoreView {
        code: item.code.clone(),
        title: item.title.clone(),
        owner_id: item.owner_id.clone(),
        burst_text: item.burst.to_string(),
        label: item.label.clone(),
        contact_email: item.contact_email.clone(),
        priority: item.priority.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_failover_score_views(items: &[BatchedFailoverScore]) -> Vec<BatchedFailoverScoreView> {
    let mut views: Vec<BatchedFailoverScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_failover_score_view(item));
    }
    views
}

pub fn sum_batched_failover_score_view(view: &BatchedFailoverScoreView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_failover_score("code-367", "title-416");
        let view = to_batched_failover_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_failover_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
