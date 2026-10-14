use super::*;

use acme_kit::text;

/// How screens and exports show a lenient suspension adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAdjustmentView {
    pub code: String,
    pub title_text: String,
    pub segment: String,
    pub ends_at: String,
    pub visible: String,
    pub source: String,
    pub minimum: String,
    pub quota_units: String,
    pub ratio: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_lenient_suspension_adjustment_view(item: &LenientSuspensionAdjustment) -> LenientSuspensionAdjustmentView {
    LenientSuspensionAdjustmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        segment: item.segment.clone(),
        ends_at: item.ends_at.to_string(),
        visible: format!("{}", item.visible),
        source: text::truncate(&item.source, 80),
        minimum: format!("{:.2}", item.minimum),
        quota_units: item.quota_units.to_string(),
        ratio: format!("{:.2}", item.ratio),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_suspension_adjustment_views(
    items: &[LenientSuspensionAdjustment],
) -> Vec<LenientSuspensionAdjustmentView> {
    let mut views: Vec<LenientSuspensionAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_suspension_adjustment_view(item));
    }
    views
}

pub fn sum_lenient_suspension_adjustment_view(view: &LenientSuspensionAdjustmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_minimum + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_suspension_adjustment("code-389", "title-517");
        let view = to_lenient_suspension_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_suspension_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
