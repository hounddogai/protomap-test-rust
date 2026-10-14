use super::*;

use acme_kit::text;

/// How screens and exports show a fallback usage allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageAllocationView {
    pub code: String,
    pub title_text: String,
    pub updated_at_text: String,
    pub namespace: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_fallback_usage_allocation_view(item: &FallbackUsageAllocation) -> FallbackUsageAllocationView {
    FallbackUsageAllocationView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        updated_at_text: item.updated_at.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_usage_allocation_views(items: &[FallbackUsageAllocation]) -> Vec<FallbackUsageAllocationView> {
    let mut views: Vec<FallbackUsageAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_usage_allocation_view(item));
    }
    views
}

pub fn sum_fallback_usage_allocation_view(view: &FallbackUsageAllocationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_usage_allocation("code-830", "title-514");
        let view = to_fallback_usage_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_usage_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
