use super::*;

use acme_kit::text;

/// How screens and exports show a tiered dns record threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub rotation_days_text: String,
    pub budget: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_tiered_dns_record_threshold_view(item: &TieredDnsRecordThreshold) -> TieredDnsRecordThresholdView {
    TieredDnsRecordThresholdView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        budget: format!("{:.2}", item.budget),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_dns_record_threshold_views(items: &[TieredDnsRecordThreshold]) -> Vec<TieredDnsRecordThresholdView> {
    let mut views: Vec<TieredDnsRecordThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_dns_record_threshold_view(item));
    }
    views
}

pub fn sum_tiered_dns_record_threshold_view(view: &TieredDnsRecordThresholdView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_dns_record_threshold("code-872", "name-804");
        let view = to_tiered_dns_record_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_dns_record_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
