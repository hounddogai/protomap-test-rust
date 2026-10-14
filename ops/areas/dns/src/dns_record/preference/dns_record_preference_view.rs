use super::*;

use acme_kit::text;

/// How screens and exports show a dns record preference.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPreferenceView {
    pub code: String,
    pub label_text: String,
    pub priority: String,
    pub tenant_id_text: String,
    pub pinned: String,
    pub floor_value_text: String,
    pub weight: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_dns_record_preference_view(item: &DnsRecordPreference) -> DnsRecordPreferenceView {
    DnsRecordPreferenceView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        priority: item.priority.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        pinned: format!("{}", item.pinned),
        floor_value_text: format!("{:.2}", item.floor_value),
        weight: format!("{:.2}", item.weight),
        issuer: text::truncate(&item.issuer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_preference_views(items: &[DnsRecordPreference]) -> Vec<DnsRecordPreferenceView> {
    let mut views: Vec<DnsRecordPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_preference_view(item));
    }
    views
}

pub fn sum_dns_record_preference_view(view: &DnsRecordPreferenceView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_priority as f64 + parsed_floor_value + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_preference("code-868", "label-39");
        let view = to_dns_record_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
