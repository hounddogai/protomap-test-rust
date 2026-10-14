use super::*;

use acme_kit::text;

/// How screens and exports show a manual tenant notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub archived_text: String,
    pub batch_size_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_manual_tenant_notice_view(item: &ManualTenantNotice) -> ManualTenantNoticeView {
    ManualTenantNoticeView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 64),
        quantity: item.quantity.to_string(),
        archived_text: format!("{}", item.archived),
        batch_size_text: item.batch_size.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_tenant_notice_views(items: &[ManualTenantNotice]) -> Vec<ManualTenantNoticeView> {
    let mut views: Vec<ManualTenantNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_tenant_notice_view(item));
    }
    views
}

pub fn sum_manual_tenant_notice_view(view: &ManualTenantNoticeView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_tenant_notice("code-899", "name-170");
        let view = to_manual_tenant_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_tenant_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
