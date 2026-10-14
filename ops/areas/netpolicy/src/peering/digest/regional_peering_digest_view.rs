use super::*;

/// How screens and exports show a regional peering digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringDigestView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub created_at: String,
    pub expires_at_text: String,
    pub external_ref: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_regional_peering_digest_view(item: &RegionalPeeringDigest) -> RegionalPeeringDigestView {
    RegionalPeeringDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        created_at: item.created_at.to_string(),
        expires_at_text: item.expires_at.to_string(),
        external_ref: item.external_ref.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_peering_digest_views(items: &[RegionalPeeringDigest]) -> Vec<RegionalPeeringDigestView> {
    let mut views: Vec<RegionalPeeringDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_peering_digest_view(item));
    }
    views
}

pub fn sum_regional_peering_digest_view(view: &RegionalPeeringDigestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_created_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_peering_digest("code-240", "name-353");
        let view = to_regional_peering_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_peering_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
