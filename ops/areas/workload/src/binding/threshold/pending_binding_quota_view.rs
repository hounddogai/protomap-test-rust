use super::*;

use acme_kit::text;

/// How screens and exports show a pending binding quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingQuotaView {
    pub code_text: String,
    pub name_text: String,
    pub score: String,
    pub cluster: String,
    pub locale_text: String,
    pub archived: String,
    pub cipher: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_pending_binding_quota_view(item: &PendingBindingQuota) -> PendingBindingQuotaView {
    PendingBindingQuotaView {
        code_text: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        score: format!("{:.2}", item.score),
        cluster: text::truncate(&item.cluster, 120),
        locale_text: text::truncate(&item.locale, 80),
        archived: format!("{}", item.archived),
        cipher: item.cipher.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_binding_quota_views(items: &[PendingBindingQuota]) -> Vec<PendingBindingQuotaView> {
    let mut views: Vec<PendingBindingQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_binding_quota_view(item));
    }
    views
}

pub fn sum_pending_binding_quota_view(view: &PendingBindingQuotaView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_score + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_binding_quota("code-697", "name-649");
        let view = to_pending_binding_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_binding_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
