use super::*;

/// How screens and exports show a fallback segment entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentEntryView {
    pub code: String,
    pub title: String,
    pub version_text: String,
    pub approved: String,
    pub ends_at: String,
    pub quota_units: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_fallback_segment_entry_view(item: &FallbackSegmentEntry) -> FallbackSegmentEntryView {
    FallbackSegmentEntryView {
        code: item.code.clone(),
        title: item.title.clone(),
        version_text: item.version.to_string(),
        approved: format!("{}", item.approved),
        ends_at: item.ends_at.to_string(),
        quota_units: item.quota_units.to_string(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_segment_entry_views(items: &[FallbackSegmentEntry]) -> Vec<FallbackSegmentEntryView> {
    let mut views: Vec<FallbackSegmentEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_segment_entry_view(item));
    }
    views
}

pub fn sum_fallback_segment_entry_view(view: &FallbackSegmentEntryView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_version as f64 + (parsed_ends_at as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_segment_entry("code-677", "title-413");
        let view = to_fallback_segment_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_segment_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
