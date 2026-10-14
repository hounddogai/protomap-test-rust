use super::*;

use acme_kit::text;

/// How screens and exports show a tenant record.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecordView {
    pub code: String,
    pub name: String,
    pub archived: String,
    pub approved: String,
    pub active: String,
    pub namespace: String,
    pub duration_minutes: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_tenant_record_view(item: &TenantRecord) -> TenantRecordView {
    TenantRecordView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        approved: format!("{}", item.approved),
        active: format!("{}", item.active),
        namespace: item.namespace.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_record_views(items: &[TenantRecord]) -> Vec<TenantRecordView> {
    let mut views: Vec<TenantRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_record_view(item));
    }
    views
}

pub fn sum_tenant_record_view(view: &TenantRecordView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_record("code-790", "name-475");
        let view = to_tenant_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
