use super::*;

use acme_kit::text;

/// How screens and exports show a tenant claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantClaimView {
    pub code: String,
    pub title: String,
    pub margin: String,
    pub region: String,
    pub notes: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_tenant_claim_view(item: &TenantClaim) -> TenantClaimView {
    TenantClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        region: text::truncate(&item.region, 64),
        notes: item.notes.clone(),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_claim_views(items: &[TenantClaim]) -> Vec<TenantClaimView> {
    let mut views: Vec<TenantClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_claim_view(item));
    }
    views
}

pub fn sum_tenant_claim_view(view: &TenantClaimView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_margin + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_claim("code-764", "title-167");
        let view = to_tenant_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
