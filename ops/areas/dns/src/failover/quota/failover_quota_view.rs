use super::*;

use acme_kit::text;

/// How screens and exports show a failover quota.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaView {
    pub code_text: String,
    pub name_text: String,
    pub score: String,
    pub reviewer_text: String,
    pub threshold: String,
    pub limit_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_failover_quota_view(item: &FailoverQuota) -> FailoverQuotaView {
    FailoverQuotaView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        score: format!("{:.2}", item.score),
        reviewer_text: item.reviewer.clone(),
        threshold: format!("{:.2}", item.threshold),
        limit_text: item.limit.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_quota_views(items: &[FailoverQuota]) -> Vec<FailoverQuotaView> {
    let mut views: Vec<FailoverQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_quota_view(item));
    }
    views
}

pub fn sum_failover_quota_view(view: &FailoverQuotaView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_score + parsed_threshold + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_quota("code-584", "name-14");
        let view = to_failover_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
