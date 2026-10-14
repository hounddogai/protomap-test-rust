use super::*;

use acme_kit::text;

/// How screens and exports show a manual sink target.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkTargetView {
    pub code: String,
    pub name: String,
    pub duration_minutes_text: String,
    pub updated_at: String,
    pub rate: String,
    pub owner_id_text: String,
    pub segment: String,
    pub margin: String,
    pub unit_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_manual_sink_target_view(item: &ManualSinkTarget) -> ManualSinkTargetView {
    ManualSinkTargetView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        updated_at: item.updated_at.to_string(),
        rate: format!("{:.2}", item.rate),
        owner_id_text: item.owner_id.clone(),
        segment: item.segment.clone(),
        margin: format!("{:.2}", item.margin),
        unit_text: item.unit.clone(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_sink_target_views(items: &[ManualSinkTarget]) -> Vec<ManualSinkTargetView> {
    let mut views: Vec<ManualSinkTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_sink_target_view(item));
    }
    views
}

pub fn sum_manual_sink_target_view(view: &ManualSinkTargetView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_updated_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_sink_target("code-251", "name-426");
        let view = to_manual_sink_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_sink_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
