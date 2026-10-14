use super::*;

use acme_kit::text;

/// How screens and exports show a global segment item.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentItemView {
    pub code: String,
    pub name: String,
    pub expires_at: String,
    pub quota_units_text: String,
    pub description: String,
    pub port: String,
    pub starts_at: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_global_segment_item_view(item: &GlobalSegmentItem) -> GlobalSegmentItemView {
    GlobalSegmentItemView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 80),
        expires_at: item.expires_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        description: text::truncate(&item.description, 40),
        port: item.port.to_string(),
        starts_at: item.starts_at.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_segment_item_views(items: &[GlobalSegmentItem]) -> Vec<GlobalSegmentItemView> {
    let mut views: Vec<GlobalSegmentItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_segment_item_view(item));
    }
    views
}

pub fn sum_global_segment_item_view(view: &GlobalSegmentItemView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_quota_units as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_segment_item("code-725", "name-186");
        let view = to_global_segment_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_segment_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
