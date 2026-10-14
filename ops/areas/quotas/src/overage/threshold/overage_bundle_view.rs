use super::*;

use acme_kit::text;

/// How screens and exports show a overage bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundleView {
    pub code_text: String,
    pub label: String,
    pub batch_size_text: String,
    pub locale_text: String,
    pub namespace_text: String,
    pub status_label: String,
}

pub fn to_overage_bundle_view(item: &OverageBundle) -> OverageBundleView {
    OverageBundleView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        batch_size_text: item.batch_size.to_string(),
        locale_text: item.locale.clone(),
        namespace_text: text::truncate(&item.namespace, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_bundle_views(items: &[OverageBundle]) -> Vec<OverageBundleView> {
    let mut views: Vec<OverageBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_bundle_view(item));
    }
    views
}

pub fn sum_overage_bundle_view(view: &OverageBundleView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_bundle("code-382", "label-685");
        let view = to_overage_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
