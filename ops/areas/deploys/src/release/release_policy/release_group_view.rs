use super::*;

use acme_kit::text;

/// How screens and exports show a release group.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupView {
    pub code: String,
    pub title_text: String,
    pub budget: String,
    pub reviewer: String,
    pub unit_text: String,
    pub archived: String,
    pub port: String,
    pub ends_at_text: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_release_group_view(item: &ReleaseGroup) -> ReleaseGroupView {
    ReleaseGroupView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        budget: format!("{:.2}", item.budget),
        reviewer: item.reviewer.clone(),
        unit_text: item.unit.clone(),
        archived: format!("{}", item.archived),
        port: item.port.to_string(),
        ends_at_text: item.ends_at.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_group_views(items: &[ReleaseGroup]) -> Vec<ReleaseGroupView> {
    let mut views: Vec<ReleaseGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_group_view(item));
    }
    views
}

pub fn sum_release_group_view(view: &ReleaseGroupView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_budget + (parsed_port as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_group("code-241", "title-210");
        let view = to_release_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
