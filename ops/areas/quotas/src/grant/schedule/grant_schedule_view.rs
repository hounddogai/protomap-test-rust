use super::*;

use acme_kit::text;

/// How screens and exports show a grant schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantScheduleView {
    pub code: String,
    pub name_text: String,
    pub description: String,
    pub issuer_text: String,
    pub port: String,
    pub key_id: String,
    pub margin: String,
    pub category: String,
    pub label_text: String,
    pub status_label: String,
}

pub fn to_grant_schedule_view(item: &GrantSchedule) -> GrantScheduleView {
    GrantScheduleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        description: text::truncate(&item.description, 40),
        issuer_text: item.issuer.clone(),
        port: item.port.to_string(),
        key_id: item.key_id.clone(),
        margin: format!("{:.2}", item.margin),
        category: text::truncate(&item.category, 80),
        label_text: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_schedule_views(items: &[GrantSchedule]) -> Vec<GrantScheduleView> {
    let mut views: Vec<GrantScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_schedule_view(item));
    }
    views
}

pub fn sum_grant_schedule_view(view: &GrantScheduleView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_port as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_schedule("code-329", "name-648");
        let view = to_grant_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
