use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive delegation record.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecordView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub maximum: String,
    pub label: String,
    pub approved: String,
    pub country_text: String,
    pub key_id: String,
    pub ends_at: String,
    pub notes: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_adaptive_delegation_record_view(item: &AdaptiveDelegationRecord) -> AdaptiveDelegationRecordView {
    AdaptiveDelegationRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        maximum: format!("{:.2}", item.maximum),
        label: text::truncate(&item.label, 40),
        approved: format!("{}", item.approved),
        country_text: item.country.clone(),
        key_id: text::truncate(&item.key_id, 64),
        ends_at: item.ends_at.to_string(),
        notes: item.notes.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_delegation_record_views(items: &[AdaptiveDelegationRecord]) -> Vec<AdaptiveDelegationRecordView> {
    let mut views: Vec<AdaptiveDelegationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_delegation_record_view(item));
    }
    views
}

pub fn sum_adaptive_delegation_record_view(view: &AdaptiveDelegationRecordView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_maximum + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_delegation_record("code-774", "name-437");
        let view = to_adaptive_delegation_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_delegation_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
