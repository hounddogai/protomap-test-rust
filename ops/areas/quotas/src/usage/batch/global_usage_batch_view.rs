use super::*;

use acme_kit::text;

/// How screens and exports show a global usage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageBatchView {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub cost: String,
    pub description: String,
    pub attempts: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_global_usage_batch_view(item: &GlobalUsageBatch) -> GlobalUsageBatchView {
    GlobalUsageBatchView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        contact_email: item.contact_email.clone(),
        cost: format!("{:.2}", item.cost),
        description: item.description.clone(),
        attempts: item.attempts.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_usage_batch_views(items: &[GlobalUsageBatch]) -> Vec<GlobalUsageBatchView> {
    let mut views: Vec<GlobalUsageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_usage_batch_view(item));
    }
    views
}

pub fn sum_global_usage_batch_view(view: &GlobalUsageBatchView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_usage_batch("code-145", "label-93");
        let view = to_global_usage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_usage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
