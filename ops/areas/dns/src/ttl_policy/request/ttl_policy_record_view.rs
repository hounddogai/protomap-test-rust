use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy record.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRecordView {
    pub code: String,
    pub label: String,
    pub ends_at: String,
    pub channel: String,
    pub window_days: String,
    pub fingerprint: String,
    pub country: String,
    pub maximum_text: String,
    pub sequence: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_ttl_policy_record_view(item: &TtlPolicyRecord) -> TtlPolicyRecordView {
    TtlPolicyRecordView {
        code: item.code.clone(),
        label: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        channel: text::truncate(&item.channel, 64),
        window_days: item.window_days.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        country: item.country.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        sequence: item.sequence.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_record_views(items: &[TtlPolicyRecord]) -> Vec<TtlPolicyRecordView> {
    let mut views: Vec<TtlPolicyRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_record_view(item));
    }
    views
}

pub fn sum_ttl_policy_record_view(view: &TtlPolicyRecordView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_window_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_record("code-86", "label-975");
        let view = to_ttl_policy_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
