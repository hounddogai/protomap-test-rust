use super::*;

use acme_kit::text;

/// How screens and exports show a limit bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitBundleView {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub source: String,
    pub cipher_text: String,
    pub created_at: String,
    pub minimum: String,
    pub budget: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_limit_bundle_view(item: &LimitBundle) -> LimitBundleView {
    LimitBundleView {
        code: item.code.clone(),
        label: item.label.clone(),
        segment: text::truncate(&item.segment, 40),
        source: text::truncate(&item.source, 80),
        cipher_text: item.cipher.clone(),
        created_at: item.created_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        budget: format!("{:.2}", item.budget),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_bundle_views(items: &[LimitBundle]) -> Vec<LimitBundleView> {
    let mut views: Vec<LimitBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_bundle_view(item));
    }
    views
}

pub fn sum_limit_bundle_view(view: &LimitBundleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_minimum + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_bundle("code-563", "label-599");
        let view = to_limit_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
