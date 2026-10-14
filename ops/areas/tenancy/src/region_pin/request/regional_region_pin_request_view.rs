use super::*;

/// How screens and exports show a regional region pin request.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRegionPinRequestView {
    pub code: String,
    pub title_text: String,
    pub revision_text: String,
    pub burst_text: String,
    pub active: String,
    pub rotation_days: String,
    pub name: String,
    pub version_text: String,
    pub limit: String,
    pub quantity_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_regional_region_pin_request_view(item: &RegionalRegionPinRequest) -> RegionalRegionPinRequestView {
    RegionalRegionPinRequestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        revision_text: item.revision.to_string(),
        burst_text: item.burst.to_string(),
        active: format!("{}", item.active),
        rotation_days: item.rotation_days.to_string(),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        limit: item.limit.to_string(),
        quantity_text: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_region_pin_request_views(items: &[RegionalRegionPinRequest]) -> Vec<RegionalRegionPinRequestView> {
    let mut views: Vec<RegionalRegionPinRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_region_pin_request_view(item));
    }
    views
}

pub fn sum_regional_region_pin_request_view(view: &RegionalRegionPinRequestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_burst as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_region_pin_request("code-986", "title-576");
        let view = to_regional_region_pin_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_region_pin_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
