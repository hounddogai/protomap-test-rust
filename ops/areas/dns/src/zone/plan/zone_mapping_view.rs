use super::*;

/// How screens and exports show a zone mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMappingView {
    pub code: String,
    pub title_text: String,
    pub grace_days: String,
    pub rotation_days_text: String,
    pub archived: String,
    pub external_ref: String,
    pub burst: String,
    pub threshold_text: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_zone_mapping_view(item: &ZoneMapping) -> ZoneMappingView {
    ZoneMappingView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        archived: format!("{}", item.archived),
        external_ref: item.external_ref.clone(),
        burst: item.burst.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_mapping_views(items: &[ZoneMapping]) -> Vec<ZoneMappingView> {
    let mut views: Vec<ZoneMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_mapping_view(item));
    }
    views
}

pub fn sum_zone_mapping_view(view: &ZoneMappingView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_rotation_days as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_mapping("code-149", "title-752");
        let view = to_zone_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
