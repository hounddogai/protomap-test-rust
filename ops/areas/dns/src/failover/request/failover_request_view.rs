use super::*;

use acme_kit::text;

/// How screens and exports show a failover request.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestView {
    pub code: String,
    pub label: String,
    pub active: String,
    pub reviewer_text: String,
    pub retries_text: String,
    pub country: String,
    pub burst: String,
    pub currency: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_failover_request_view(item: &FailoverRequest) -> FailoverRequestView {
    FailoverRequestView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        active: format!("{}", item.active),
        reviewer_text: item.reviewer.clone(),
        retries_text: item.retries.to_string(),
        country: item.country.clone(),
        burst: item.burst.to_string(),
        currency: text::truncate(&item.currency, 80),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_request_views(items: &[FailoverRequest]) -> Vec<FailoverRequestView> {
    let mut views: Vec<FailoverRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_request_view(item));
    }
    views
}

pub fn sum_failover_request_view(view: &FailoverRequestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_burst as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_request("code-66", "label-559");
        let view = to_failover_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
