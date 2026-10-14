use super::*;

/// How screens and exports show a manual tenant claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaimView {
    pub code_text: String,
    pub title_text: String,
    pub description: String,
    pub external_ref: String,
    pub timezone_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_manual_tenant_claim_view(item: &ManualTenantClaim) -> ManualTenantClaimView {
    ManualTenantClaimView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        description: item.description.clone(),
        external_ref: item.external_ref.clone(),
        timezone_text: item.timezone.clone(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_tenant_claim_views(items: &[ManualTenantClaim]) -> Vec<ManualTenantClaimView> {
    let mut views: Vec<ManualTenantClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_tenant_claim_view(item));
    }
    views
}

pub fn sum_manual_tenant_claim_view(view: &ManualTenantClaimView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_tenant_claim("code-709", "title-729");
        let view = to_manual_tenant_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_tenant_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
