use super::*;

use acme_kit::text;

/// How screens and exports show a dns record policy.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyView {
    pub code: String,
    pub name_text: String,
    pub phone: String,
    pub namespace: String,
    pub score: String,
    pub owner_id: String,
    pub revision_text: String,
    pub weight: String,
    pub created_at_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_dns_record_policy_view(item: &DnsRecordPolicy) -> DnsRecordPolicyView {
    DnsRecordPolicyView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        phone: text::truncate(&item.phone, 40),
        namespace: item.namespace.clone(),
        score: format!("{:.2}", item.score),
        owner_id: item.owner_id.clone(),
        revision_text: item.revision.to_string(),
        weight: format!("{:.2}", item.weight),
        created_at_text: item.created_at.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_policy_views(items: &[DnsRecordPolicy]) -> Vec<DnsRecordPolicyView> {
    let mut views: Vec<DnsRecordPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_policy_view(item));
    }
    views
}

pub fn sum_dns_record_policy_view(view: &DnsRecordPolicyView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_score + (parsed_revision as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_policy("code-803", "name-70");
        let view = to_dns_record_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
