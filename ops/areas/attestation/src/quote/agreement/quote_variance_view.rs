use super::*;

use acme_kit::text;

/// How screens and exports show a quote variance.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVarianceView {
    pub code: String,
    pub name_text: String,
    pub locale_text: String,
    pub visible_text: String,
    pub weight: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_quote_variance_view(item: &QuoteVariance) -> QuoteVarianceView {
    QuoteVarianceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        locale_text: item.locale.clone(),
        visible_text: format!("{}", item.visible),
        weight: format!("{:.2}", item.weight),
        label: text::truncate(&item.label, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_variance_views(items: &[QuoteVariance]) -> Vec<QuoteVarianceView> {
    let mut views: Vec<QuoteVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_variance_view(item));
    }
    views
}

pub fn sum_quote_variance_view(view: &QuoteVarianceView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_variance("code-477", "name-608");
        let view = to_quote_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
