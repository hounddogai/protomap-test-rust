use super::*;

use acme_kit::text;

/// How screens and exports show a lenient tenant quota.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuotaView {
    pub code: String,
    pub name: String,
    pub rotation_days: String,
    pub locale: String,
    pub host: String,
    pub pinned_text: String,
    pub duration_minutes: String,
    pub country: String,
    pub cipher: String,
    pub amount: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_lenient_tenant_quota_view(item: &LenientTenantQuota) -> LenientTenantQuotaView {
    LenientTenantQuotaView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        rotation_days: item.rotation_days.to_string(),
        locale: item.locale.clone(),
        host: item.host.clone(),
        pinned_text: format!("{}", item.pinned),
        duration_minutes: item.duration_minutes.to_string(),
        country: text::truncate(&item.country, 64),
        cipher: item.cipher.clone(),
        amount: format!("{:.2}", item.amount),
        title: text::truncate(&item.title, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_tenant_quota_views(items: &[LenientTenantQuota]) -> Vec<LenientTenantQuotaView> {
    let mut views: Vec<LenientTenantQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_tenant_quota_view(item));
    }
    views
}

pub fn sum_lenient_tenant_quota_view(view: &LenientTenantQuotaView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_duration_minutes as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_tenant_quota("code-672", "name-742");
        let view = to_lenient_tenant_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_tenant_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
