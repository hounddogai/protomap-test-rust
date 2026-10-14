use super::*;

/// How screens and exports show a delegation quota.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationQuotaView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub created_at: String,
    pub external_ref_text: String,
    pub notes: String,
    pub ttl_seconds: String,
    pub burst: String,
    pub expires_at: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_delegation_quota_view(item: &DelegationQuota) -> DelegationQuotaView {
    DelegationQuotaView {
        code: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        created_at: item.created_at.to_string(),
        external_ref_text: item.external_ref.clone(),
        notes: item.notes.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        burst: item.burst.to_string(),
        expires_at: item.expires_at.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_quota_views(items: &[DelegationQuota]) -> Vec<DelegationQuotaView> {
    let mut views: Vec<DelegationQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_quota_view(item));
    }
    views
}

pub fn sum_delegation_quota_view(view: &DelegationQuotaView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_score + (parsed_created_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_quota("code-331", "title-659");
        let view = to_delegation_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
