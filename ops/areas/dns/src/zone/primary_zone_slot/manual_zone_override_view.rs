use super::*;

/// How screens and exports show a manual zone override.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualZoneOverrideView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub burst: String,
    pub port_text: String,
    pub description: String,
    pub archived_text: String,
    pub floor_value_text: String,
    pub region: String,
    pub grace_days: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_manual_zone_override_view(item: &ManualZoneOverride) -> ManualZoneOverrideView {
    ManualZoneOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        burst: item.burst.to_string(),
        port_text: item.port.to_string(),
        description: item.description.clone(),
        archived_text: format!("{}", item.archived),
        floor_value_text: format!("{:.2}", item.floor_value),
        region: item.region.clone(),
        grace_days: item.grace_days.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_zone_override_views(items: &[ManualZoneOverride]) -> Vec<ManualZoneOverrideView> {
    let mut views: Vec<ManualZoneOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_zone_override_view(item));
    }
    views
}

pub fn sum_manual_zone_override_view(view: &ManualZoneOverrideView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_minimum + (parsed_burst as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_zone_override("code-649", "name-7");
        let view = to_manual_zone_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_zone_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
