use super::*;

use acme_kit::text;

/// How screens and exports show a batched rollback target.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTargetView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub reviewer: String,
    pub source: String,
    pub visible: String,
    pub attempts: String,
    pub timezone_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_batched_rollback_target_view(item: &BatchedRollbackTarget) -> BatchedRollbackTargetView {
    BatchedRollbackTargetView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        category: item.category.clone(),
        reviewer: item.reviewer.clone(),
        source: text::truncate(&item.source, 80),
        visible: format!("{}", item.visible),
        attempts: item.attempts.to_string(),
        timezone_text: text::truncate(&item.timezone, 80),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_rollback_target_views(items: &[BatchedRollbackTarget]) -> Vec<BatchedRollbackTargetView> {
    let mut views: Vec<BatchedRollbackTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_rollback_target_view(item));
    }
    views
}

pub fn sum_batched_rollback_target_view(view: &BatchedRollbackTargetView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_rollback_target("code-460", "title-774");
        let view = to_batched_rollback_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_rollback_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
