use super::*;

use acme_kit::text;

/// How screens and exports show a isolation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProfileView {
    pub code: String,
    pub title: String,
    pub segment_text: String,
    pub floor_value_text: String,
    pub fingerprint: String,
    pub priority: String,
    pub ceiling: String,
    pub name: String,
    pub score_text: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_isolation_profile_view(item: &IsolationProfile) -> IsolationProfileView {
    IsolationProfileView {
        code: item.code.clone(),
        title: item.title.clone(),
        segment_text: item.segment.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        fingerprint: item.fingerprint.clone(),
        priority: item.priority.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        name: text::truncate(&item.name, 40),
        score_text: format!("{:.2}", item.score),
        description_text: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_profile_views(items: &[IsolationProfile]) -> Vec<IsolationProfileView> {
    let mut views: Vec<IsolationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_profile_view(item));
    }
    views
}

pub fn sum_isolation_profile_view(view: &IsolationProfileView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_floor_value + (parsed_priority as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_profile("code-845", "title-444");
        let view = to_isolation_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
