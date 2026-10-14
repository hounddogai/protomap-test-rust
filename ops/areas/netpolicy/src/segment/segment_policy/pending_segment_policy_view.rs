use super::*;

use acme_kit::text;

/// How screens and exports show a pending segment policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPolicyView {
    pub code_text: String,
    pub label: String,
    pub source: String,
    pub pinned: String,
    pub discount_text: String,
    pub phone: String,
    pub ends_at: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_pending_segment_policy_view(item: &PendingSegmentPolicy) -> PendingSegmentPolicyView {
    PendingSegmentPolicyView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        source: text::truncate(&item.source, 64),
        pinned: format!("{}", item.pinned),
        discount_text: format!("{:.2}", item.discount),
        phone: item.phone.clone(),
        ends_at: item.ends_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_segment_policy_views(items: &[PendingSegmentPolicy]) -> Vec<PendingSegmentPolicyView> {
    let mut views: Vec<PendingSegmentPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_segment_policy_view(item));
    }
    views
}

pub fn sum_pending_segment_policy_view(view: &PendingSegmentPolicyView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_discount + (parsed_ends_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_segment_policy("code-791", "label-986");
        let view = to_pending_segment_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_segment_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
