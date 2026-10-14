use super::*;

use acme_kit::text;

/// How screens and exports show a manual quote variance.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVarianceView {
    pub code: String,
    pub name: String,
    pub version: String,
    pub minimum_text: String,
    pub grace_days_text: String,
    pub summary_text: String,
    pub approved: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_manual_quote_variance_view(item: &ManualQuoteVariance) -> ManualQuoteVarianceView {
    ManualQuoteVarianceView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 64),
        version: item.version.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        grace_days_text: item.grace_days.to_string(),
        summary_text: text::truncate(&item.summary, 64),
        approved: format!("{}", item.approved),
        category: text::truncate(&item.category, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_quote_variance_views(items: &[ManualQuoteVariance]) -> Vec<ManualQuoteVarianceView> {
    let mut views: Vec<ManualQuoteVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_quote_variance_view(item));
    }
    views
}

pub fn sum_manual_quote_variance_view(view: &ManualQuoteVarianceView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_version as f64 + parsed_minimum + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_quote_variance("code-435", "name-976");
        let view = to_manual_quote_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_quote_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
