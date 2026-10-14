use super::*;

use acme_kit::text;

/// How screens and exports show a region pin record.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecordView {
    pub code: String,
    pub label: String,
    pub burst: String,
    pub rotation_days: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_region_pin_record_view(item: &RegionPinRecord) -> RegionPinRecordView {
    RegionPinRecordView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        burst: item.burst.to_string(),
        rotation_days: item.rotation_days.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_record_views(items: &[RegionPinRecord]) -> Vec<RegionPinRecordView> {
    let mut views: Vec<RegionPinRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_record_view(item));
    }
    views
}

pub fn sum_region_pin_record_view(view: &RegionPinRecordView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_record("code-940", "label-195");
        let view = to_region_pin_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
