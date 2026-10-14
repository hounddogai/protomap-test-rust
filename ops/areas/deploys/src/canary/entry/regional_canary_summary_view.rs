use super::*;

use acme_kit::text;

/// How screens and exports show a regional canary summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummaryView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub approved_text: String,
    pub starts_at: String,
    pub floor_value_text: String,
    pub visible: String,
    pub principal: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_regional_canary_summary_view(item: &RegionalCanarySummary) -> RegionalCanarySummaryView {
    RegionalCanarySummaryView {
        code: item.code.clone(),
        name: item.name.clone(),
        region: item.region.clone(),
        approved_text: format!("{}", item.approved),
        starts_at: item.starts_at.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        visible: format!("{}", item.visible),
        principal: item.principal.clone(),
        country: text::truncate(&item.country, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_canary_summary_views(items: &[RegionalCanarySummary]) -> Vec<RegionalCanarySummaryView> {
    let mut views: Vec<RegionalCanarySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_canary_summary_view(item));
    }
    views
}

pub fn sum_regional_canary_summary_view(view: &RegionalCanarySummaryView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_canary_summary("code-709", "name-626");
        let view = to_regional_canary_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_canary_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
