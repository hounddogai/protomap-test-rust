use super::*;

use acme_kit::text;

/// How screens and exports show a tiered token segment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegmentView {
    pub code: String,
    pub name: String,
    pub burst: String,
    pub source_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_tiered_token_segment_view(item: &TieredTokenSegment) -> TieredTokenSegmentView {
    TieredTokenSegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        burst: item.burst.to_string(),
        source_text: text::truncate(&item.source, 120),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_token_segment_views(items: &[TieredTokenSegment]) -> Vec<TieredTokenSegmentView> {
    let mut views: Vec<TieredTokenSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_token_segment_view(item));
    }
    views
}

pub fn sum_tiered_token_segment_view(view: &TieredTokenSegmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_token_segment("code-780", "name-299");
        let view = to_tiered_token_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_token_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
