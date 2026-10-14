use super::*;

/// How screens and exports show a seasonal probe override.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverrideView {
    pub code: String,
    pub name: String,
    pub minimum_text: String,
    pub floor_value: String,
    pub margin: String,
    pub checksum: String,
    pub pinned_text: String,
    pub active: String,
    pub discount: String,
    pub visible: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_seasonal_probe_override_view(item: &SeasonalProbeOverride) -> SeasonalProbeOverrideView {
    SeasonalProbeOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        floor_value: format!("{:.2}", item.floor_value),
        margin: format!("{:.2}", item.margin),
        checksum: item.checksum.clone(),
        pinned_text: format!("{}", item.pinned),
        active: format!("{}", item.active),
        discount: format!("{:.2}", item.discount),
        visible: format!("{}", item.visible),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_probe_override_views(items: &[SeasonalProbeOverride]) -> Vec<SeasonalProbeOverrideView> {
    let mut views: Vec<SeasonalProbeOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_probe_override_view(item));
    }
    views
}

pub fn sum_seasonal_probe_override_view(view: &SeasonalProbeOverrideView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_minimum + parsed_floor_value + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_probe_override("code-97", "name-221");
        let view = to_seasonal_probe_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_probe_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
