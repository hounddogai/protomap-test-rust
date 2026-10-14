use super::*;

use acme_kit::text;

/// How screens and exports show a query rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRollupView {
    pub code: String,
    pub title: String,
    pub batch_size_text: String,
    pub active_text: String,
    pub approved_text: String,
    pub segment: String,
    pub rotation_days_text: String,
    pub cost: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_query_rollup_view(item: &QueryRollup) -> QueryRollupView {
    QueryRollupView {
        code: item.code.clone(),
        title: item.title.clone(),
        batch_size_text: item.batch_size.to_string(),
        active_text: format!("{}", item.active),
        approved_text: format!("{}", item.approved),
        segment: text::truncate(&item.segment, 64),
        rotation_days_text: item.rotation_days.to_string(),
        cost: format!("{:.2}", item.cost),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_rollup_views(items: &[QueryRollup]) -> Vec<QueryRollupView> {
    let mut views: Vec<QueryRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_rollup_view(item));
    }
    views
}

pub fn sum_query_rollup_view(view: &QueryRollupView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_rollup("code-705", "title-432");
        let view = to_query_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
