use super::*;

/// How screens and exports show a segment preference.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPreferenceView {
    pub code: String,
    pub title_text: String,
    pub cluster: String,
    pub starts_at: String,
    pub weight_text: String,
    pub cipher_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_segment_preference_view(item: &SegmentPreference) -> SegmentPreferenceView {
    SegmentPreferenceView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        cluster: item.cluster.clone(),
        starts_at: item.starts_at.to_string(),
        weight_text: format!("{:.2}", item.weight),
        cipher_text: item.cipher.clone(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_preference_views(items: &[SegmentPreference]) -> Vec<SegmentPreferenceView> {
    let mut views: Vec<SegmentPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_preference_view(item));
    }
    views
}

pub fn sum_segment_preference_view(view: &SegmentPreferenceView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_preference("code-737", "title-795");
        let view = to_segment_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
