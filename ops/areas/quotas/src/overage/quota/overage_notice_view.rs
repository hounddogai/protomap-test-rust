use super::*;

use acme_kit::text;

/// How screens and exports show a overage notice.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageNoticeView {
    pub code: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub region: String,
    pub quantity_text: String,
    pub approved: String,
    pub checksum_text: String,
    pub segment: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_overage_notice_view(item: &OverageNotice) -> OverageNoticeView {
    OverageNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        version: item.version.to_string(),
        description: item.description.clone(),
        region: item.region.clone(),
        quantity_text: item.quantity.to_string(),
        approved: format!("{}", item.approved),
        checksum_text: item.checksum.clone(),
        segment: text::truncate(&item.segment, 80),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_notice_views(items: &[OverageNotice]) -> Vec<OverageNoticeView> {
    let mut views: Vec<OverageNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_notice_view(item));
    }
    views
}

pub fn sum_overage_notice_view(view: &OverageNoticeView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_version as f64 + (parsed_quantity as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_notice("code-130", "title-707");
        let view = to_overage_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
