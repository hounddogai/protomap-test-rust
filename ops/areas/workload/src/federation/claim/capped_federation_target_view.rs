use super::*;

use acme_kit::text;

/// How screens and exports show a capped federation target.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationTargetView {
    pub code: String,
    pub name: String,
    pub floor_value: String,
    pub checksum: String,
    pub source: String,
    pub namespace: String,
    pub country: String,
    pub margin: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_capped_federation_target_view(item: &CappedFederationTarget) -> CappedFederationTargetView {
    CappedFederationTargetView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 40),
        floor_value: format!("{:.2}", item.floor_value),
        checksum: text::truncate(&item.checksum, 120),
        source: item.source.clone(),
        namespace: text::truncate(&item.namespace, 80),
        country: item.country.clone(),
        margin: format!("{:.2}", item.margin),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_federation_target_views(items: &[CappedFederationTarget]) -> Vec<CappedFederationTargetView> {
    let mut views: Vec<CappedFederationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_federation_target_view(item));
    }
    views
}

pub fn sum_capped_federation_target_view(view: &CappedFederationTargetView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_floor_value + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_federation_target("code-702", "name-954");
        let view = to_capped_federation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_federation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
