use super::*;

/// How screens and exports show a region pin target.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTargetView {
    pub code_text: String,
    pub title: String,
    pub window_days: String,
    pub source_text: String,
    pub ratio: String,
    pub rate_text: String,
    pub approved_text: String,
    pub margin: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_region_pin_target_view(item: &RegionPinTarget) -> RegionPinTargetView {
    RegionPinTargetView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        source_text: item.source.clone(),
        ratio: format!("{:.2}", item.ratio),
        rate_text: format!("{:.2}", item.rate),
        approved_text: format!("{}", item.approved),
        margin: format!("{:.2}", item.margin),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_target_views(items: &[RegionPinTarget]) -> Vec<RegionPinTargetView> {
    let mut views: Vec<RegionPinTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_target_view(item));
    }
    views
}

pub fn sum_region_pin_target_view(view: &RegionPinTargetView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_target("code-680", "title-629");
        let view = to_region_pin_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
