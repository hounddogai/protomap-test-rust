use super::*;

use acme_kit::text;

/// How screens and exports show a legacy quote group.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroupView {
    pub code: String,
    pub name_text: String,
    pub locale: String,
    pub score_text: String,
    pub revision_text: String,
    pub owner_id: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_legacy_quote_group_view(item: &LegacyQuoteGroup) -> LegacyQuoteGroupView {
    LegacyQuoteGroupView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        locale: text::truncate(&item.locale, 120),
        score_text: format!("{:.2}", item.score),
        revision_text: item.revision.to_string(),
        owner_id: item.owner_id.clone(),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_quote_group_views(items: &[LegacyQuoteGroup]) -> Vec<LegacyQuoteGroupView> {
    let mut views: Vec<LegacyQuoteGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_quote_group_view(item));
    }
    views
}

pub fn sum_legacy_quote_group_view(view: &LegacyQuoteGroupView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_score + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_quote_group("code-211", "name-851");
        let view = to_legacy_quote_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_quote_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
