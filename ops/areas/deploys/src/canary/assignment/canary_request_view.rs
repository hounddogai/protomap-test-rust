use super::*;

use acme_kit::text;

/// How screens and exports show a canary request.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRequestView {
    pub code: String,
    pub name_text: String,
    pub reviewer: String,
    pub currency: String,
    pub title: String,
    pub scope: String,
    pub rotation_days: String,
    pub rate: String,
    pub cost: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_canary_request_view(item: &CanaryRequest) -> CanaryRequestView {
    CanaryRequestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        reviewer: text::truncate(&item.reviewer, 64),
        currency: item.currency.clone(),
        title: item.title.clone(),
        scope: text::truncate(&item.scope, 120),
        rotation_days: item.rotation_days.to_string(),
        rate: format!("{:.2}", item.rate),
        cost: format!("{:.2}", item.cost),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_request_views(items: &[CanaryRequest]) -> Vec<CanaryRequestView> {
    let mut views: Vec<CanaryRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_request_view(item));
    }
    views
}

pub fn sum_canary_request_view(view: &CanaryRequestView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_rate + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_request("code-454", "name-646");
        let view = to_canary_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
