use super::*;

/// How screens and exports show a query summary.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySummaryView {
    pub code: String,
    pub name_text: String,
    pub active: String,
    pub reviewer_text: String,
    pub namespace: String,
    pub score_text: String,
    pub rate_text: String,
    pub cluster: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_query_summary_view(item: &QuerySummary) -> QuerySummaryView {
    QuerySummaryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        active: format!("{}", item.active),
        reviewer_text: item.reviewer.clone(),
        namespace: item.namespace.clone(),
        score_text: format!("{:.2}", item.score),
        rate_text: format!("{:.2}", item.rate),
        cluster: item.cluster.clone(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_summary_views(items: &[QuerySummary]) -> Vec<QuerySummaryView> {
    let mut views: Vec<QuerySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_summary_view(item));
    }
    views
}

pub fn sum_query_summary_view(view: &QuerySummaryView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_score + parsed_rate + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_summary("code-652", "name-328");
        let view = to_query_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
