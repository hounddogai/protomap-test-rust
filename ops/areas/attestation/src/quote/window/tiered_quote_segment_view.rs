use super::*;

/// How screens and exports show a tiered quote segment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegmentView {
    pub code_text: String,
    pub label: String,
    pub grace_days: String,
    pub currency: String,
    pub active: String,
    pub retries_text: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_tiered_quote_segment_view(item: &TieredQuoteSegment) -> TieredQuoteSegmentView {
    TieredQuoteSegmentView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        currency: item.currency.clone(),
        active: format!("{}", item.active),
        retries_text: item.retries.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_quote_segment_views(items: &[TieredQuoteSegment]) -> Vec<TieredQuoteSegmentView> {
    let mut views: Vec<TieredQuoteSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_quote_segment_view(item));
    }
    views
}

pub fn sum_tiered_quote_segment_view(view: &TieredQuoteSegmentView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_retries as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_quote_segment("code-665", "label-35");
        let view = to_tiered_quote_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_quote_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
