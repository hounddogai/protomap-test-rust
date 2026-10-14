use super::*;

/// How screens and exports show a regional isolation tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationTierView {
    pub code: String,
    pub label: String,
    pub issuer_text: String,
    pub cipher_text: String,
    pub rate: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_regional_isolation_tier_view(item: &RegionalIsolationTier) -> RegionalIsolationTierView {
    RegionalIsolationTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer_text: item.issuer.clone(),
        cipher_text: item.cipher.clone(),
        rate: format!("{:.2}", item.rate),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_isolation_tier_views(items: &[RegionalIsolationTier]) -> Vec<RegionalIsolationTierView> {
    let mut views: Vec<RegionalIsolationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_isolation_tier_view(item));
    }
    views
}

pub fn sum_regional_isolation_tier_view(view: &RegionalIsolationTierView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_isolation_tier("code-804", "label-342");
        let view = to_regional_isolation_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_isolation_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
