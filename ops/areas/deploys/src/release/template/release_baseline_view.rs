use super::*;

use acme_kit::text;

/// How screens and exports show a release baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaselineView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub notes_text: String,
    pub tenant_id: String,
    pub floor_value: String,
    pub discount_text: String,
    pub window_days: String,
    pub audience_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_release_baseline_view(item: &ReleaseBaseline) -> ReleaseBaselineView {
    ReleaseBaselineView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        notes_text: item.notes.clone(),
        tenant_id: item.tenant_id.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        discount_text: format!("{:.2}", item.discount),
        window_days: item.window_days.to_string(),
        audience_text: item.audience.clone(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_baseline_views(items: &[ReleaseBaseline]) -> Vec<ReleaseBaselineView> {
    let mut views: Vec<ReleaseBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_baseline_view(item));
    }
    views
}

pub fn sum_release_baseline_view(view: &ReleaseBaselineView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_floor_value + parsed_discount + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_baseline("code-857", "label-91");
        let view = to_release_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
