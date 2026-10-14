use super::*;

use acme_kit::text;

/// How screens and exports show a quote cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCycleView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub grace_days: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_quote_cycle_view(item: &QuoteCycle) -> QuoteCycleView {
    QuoteCycleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        summary: item.summary.clone(),
        grace_days: item.grace_days.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_cycle_views(items: &[QuoteCycle]) -> Vec<QuoteCycleView> {
    let mut views: Vec<QuoteCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_cycle_view(item));
    }
    views
}

pub fn sum_quote_cycle_view(view: &QuoteCycleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_cycle("code-889", "label-880");
        let view = to_quote_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
