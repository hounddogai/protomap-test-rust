use super::*;

use acme_kit::text;

/// How screens and exports show a overage request.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequestView {
    pub code_text: String,
    pub label_text: String,
    pub retries_text: String,
    pub channel_text: String,
    pub currency: String,
    pub sequence: String,
    pub grace_days: String,
    pub window_days_text: String,
    pub limit: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_overage_request_view(item: &OverageRequest) -> OverageRequestView {
    OverageRequestView {
        code_text: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        retries_text: item.retries.to_string(),
        channel_text: text::truncate(&item.channel, 80),
        currency: text::truncate(&item.currency, 64),
        sequence: item.sequence.to_string(),
        grace_days: item.grace_days.to_string(),
        window_days_text: item.window_days.to_string(),
        limit: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_request_views(items: &[OverageRequest]) -> Vec<OverageRequestView> {
    let mut views: Vec<OverageRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_request_view(item));
    }
    views
}

pub fn sum_overage_request_view(view: &OverageRequestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_sequence as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_request("code-976", "label-938");
        let view = to_overage_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
