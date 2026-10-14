use super::*;

use acme_kit::text;

/// How screens and exports show a slo profile.
#[derive(Debug, Clone, PartialEq)]
pub struct SloProfileView {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub minimum: String,
    pub active: String,
    pub margin: String,
    pub reviewer: String,
    pub duration_minutes: String,
    pub discount: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_slo_profile_view(item: &SloProfile) -> SloProfileView {
    SloProfileView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        segment: item.segment.clone(),
        minimum: format!("{:.2}", item.minimum),
        active: format!("{}", item.active),
        margin: format!("{:.2}", item.margin),
        reviewer: item.reviewer.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        discount: format!("{:.2}", item.discount),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_profile_views(items: &[SloProfile]) -> Vec<SloProfileView> {
    let mut views: Vec<SloProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_profile_view(item));
    }
    views
}

pub fn sum_slo_profile_view(view: &SloProfileView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_minimum + parsed_margin + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_profile("code-164", "label-379");
        let view = to_slo_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
