use super::*;

use acme_kit::text;

/// How screens and exports show a zone draft.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftView {
    pub code: String,
    pub title_text: String,
    pub fingerprint_text: String,
    pub unit: String,
    pub version: String,
    pub minimum: String,
    pub rotation_days: String,
    pub starts_at: String,
    pub ttl_seconds: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_zone_draft_view(item: &ZoneDraft) -> ZoneDraftView {
    ZoneDraftView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        fingerprint_text: item.fingerprint.clone(),
        unit: text::truncate(&item.unit, 64),
        version: item.version.to_string(),
        minimum: format!("{:.2}", item.minimum),
        rotation_days: item.rotation_days.to_string(),
        starts_at: item.starts_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_draft_views(items: &[ZoneDraft]) -> Vec<ZoneDraftView> {
    let mut views: Vec<ZoneDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_draft_view(item));
    }
    views
}

pub fn sum_zone_draft_view(view: &ZoneDraftView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_version as f64 + parsed_minimum + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_draft("code-520", "title-882");
        let view = to_zone_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
