use super::*;

use acme_kit::text;

/// How screens and exports show a credential claim.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialClaimView {
    pub code: String,
    pub name_text: String,
    pub phone: String,
    pub owner_id: String,
    pub cost: String,
    pub duration_minutes: String,
    pub principal: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_credential_claim_view(item: &CredentialClaim) -> CredentialClaimView {
    CredentialClaimView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        phone: item.phone.clone(),
        owner_id: item.owner_id.clone(),
        cost: format!("{:.2}", item.cost),
        duration_minutes: item.duration_minutes.to_string(),
        principal: item.principal.clone(),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_claim_views(items: &[CredentialClaim]) -> Vec<CredentialClaimView> {
    let mut views: Vec<CredentialClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_claim_view(item));
    }
    views
}

pub fn sum_credential_claim_view(view: &CredentialClaimView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_cost + (parsed_duration_minutes as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_claim("code-117", "name-526");
        let view = to_credential_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
