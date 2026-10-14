use super::*;

/// How screens and exports show a identity mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMappingView {
    pub code_text: String,
    pub title_text: String,
    pub external_ref_text: String,
    pub ratio: String,
    pub source: String,
    pub quota_units: String,
    pub quantity: String,
    pub created_at_text: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_identity_mapping_view(item: &IdentityMapping) -> IdentityMappingView {
    IdentityMappingView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        external_ref_text: item.external_ref.clone(),
        ratio: format!("{:.2}", item.ratio),
        source: item.source.clone(),
        quota_units: item.quota_units.to_string(),
        quantity: item.quantity.to_string(),
        created_at_text: item.created_at.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_mapping_views(items: &[IdentityMapping]) -> Vec<IdentityMappingView> {
    let mut views: Vec<IdentityMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_mapping_view(item));
    }
    views
}

pub fn sum_identity_mapping_view(view: &IdentityMappingView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_ratio + (parsed_quota_units as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_mapping("code-526", "title-473");
        let view = to_identity_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
