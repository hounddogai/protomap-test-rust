use super::*;

use acme_kit::text;

/// How screens and exports show a segment batch.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBatchView {
    pub code: String,
    pub name_text: String,
    pub category: String,
    pub issuer: String,
    pub maximum: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_segment_batch_view(item: &SegmentBatch) -> SegmentBatchView {
    SegmentBatchView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        category: item.category.clone(),
        issuer: item.issuer.clone(),
        maximum: format!("{:.2}", item.maximum),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_batch_views(items: &[SegmentBatch]) -> Vec<SegmentBatchView> {
    let mut views: Vec<SegmentBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_batch_view(item));
    }
    views
}

pub fn sum_segment_batch_view(view: &SegmentBatchView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_maximum + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_batch("code-297", "name-350");
        let view = to_segment_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
