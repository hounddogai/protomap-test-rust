use super::*;

use acme_kit::text;

/// How screens and exports show a manual segment entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentEntryView {
    pub code: String,
    pub name: String,
    pub version_text: String,
    pub segment_text: String,
    pub visible: String,
    pub currency: String,
    pub starts_at: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_manual_segment_entry_view(item: &ManualSegmentEntry) -> ManualSegmentEntryView {
    ManualSegmentEntryView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        version_text: item.version.to_string(),
        segment_text: text::truncate(&item.segment, 80),
        visible: format!("{}", item.visible),
        currency: item.currency.clone(),
        starts_at: item.starts_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_segment_entry_views(items: &[ManualSegmentEntry]) -> Vec<ManualSegmentEntryView> {
    let mut views: Vec<ManualSegmentEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_segment_entry_view(item));
    }
    views
}

pub fn sum_manual_segment_entry_view(view: &ManualSegmentEntryView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_version as f64 + (parsed_starts_at as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_segment_entry("code-186", "name-224");
        let view = to_manual_segment_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_segment_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
