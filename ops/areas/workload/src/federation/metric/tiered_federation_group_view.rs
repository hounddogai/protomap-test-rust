use super::*;

/// How screens and exports show a tiered federation group.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroupView {
    pub code_text: String,
    pub label: String,
    pub expires_at: String,
    pub attempts: String,
    pub currency: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_tiered_federation_group_view(item: &TieredFederationGroup) -> TieredFederationGroupView {
    TieredFederationGroupView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        attempts: item.attempts.to_string(),
        currency: item.currency.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_federation_group_views(items: &[TieredFederationGroup]) -> Vec<TieredFederationGroupView> {
    let mut views: Vec<TieredFederationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_federation_group_view(item));
    }
    views
}

pub fn sum_tiered_federation_group_view(view: &TieredFederationGroupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_attempts as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_federation_group("code-312", "label-502");
        let view = to_tiered_federation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_federation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
