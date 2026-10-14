use super::*;

use acme_kit::text;

/// How screens and exports show a capped tenant quota.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaView {
    pub code_text: String,
    pub name: String,
    pub summary: String,
    pub rate: String,
    pub ceiling_text: String,
    pub scope_text: String,
    pub notes: String,
    pub attempts: String,
    pub batch_size: String,
    pub issuer_text: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_capped_tenant_quota_view(item: &CappedTenantQuota) -> CappedTenantQuotaView {
    CappedTenantQuotaView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        summary: text::truncate(&item.summary, 40),
        rate: format!("{:.2}", item.rate),
        ceiling_text: format!("{:.2}", item.ceiling),
        scope_text: item.scope.clone(),
        notes: item.notes.clone(),
        attempts: item.attempts.to_string(),
        batch_size: item.batch_size.to_string(),
        issuer_text: text::truncate(&item.issuer, 40),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_tenant_quota_views(items: &[CappedTenantQuota]) -> Vec<CappedTenantQuotaView> {
    let mut views: Vec<CappedTenantQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_tenant_quota_view(item));
    }
    views
}

pub fn sum_capped_tenant_quota_view(view: &CappedTenantQuotaView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_rate + parsed_ceiling + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_tenant_quota("code-153", "name-780");
        let view = to_capped_tenant_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_tenant_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
