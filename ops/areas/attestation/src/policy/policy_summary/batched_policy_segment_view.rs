use super::*;

/// How screens and exports show a batched policy segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicySegmentView {
    pub code: String,
    pub title: String,
    pub burst: String,
    pub revision: String,
    pub source_text: String,
    pub threshold: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_batched_policy_segment_view(item: &BatchedPolicySegment) -> BatchedPolicySegmentView {
    BatchedPolicySegmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        burst: item.burst.to_string(),
        revision: item.revision.to_string(),
        source_text: item.source.clone(),
        threshold: format!("{:.2}", item.threshold),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_segment_views(items: &[BatchedPolicySegment]) -> Vec<BatchedPolicySegmentView> {
    let mut views: Vec<BatchedPolicySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_segment_view(item));
    }
    views
}

pub fn sum_batched_policy_segment_view(view: &BatchedPolicySegmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_revision as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_policy_segment("code-235", "title-474");
        let view = to_batched_policy_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_policy_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
