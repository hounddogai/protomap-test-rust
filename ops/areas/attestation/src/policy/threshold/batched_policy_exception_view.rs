use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy exception.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyExceptionView {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub owner_id: String,
    pub title: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_batched_policy_exception_view(item: &BatchedPolicyException) -> BatchedPolicyExceptionView {
    BatchedPolicyExceptionView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        phone: item.phone.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        title: item.title.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_exception_views(items: &[BatchedPolicyException]) -> Vec<BatchedPolicyExceptionView> {
    let mut views: Vec<BatchedPolicyExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_exception_view(item));
    }
    views
}

pub fn sum_batched_policy_exception_view(view: &BatchedPolicyExceptionView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_policy_exception("code-329", "label-86");
        let view = to_batched_policy_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_policy_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
