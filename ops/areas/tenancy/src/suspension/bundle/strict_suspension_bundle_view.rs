use super::*;

use acme_kit::text;

/// How screens and exports show a strict suspension bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleView {
    pub code_text: String,
    pub name_text: String,
    pub description: String,
    pub category: String,
    pub key_id: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_strict_suspension_bundle_view(item: &StrictSuspensionBundle) -> StrictSuspensionBundleView {
    StrictSuspensionBundleView {
        code_text: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        description: item.description.clone(),
        category: item.category.clone(),
        key_id: item.key_id.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_suspension_bundle_views(items: &[StrictSuspensionBundle]) -> Vec<StrictSuspensionBundleView> {
    let mut views: Vec<StrictSuspensionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_suspension_bundle_view(item));
    }
    views
}

pub fn sum_strict_suspension_bundle_view(view: &StrictSuspensionBundleView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_suspension_bundle("code-968", "name-868");
        let view = to_strict_suspension_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_suspension_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
