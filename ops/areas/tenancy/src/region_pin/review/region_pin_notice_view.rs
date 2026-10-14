use super::*;

use acme_kit::text;

/// How screens and exports show a region pin notice.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinNoticeView {
    pub code: String,
    pub title: String,
    pub region_text: String,
    pub name: String,
    pub attempts_text: String,
    pub updated_at_text: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_region_pin_notice_view(item: &RegionPinNotice) -> RegionPinNoticeView {
    RegionPinNoticeView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        region_text: text::truncate(&item.region, 120),
        name: text::truncate(&item.name, 40),
        attempts_text: item.attempts.to_string(),
        updated_at_text: item.updated_at.to_string(),
        country_text: text::truncate(&item.country, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_notice_views(items: &[RegionPinNotice]) -> Vec<RegionPinNoticeView> {
    let mut views: Vec<RegionPinNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_notice_view(item));
    }
    views
}

pub fn sum_region_pin_notice_view(view: &RegionPinNoticeView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_notice("code-777", "title-660");
        let view = to_region_pin_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
