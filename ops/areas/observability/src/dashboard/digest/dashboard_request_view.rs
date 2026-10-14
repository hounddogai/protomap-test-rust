use super::*;

/// How screens and exports show a dashboard request.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequestView {
    pub code: String,
    pub title: String,
    pub visible: String,
    pub contact_email: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_dashboard_request_view(item: &DashboardRequest) -> DashboardRequestView {
    DashboardRequestView {
        code: item.code.clone(),
        title: item.title.clone(),
        visible: format!("{}", item.visible),
        contact_email: item.contact_email.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_request_views(items: &[DashboardRequest]) -> Vec<DashboardRequestView> {
    let mut views: Vec<DashboardRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_request_view(item));
    }
    views
}

pub fn sum_dashboard_request_view(view: &DashboardRequestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_request("code-472", "title-91");
        let view = to_dashboard_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
