use super::*;

use acme_kit::text;

/// How screens and exports show a weighted failover segment.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSegmentView {
    pub code_text: String,
    pub title: String,
    pub fingerprint: String,
    pub checksum: String,
    pub name_text: String,
    pub limit: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_weighted_failover_segment_view(item: &WeightedFailoverSegment) -> WeightedFailoverSegmentView {
    WeightedFailoverSegmentView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        fingerprint: item.fingerprint.clone(),
        checksum: text::truncate(&item.checksum, 64),
        name_text: item.name.clone(),
        limit: item.limit.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_failover_segment_views(items: &[WeightedFailoverSegment]) -> Vec<WeightedFailoverSegmentView> {
    let mut views: Vec<WeightedFailoverSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_failover_segment_view(item));
    }
    views
}

pub fn sum_weighted_failover_segment_view(view: &WeightedFailoverSegmentView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_failover_segment("code-85", "title-5");
        let view = to_weighted_failover_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_failover_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
