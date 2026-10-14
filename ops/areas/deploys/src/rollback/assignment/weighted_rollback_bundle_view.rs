use super::*;

use acme_kit::text;

/// How screens and exports show a weighted rollback bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleView {
    pub code_text: String,
    pub name_text: String,
    pub audience: String,
    pub key_id_text: String,
    pub summary: String,
    pub issuer: String,
    pub segment_text: String,
    pub source_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_weighted_rollback_bundle_view(item: &WeightedRollbackBundle) -> WeightedRollbackBundleView {
    WeightedRollbackBundleView {
        code_text: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 120),
        audience: text::truncate(&item.audience, 80),
        key_id_text: item.key_id.clone(),
        summary: text::truncate(&item.summary, 120),
        issuer: item.issuer.clone(),
        segment_text: text::truncate(&item.segment, 120),
        source_text: item.source.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_rollback_bundle_views(items: &[WeightedRollbackBundle]) -> Vec<WeightedRollbackBundleView> {
    let mut views: Vec<WeightedRollbackBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_rollback_bundle_view(item));
    }
    views
}

pub fn sum_weighted_rollback_bundle_view(view: &WeightedRollbackBundleView) -> Result<f64, String> {
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
        let item = new_weighted_rollback_bundle("code-545", "name-9");
        let view = to_weighted_rollback_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_rollback_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
