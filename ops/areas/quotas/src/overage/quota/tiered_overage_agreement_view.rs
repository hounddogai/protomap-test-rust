use super::*;

use acme_kit::text;

/// How screens and exports show a tiered overage agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAgreementView {
    pub code: String,
    pub name: String,
    pub batch_size: String,
    pub grace_days_text: String,
    pub reviewer: String,
    pub external_ref: String,
    pub audience: String,
    pub cluster: String,
    pub port_text: String,
    pub ends_at: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_tiered_overage_agreement_view(item: &TieredOverageAgreement) -> TieredOverageAgreementView {
    TieredOverageAgreementView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        grace_days_text: item.grace_days.to_string(),
        reviewer: item.reviewer.clone(),
        external_ref: item.external_ref.clone(),
        audience: item.audience.clone(),
        cluster: item.cluster.clone(),
        port_text: item.port.to_string(),
        ends_at: item.ends_at.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_overage_agreement_views(items: &[TieredOverageAgreement]) -> Vec<TieredOverageAgreementView> {
    let mut views: Vec<TieredOverageAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_overage_agreement_view(item));
    }
    views
}

pub fn sum_tiered_overage_agreement_view(view: &TieredOverageAgreementView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_grace_days as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_overage_agreement("code-973", "name-412");
        let view = to_tiered_overage_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_overage_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
