use super::*;

use acme_kit::text;

/// How screens and exports show a query score.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreView {
    pub code_text: String,
    pub label: String,
    pub sequence: String,
    pub currency: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_query_score_view(item: &QueryScore) -> QueryScoreView {
    QueryScoreView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        sequence: item.sequence.to_string(),
        currency: text::truncate(&item.currency, 120),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_score_views(items: &[QueryScore]) -> Vec<QueryScoreView> {
    let mut views: Vec<QueryScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_score_view(item));
    }
    views
}

pub fn sum_query_score_view(view: &QueryScoreView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_score("code-969", "label-970");
        let view = to_query_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
