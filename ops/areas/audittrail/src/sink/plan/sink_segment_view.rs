use super::*;

use acme_kit::text;

/// How screens and exports show a sink segment.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegmentView {
    pub code_text: String,
    pub label_text: String,
    pub channel: String,
    pub updated_at: String,
    pub locale_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_sink_segment_view(item: &SinkSegment) -> SinkSegmentView {
    SinkSegmentView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        channel: item.channel.clone(),
        updated_at: item.updated_at.to_string(),
        locale_text: item.locale.clone(),
        reviewer: text::truncate(&item.reviewer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_segment_views(items: &[SinkSegment]) -> Vec<SinkSegmentView> {
    let mut views: Vec<SinkSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_segment_view(item));
    }
    views
}

pub fn sum_sink_segment_view(view: &SinkSegmentView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_segment("code-273", "label-680");
        let view = to_sink_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
