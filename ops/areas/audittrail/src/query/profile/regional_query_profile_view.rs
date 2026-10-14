use super::*;

use acme_kit::text;

/// How screens and exports show a regional query profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryProfileView {
    pub code_text: String,
    pub name: String,
    pub category: String,
    pub source: String,
    pub port: String,
    pub notes_text: String,
    pub floor_value: String,
    pub ttl_seconds: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_regional_query_profile_view(item: &RegionalQueryProfile) -> RegionalQueryProfileView {
    RegionalQueryProfileView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        category: text::truncate(&item.category, 64),
        source: text::truncate(&item.source, 64),
        port: item.port.to_string(),
        notes_text: text::truncate(&item.notes, 40),
        floor_value: format!("{:.2}", item.floor_value),
        ttl_seconds: item.ttl_seconds.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_query_profile_views(items: &[RegionalQueryProfile]) -> Vec<RegionalQueryProfileView> {
    let mut views: Vec<RegionalQueryProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_query_profile_view(item));
    }
    views
}

pub fn sum_regional_query_profile_view(view: &RegionalQueryProfileView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_port as f64 + parsed_floor_value + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_query_profile("code-36", "name-276");
        let view = to_regional_query_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_query_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
