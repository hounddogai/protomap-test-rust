use super::*;

use acme_kit::text;

/// How screens and exports show a isolation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScheduleView {
    pub code_text: String,
    pub name_text: String,
    pub ends_at: String,
    pub retries: String,
    pub grace_days_text: String,
    pub rate: String,
    pub amount: String,
    pub title: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_isolation_schedule_view(item: &IsolationSchedule) -> IsolationScheduleView {
    IsolationScheduleView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        ends_at: item.ends_at.to_string(),
        retries: item.retries.to_string(),
        grace_days_text: item.grace_days.to_string(),
        rate: format!("{:.2}", item.rate),
        amount: format!("{:.2}", item.amount),
        title: item.title.clone(),
        principal_text: text::truncate(&item.principal, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_schedule_views(items: &[IsolationSchedule]) -> Vec<IsolationScheduleView> {
    let mut views: Vec<IsolationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_schedule_view(item));
    }
    views
}

pub fn sum_isolation_schedule_view(view: &IsolationScheduleView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_retries as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_schedule("code-93", "name-427");
        let view = to_isolation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
