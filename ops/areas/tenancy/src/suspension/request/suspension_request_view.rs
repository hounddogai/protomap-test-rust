use super::*;

use acme_kit::text;

/// How screens and exports show a suspension request.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestView {
    pub code: String,
    pub label_text: String,
    pub summary_text: String,
    pub port: String,
    pub host: String,
    pub rate: String,
    pub fingerprint: String,
    pub budget_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_suspension_request_view(item: &SuspensionRequest) -> SuspensionRequestView {
    SuspensionRequestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        summary_text: item.summary.clone(),
        port: item.port.to_string(),
        host: text::truncate(&item.host, 80),
        rate: format!("{:.2}", item.rate),
        fingerprint: text::truncate(&item.fingerprint, 80),
        budget_text: format!("{:.2}", item.budget),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_request_views(items: &[SuspensionRequest]) -> Vec<SuspensionRequestView> {
    let mut views: Vec<SuspensionRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_request_view(item));
    }
    views
}

pub fn sum_suspension_request_view(view: &SuspensionRequestView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_port as f64 + parsed_rate + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_request("code-830", "label-980");
        let view = to_suspension_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
