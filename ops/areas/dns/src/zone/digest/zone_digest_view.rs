use super::*;

use acme_kit::text;

/// How screens and exports show a zone digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestView {
    pub code: String,
    pub title: String,
    pub minimum_text: String,
    pub fingerprint: String,
    pub ends_at_text: String,
    pub summary_text: String,
    pub active: String,
    pub reviewer_text: String,
    pub version_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_zone_digest_view(item: &ZoneDigest) -> ZoneDigestView {
    ZoneDigestView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        fingerprint: item.fingerprint.clone(),
        ends_at_text: item.ends_at.to_string(),
        summary_text: text::truncate(&item.summary, 80),
        active: format!("{}", item.active),
        reviewer_text: item.reviewer.clone(),
        version_text: item.version.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_digest_views(items: &[ZoneDigest]) -> Vec<ZoneDigestView> {
    let mut views: Vec<ZoneDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_digest_view(item));
    }
    views
}

pub fn sum_zone_digest_view(view: &ZoneDigestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_minimum + (parsed_ends_at as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_digest("code-118", "title-245");
        let view = to_zone_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
