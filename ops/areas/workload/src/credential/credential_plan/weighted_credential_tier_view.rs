use super::*;

use acme_kit::text;

/// How screens and exports show a weighted credential tier.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialTierView {
    pub code: String,
    pub title: String,
    pub approved: String,
    pub description_text: String,
    pub capacity: String,
    pub source: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_weighted_credential_tier_view(item: &WeightedCredentialTier) -> WeightedCredentialTierView {
    WeightedCredentialTierView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        description_text: text::truncate(&item.description, 80),
        capacity: item.capacity.to_string(),
        source: item.source.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_credential_tier_views(items: &[WeightedCredentialTier]) -> Vec<WeightedCredentialTierView> {
    let mut views: Vec<WeightedCredentialTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_credential_tier_view(item));
    }
    views
}

pub fn sum_weighted_credential_tier_view(view: &WeightedCredentialTierView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_credential_tier("code-78", "title-409");
        let view = to_weighted_credential_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_credential_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
