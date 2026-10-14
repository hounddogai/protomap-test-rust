use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard exception.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionView {
    pub code: String,
    pub name: String,
    pub ratio: String,
    pub archived: String,
    pub starts_at_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_dashboard_exception_view(item: &DashboardException) -> DashboardExceptionView {
    DashboardExceptionView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        ratio: format!("{:.2}", item.ratio),
        archived: format!("{}", item.archived),
        starts_at_text: item.starts_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_exception_views(items: &[DashboardException]) -> Vec<DashboardExceptionView> {
    let mut views: Vec<DashboardExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_exception_view(item));
    }
    views
}

pub fn sum_dashboard_exception_view(view: &DashboardExceptionView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ratio + (parsed_starts_at as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_exception("code-715", "name-14");
        let view = to_dashboard_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
