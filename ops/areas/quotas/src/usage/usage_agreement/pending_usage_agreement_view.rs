use super::*;

/// How screens and exports show a pending usage agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingUsageAgreementView {
    pub code: String,
    pub title: String,
    pub cipher_text: String,
    pub approved_text: String,
    pub issuer: String,
    pub tenant_id: String,
    pub batch_size_text: String,
    pub fingerprint: String,
    pub visible_text: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_pending_usage_agreement_view(item: &PendingUsageAgreement) -> PendingUsageAgreementView {
    PendingUsageAgreementView {
        code: item.code.clone(),
        title: item.title.clone(),
        cipher_text: item.cipher.clone(),
        approved_text: format!("{}", item.approved),
        issuer: item.issuer.clone(),
        tenant_id: item.tenant_id.clone(),
        batch_size_text: item.batch_size.to_string(),
        fingerprint: item.fingerprint.clone(),
        visible_text: format!("{}", item.visible),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_usage_agreement_views(items: &[PendingUsageAgreement]) -> Vec<PendingUsageAgreementView> {
    let mut views: Vec<PendingUsageAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_usage_agreement_view(item));
    }
    views
}

pub fn sum_pending_usage_agreement_view(view: &PendingUsageAgreementView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_usage_agreement("code-414", "title-456");
        let view = to_pending_usage_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_usage_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
