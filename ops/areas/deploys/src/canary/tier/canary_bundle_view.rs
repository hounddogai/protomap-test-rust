use super::*;

use acme_kit::text;

/// How screens and exports show a canary bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBundleView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub issuer: String,
    pub notes: String,
    pub locale: String,
    pub sequence: String,
    pub source_text: String,
    pub updated_at: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_canary_bundle_view(item: &CanaryBundle) -> CanaryBundleView {
    CanaryBundleView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        quantity: item.quantity.to_string(),
        issuer: item.issuer.clone(),
        notes: item.notes.clone(),
        locale: item.locale.clone(),
        sequence: item.sequence.to_string(),
        source_text: text::truncate(&item.source, 64),
        updated_at: item.updated_at.to_string(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_bundle_views(items: &[CanaryBundle]) -> Vec<CanaryBundleView> {
    let mut views: Vec<CanaryBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_bundle_view(item));
    }
    views
}

pub fn sum_canary_bundle_view(view: &CanaryBundleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_sequence as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_bundle("code-487", "name-362");
        let view = to_canary_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
