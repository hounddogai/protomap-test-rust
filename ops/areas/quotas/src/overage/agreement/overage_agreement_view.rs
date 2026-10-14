use super::*;

use acme_kit::text;

/// How screens and exports show a overage agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAgreementView {
    pub code_text: String,
    pub title: String,
    pub tenant_id: String,
    pub port_text: String,
    pub ratio_text: String,
    pub visible: String,
    pub attempts: String,
    pub created_at: String,
    pub batch_size: String,
    pub locale: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_overage_agreement_view(item: &OverageAgreement) -> OverageAgreementView {
    OverageAgreementView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 120),
        tenant_id: item.tenant_id.clone(),
        port_text: item.port.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        visible: format!("{}", item.visible),
        attempts: item.attempts.to_string(),
        created_at: item.created_at.to_string(),
        batch_size: item.batch_size.to_string(),
        locale: text::truncate(&item.locale, 120),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_agreement_views(items: &[OverageAgreement]) -> Vec<OverageAgreementView> {
    let mut views: Vec<OverageAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_agreement_view(item));
    }
    views
}

pub fn sum_overage_agreement_view(view: &OverageAgreementView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_port as f64 + parsed_ratio + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_agreement("code-681", "title-35");
        let view = to_overage_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
