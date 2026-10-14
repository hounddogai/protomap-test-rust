use super::*;

use acme_kit::text;

/// How screens and exports show a canary tier.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTierView {
    pub code_text: String,
    pub title_text: String,
    pub ratio: String,
    pub approved_text: String,
    pub cipher: String,
    pub source: String,
    pub cost: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_canary_tier_view(item: &CanaryTier) -> CanaryTierView {
    CanaryTierView {
        code_text: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 80),
        ratio: format!("{:.2}", item.ratio),
        approved_text: format!("{}", item.approved),
        cipher: item.cipher.clone(),
        source: item.source.clone(),
        cost: format!("{:.2}", item.cost),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_tier_views(items: &[CanaryTier]) -> Vec<CanaryTierView> {
    let mut views: Vec<CanaryTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_tier_view(item));
    }
    views
}

pub fn sum_canary_tier_view(view: &CanaryTierView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ratio + parsed_cost + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_tier("code-329", "title-860");
        let view = to_canary_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
