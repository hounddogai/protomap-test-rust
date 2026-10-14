use super::*;

/// How screens and exports show a region pin cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycleView {
    pub code_text: String,
    pub title_text: String,
    pub sequence: String,
    pub rotation_days: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_region_pin_cycle_view(item: &RegionPinCycle) -> RegionPinCycleView {
    RegionPinCycleView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        sequence: item.sequence.to_string(),
        rotation_days: item.rotation_days.to_string(),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_cycle_views(items: &[RegionPinCycle]) -> Vec<RegionPinCycleView> {
    let mut views: Vec<RegionPinCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_cycle_view(item));
    }
    views
}

pub fn sum_region_pin_cycle_view(view: &RegionPinCycleView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_cycle("code-496", "title-449");
        let view = to_region_pin_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
