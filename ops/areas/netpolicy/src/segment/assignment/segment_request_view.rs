use super::*;

use acme_kit::text;

/// How screens and exports show a segment request.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRequestView {
    pub code: String,
    pub label_text: String,
    pub window_days_text: String,
    pub principal: String,
    pub margin_text: String,
    pub batch_size_text: String,
    pub starts_at: String,
    pub burst_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_segment_request_view(item: &SegmentRequest) -> SegmentRequestView {
    SegmentRequestView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        window_days_text: item.window_days.to_string(),
        principal: item.principal.clone(),
        margin_text: format!("{:.2}", item.margin),
        batch_size_text: item.batch_size.to_string(),
        starts_at: item.starts_at.to_string(),
        burst_text: item.burst.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_request_views(items: &[SegmentRequest]) -> Vec<SegmentRequestView> {
    let mut views: Vec<SegmentRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_request_view(item));
    }
    views
}

pub fn sum_segment_request_view(view: &SegmentRequestView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_margin + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_request("code-573", "label-528");
        let view = to_segment_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
