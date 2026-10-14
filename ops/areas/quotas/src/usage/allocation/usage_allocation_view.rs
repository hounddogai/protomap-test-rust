use super::*;

use acme_kit::text;

/// How screens and exports show a usage allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAllocationView {
    pub code: String,
    pub title: String,
    pub timezone_text: String,
    pub sequence_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_usage_allocation_view(item: &UsageAllocation) -> UsageAllocationView {
    UsageAllocationView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        timezone_text: item.timezone.clone(),
        sequence_text: item.sequence.to_string(),
        category: text::truncate(&item.category, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_allocation_views(items: &[UsageAllocation]) -> Vec<UsageAllocationView> {
    let mut views: Vec<UsageAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_allocation_view(item));
    }
    views
}

pub fn sum_usage_allocation_view(view: &UsageAllocationView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_allocation("code-641", "title-668");
        let view = to_usage_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
