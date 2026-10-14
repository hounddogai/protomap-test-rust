use super::*;

/// How screens and exports show a adaptive quote segment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegmentView {
    pub code: String,
    pub title_text: String,
    pub budget_text: String,
    pub threshold: String,
    pub contact_email: String,
    pub retries: String,
    pub quota_units_text: String,
    pub summary: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_adaptive_quote_segment_view(item: &AdaptiveQuoteSegment) -> AdaptiveQuoteSegmentView {
    AdaptiveQuoteSegmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        budget_text: format!("{:.2}", item.budget),
        threshold: format!("{:.2}", item.threshold),
        contact_email: item.contact_email.clone(),
        retries: item.retries.to_string(),
        quota_units_text: item.quota_units.to_string(),
        summary: item.summary.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_quote_segment_views(items: &[AdaptiveQuoteSegment]) -> Vec<AdaptiveQuoteSegmentView> {
    let mut views: Vec<AdaptiveQuoteSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_quote_segment_view(item));
    }
    views
}

pub fn sum_adaptive_quote_segment_view(view: &AdaptiveQuoteSegmentView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_budget + parsed_threshold + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_quote_segment("code-841", "title-291");
        let view = to_adaptive_quote_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_quote_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
