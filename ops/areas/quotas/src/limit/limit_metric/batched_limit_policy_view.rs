use super::*;

use acme_kit::text;

/// How screens and exports show a batched limit policy.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyView {
    pub code: String,
    pub name_text: String,
    pub starts_at: String,
    pub minimum: String,
    pub discount: String,
    pub currency: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_batched_limit_policy_view(item: &BatchedLimitPolicy) -> BatchedLimitPolicyView {
    BatchedLimitPolicyView {
        code: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 40),
        starts_at: item.starts_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        discount: format!("{:.2}", item.discount),
        currency: item.currency.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_limit_policy_views(items: &[BatchedLimitPolicy]) -> Vec<BatchedLimitPolicyView> {
    let mut views: Vec<BatchedLimitPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_limit_policy_view(item));
    }
    views
}

pub fn sum_batched_limit_policy_view(view: &BatchedLimitPolicyView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_minimum + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_limit_policy("code-740", "name-368");
        let view = to_batched_limit_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_limit_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
