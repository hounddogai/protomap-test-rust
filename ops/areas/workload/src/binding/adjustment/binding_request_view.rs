use super::*;

/// How screens and exports show a binding request.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestView {
    pub code: String,
    pub title: String,
    pub priority_text: String,
    pub rate_text: String,
    pub limit: String,
    pub threshold: String,
    pub ends_at_text: String,
    pub window_days: String,
    pub weight: String,
    pub batch_size: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_binding_request_view(item: &BindingRequest) -> BindingRequestView {
    BindingRequestView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority_text: item.priority.to_string(),
        rate_text: format!("{:.2}", item.rate),
        limit: item.limit.to_string(),
        threshold: format!("{:.2}", item.threshold),
        ends_at_text: item.ends_at.to_string(),
        window_days: item.window_days.to_string(),
        weight: format!("{:.2}", item.weight),
        batch_size: item.batch_size.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_request_views(items: &[BindingRequest]) -> Vec<BindingRequestView> {
    let mut views: Vec<BindingRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_request_view(item));
    }
    views
}

pub fn sum_binding_request_view(view: &BindingRequestView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_priority as f64 + parsed_rate + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_request("code-452", "title-209");
        let view = to_binding_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
