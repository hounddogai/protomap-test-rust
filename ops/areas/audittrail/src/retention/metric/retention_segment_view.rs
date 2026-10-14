use super::*;

/// How screens and exports show a retention segment.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSegmentView {
    pub code_text: String,
    pub label_text: String,
    pub minimum: String,
    pub key_id_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_retention_segment_view(item: &RetentionSegment) -> RetentionSegmentView {
    RetentionSegmentView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        minimum: format!("{:.2}", item.minimum),
        key_id_text: item.key_id.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_segment_views(items: &[RetentionSegment]) -> Vec<RetentionSegmentView> {
    let mut views: Vec<RetentionSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_segment_view(item));
    }
    views
}

pub fn sum_retention_segment_view(view: &RetentionSegmentView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_minimum + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_segment("code-271", "label-461");
        let view = to_retention_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
