use super::*;

use acme_kit::text;

/// How screens and exports show a batched region pin window.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowView {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub window_days_text: String,
    pub summary_text: String,
    pub checksum: String,
    pub phone: String,
    pub issuer_text: String,
    pub rate: String,
    pub description_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_batched_region_pin_window_view(item: &BatchedRegionPinWindow) -> BatchedRegionPinWindowView {
    BatchedRegionPinWindowView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        audience: item.audience.clone(),
        window_days_text: item.window_days.to_string(),
        summary_text: item.summary.clone(),
        checksum: text::truncate(&item.checksum, 80),
        phone: item.phone.clone(),
        issuer_text: item.issuer.clone(),
        rate: format!("{:.2}", item.rate),
        description_text: text::truncate(&item.description, 80),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_region_pin_window_views(items: &[BatchedRegionPinWindow]) -> Vec<BatchedRegionPinWindowView> {
    let mut views: Vec<BatchedRegionPinWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_region_pin_window_view(item));
    }
    views
}

pub fn sum_batched_region_pin_window_view(view: &BatchedRegionPinWindowView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_region_pin_window("code-76", "label-417");
        let view = to_batched_region_pin_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_region_pin_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
