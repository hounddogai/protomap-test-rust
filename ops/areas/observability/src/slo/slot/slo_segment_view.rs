use super::*;

/// How screens and exports show a slo segment.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSegmentView {
    pub code_text: String,
    pub title: String,
    pub fingerprint: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_slo_segment_view(item: &SloSegment) -> SloSegmentView {
    SloSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_segment_views(items: &[SloSegment]) -> Vec<SloSegmentView> {
    let mut views: Vec<SloSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_segment_view(item));
    }
    views
}

pub fn sum_slo_segment_view(view: &SloSegmentView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_segment("code-97", "title-219");
        let view = to_slo_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
