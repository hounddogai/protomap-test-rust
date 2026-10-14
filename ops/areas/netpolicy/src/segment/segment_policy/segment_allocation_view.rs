use super::*;

use acme_kit::text;

/// How screens and exports show a segment allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocationView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub ends_at: String,
    pub grace_days_text: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_segment_allocation_view(item: &SegmentAllocation) -> SegmentAllocationView {
    SegmentAllocationView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        ends_at: item.ends_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_allocation_views(items: &[SegmentAllocation]) -> Vec<SegmentAllocationView> {
    let mut views: Vec<SegmentAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_allocation_view(item));
    }
    views
}

pub fn sum_segment_allocation_view(view: &SegmentAllocationView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_budget + (parsed_ends_at as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_allocation("code-650", "title-708");
        let view = to_segment_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
