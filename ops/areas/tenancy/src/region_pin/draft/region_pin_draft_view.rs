use super::*;

/// How screens and exports show a region pin draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDraftView {
    pub code: String,
    pub name_text: String,
    pub attempts_text: String,
    pub checksum_text: String,
    pub weight: String,
    pub window_days_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_region_pin_draft_view(item: &RegionPinDraft) -> RegionPinDraftView {
    RegionPinDraftView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        attempts_text: item.attempts.to_string(),
        checksum_text: item.checksum.clone(),
        weight: format!("{:.2}", item.weight),
        window_days_text: item.window_days.to_string(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_draft_views(items: &[RegionPinDraft]) -> Vec<RegionPinDraftView> {
    let mut views: Vec<RegionPinDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_draft_view(item));
    }
    views
}

pub fn sum_region_pin_draft_view(view: &RegionPinDraftView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_weight + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_draft("code-612", "name-780");
        let view = to_region_pin_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
