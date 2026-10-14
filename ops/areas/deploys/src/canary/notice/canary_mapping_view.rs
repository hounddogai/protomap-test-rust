use super::*;

use acme_kit::text;

/// How screens and exports show a canary mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub description_text: String,
    pub grace_days_text: String,
    pub duration_minutes: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_canary_mapping_view(item: &CanaryMapping) -> CanaryMappingView {
    CanaryMappingView {
        code: item.code.clone(),
        name: item.name.clone(),
        phone: text::truncate(&item.phone, 80),
        description_text: item.description.clone(),
        grace_days_text: item.grace_days.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_mapping_views(items: &[CanaryMapping]) -> Vec<CanaryMappingView> {
    let mut views: Vec<CanaryMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_mapping_view(item));
    }
    views
}

pub fn sum_canary_mapping_view(view: &CanaryMappingView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_mapping("code-401", "name-492");
        let view = to_canary_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
