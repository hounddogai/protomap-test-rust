use super::*;

/// How screens and exports show a adaptive probe summary.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeSummaryView {
    pub code: String,
    pub title: String,
    pub pinned: String,
    pub sequence: String,
    pub quota_units: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_adaptive_probe_summary_view(item: &AdaptiveProbeSummary) -> AdaptiveProbeSummaryView {
    AdaptiveProbeSummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        pinned: format!("{}", item.pinned),
        sequence: item.sequence.to_string(),
        quota_units: item.quota_units.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_probe_summary_views(items: &[AdaptiveProbeSummary]) -> Vec<AdaptiveProbeSummaryView> {
    let mut views: Vec<AdaptiveProbeSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_probe_summary_view(item));
    }
    views
}

pub fn sum_adaptive_probe_summary_view(view: &AdaptiveProbeSummaryView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_probe_summary("code-78", "title-26");
        let view = to_adaptive_probe_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_probe_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
