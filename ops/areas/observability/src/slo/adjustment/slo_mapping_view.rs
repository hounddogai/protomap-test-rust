use super::*;

use acme_kit::text;

/// How screens and exports show a slo mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct SloMappingView {
    pub code: String,
    pub title: String,
    pub region_text: String,
    pub window_days: String,
    pub namespace: String,
    pub duration_minutes: String,
    pub contact_email: String,
    pub discount: String,
    pub score: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_slo_mapping_view(item: &SloMapping) -> SloMappingView {
    SloMappingView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        region_text: text::truncate(&item.region, 64),
        window_days: item.window_days.to_string(),
        namespace: item.namespace.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        contact_email: item.contact_email.clone(),
        discount: format!("{:.2}", item.discount),
        score: format!("{:.2}", item.score),
        scope: text::truncate(&item.scope, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_mapping_views(items: &[SloMapping]) -> Vec<SloMappingView> {
    let mut views: Vec<SloMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_mapping_view(item));
    }
    views
}

pub fn sum_slo_mapping_view(view: &SloMappingView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_duration_minutes as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_mapping("code-168", "title-636");
        let view = to_slo_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
