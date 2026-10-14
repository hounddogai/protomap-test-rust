use super::*;

use acme_kit::text;

/// How screens and exports show a batched suspension segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegmentView {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub checksum: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_batched_suspension_segment_view(item: &BatchedSuspensionSegment) -> BatchedSuspensionSegmentView {
    BatchedSuspensionSegmentView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        scope: item.scope.clone(),
        checksum: text::truncate(&item.checksum, 64),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_suspension_segment_views(items: &[BatchedSuspensionSegment]) -> Vec<BatchedSuspensionSegmentView> {
    let mut views: Vec<BatchedSuspensionSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_suspension_segment_view(item));
    }
    views
}

pub fn sum_batched_suspension_segment_view(view: &BatchedSuspensionSegmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_suspension_segment("code-142", "label-140");
        let view = to_batched_suspension_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_suspension_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
