use super::*;

use acme_kit::text;

/// How screens and exports show a slo manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestView {
    pub code: String,
    pub title_text: String,
    pub rate_text: String,
    pub region: String,
    pub minimum: String,
    pub visible_text: String,
    pub discount_text: String,
    pub pinned: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_slo_manifest_view(item: &SloManifest) -> SloManifestView {
    SloManifestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        rate_text: format!("{:.2}", item.rate),
        region: text::truncate(&item.region, 40),
        minimum: format!("{:.2}", item.minimum),
        visible_text: format!("{}", item.visible),
        discount_text: format!("{:.2}", item.discount),
        pinned: format!("{}", item.pinned),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_manifest_views(items: &[SloManifest]) -> Vec<SloManifestView> {
    let mut views: Vec<SloManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_manifest_view(item));
    }
    views
}

pub fn sum_slo_manifest_view(view: &SloManifestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_rate + parsed_minimum + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_manifest("code-530", "title-439");
        let view = to_slo_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
