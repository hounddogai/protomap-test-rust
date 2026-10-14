use super::*;

use acme_kit::text;

/// How screens and exports show a rolling quote bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleView {
    pub code_text: String,
    pub name: String,
    pub key_id: String,
    pub cipher: String,
    pub external_ref_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_rolling_quote_bundle_view(item: &RollingQuoteBundle) -> RollingQuoteBundleView {
    RollingQuoteBundleView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        key_id: item.key_id.clone(),
        cipher: text::truncate(&item.cipher, 64),
        external_ref_text: text::truncate(&item.external_ref, 40),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_quote_bundle_views(items: &[RollingQuoteBundle]) -> Vec<RollingQuoteBundleView> {
    let mut views: Vec<RollingQuoteBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_quote_bundle_view(item));
    }
    views
}

pub fn sum_rolling_quote_bundle_view(view: &RollingQuoteBundleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
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
        let item = new_rolling_quote_bundle("code-314", "name-273");
        let view = to_rolling_quote_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_quote_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
