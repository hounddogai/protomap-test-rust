use super::*;

use acme_kit::text;

/// How screens and exports show a zone bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBundleView {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub window_days_text: String,
    pub ratio: String,
    pub name: String,
    pub source: String,
    pub contact_email_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_zone_bundle_view(item: &ZoneBundle) -> ZoneBundleView {
    ZoneBundleView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        checksum: item.checksum.clone(),
        window_days_text: item.window_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        name: text::truncate(&item.name, 40),
        source: text::truncate(&item.source, 80),
        contact_email_text: item.contact_email.clone(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_bundle_views(items: &[ZoneBundle]) -> Vec<ZoneBundleView> {
    let mut views: Vec<ZoneBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_bundle_view(item));
    }
    views
}

pub fn sum_zone_bundle_view(view: &ZoneBundleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_bundle("code-582", "label-183");
        let view = to_zone_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
