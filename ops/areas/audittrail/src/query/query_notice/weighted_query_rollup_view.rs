use super::*;

use acme_kit::text;

/// How screens and exports show a weighted query rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryRollupView {
    pub code: String,
    pub title: String,
    pub maximum: String,
    pub pinned: String,
    pub discount: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_weighted_query_rollup_view(item: &WeightedQueryRollup) -> WeightedQueryRollupView {
    WeightedQueryRollupView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        maximum: format!("{:.2}", item.maximum),
        pinned: format!("{}", item.pinned),
        discount: format!("{:.2}", item.discount),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_query_rollup_views(items: &[WeightedQueryRollup]) -> Vec<WeightedQueryRollupView> {
    let mut views: Vec<WeightedQueryRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_query_rollup_view(item));
    }
    views
}

pub fn sum_weighted_query_rollup_view(view: &WeightedQueryRollupView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_maximum + parsed_discount + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_query_rollup("code-181", "title-987");
        let view = to_weighted_query_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_query_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
