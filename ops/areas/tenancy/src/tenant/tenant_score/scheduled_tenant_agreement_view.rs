use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled tenant agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreementView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub name: String,
    pub archived: String,
    pub score_text: String,
    pub port: String,
    pub locale_text: String,
    pub contact_email_text: String,
    pub principal: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_scheduled_tenant_agreement_view(item: &ScheduledTenantAgreement) -> ScheduledTenantAgreementView {
    ScheduledTenantAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        score_text: format!("{:.2}", item.score),
        port: item.port.to_string(),
        locale_text: text::truncate(&item.locale, 80),
        contact_email_text: text::truncate(&item.contact_email, 80),
        principal: item.principal.clone(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_tenant_agreement_views(items: &[ScheduledTenantAgreement]) -> Vec<ScheduledTenantAgreementView> {
    let mut views: Vec<ScheduledTenantAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_tenant_agreement_view(item));
    }
    views
}

pub fn sum_scheduled_tenant_agreement_view(view: &ScheduledTenantAgreementView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_score + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_tenant_agreement("code-383", "label-116");
        let view = to_scheduled_tenant_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_tenant_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
