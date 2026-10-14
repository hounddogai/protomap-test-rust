use super::*;

use acme_kit::text;

/// How screens and exports show a canary allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAllocationView {
    pub code: String,
    pub label: String,
    pub title_text: String,
    pub retries: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_canary_allocation_view(item: &CanaryAllocation) -> CanaryAllocationView {
    CanaryAllocationView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        title_text: item.title.clone(),
        retries: item.retries.to_string(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_allocation_views(items: &[CanaryAllocation]) -> Vec<CanaryAllocationView> {
    let mut views: Vec<CanaryAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_allocation_view(item));
    }
    views
}

pub fn sum_canary_allocation_view(view: &CanaryAllocationView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_allocation("code-886", "label-427");
        let view = to_canary_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
