use super::*;

/// How screens and exports show a adaptive credential tier.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialTierView {
    pub code_text: String,
    pub name_text: String,
    pub threshold_text: String,
    pub margin: String,
    pub owner_id: String,
    pub title: String,
    pub expires_at: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_adaptive_credential_tier_view(item: &AdaptiveCredentialTier) -> AdaptiveCredentialTierView {
    AdaptiveCredentialTierView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        margin: format!("{:.2}", item.margin),
        owner_id: item.owner_id.clone(),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_credential_tier_views(items: &[AdaptiveCredentialTier]) -> Vec<AdaptiveCredentialTierView> {
    let mut views: Vec<AdaptiveCredentialTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_credential_tier_view(item));
    }
    views
}

pub fn sum_adaptive_credential_tier_view(view: &AdaptiveCredentialTierView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_threshold + parsed_margin + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_credential_tier("code-183", "name-160");
        let view = to_adaptive_credential_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_credential_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
