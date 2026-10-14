use super::*;

use acme_kit::text;

/// How screens and exports show a usage baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineView {
    pub code: String,
    pub name: String,
    pub scope_text: String,
    pub currency: String,
    pub audience_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_usage_baseline_view(item: &UsageBaseline) -> UsageBaselineView {
    UsageBaselineView {
        code: item.code.clone(),
        name: item.name.clone(),
        scope_text: text::truncate(&item.scope, 80),
        currency: item.currency.clone(),
        audience_text: item.audience.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_baseline_views(items: &[UsageBaseline]) -> Vec<UsageBaselineView> {
    let mut views: Vec<UsageBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_baseline_view(item));
    }
    views
}

pub fn sum_usage_baseline_view(view: &UsageBaselineView) -> Result<f64, String> {
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
        let item = new_usage_baseline("code-537", "name-156");
        let view = to_usage_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
