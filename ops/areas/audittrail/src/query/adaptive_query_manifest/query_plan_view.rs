use super::*;

use acme_kit::text;

/// How screens and exports show a query plan.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanView {
    pub code: String,
    pub title: String,
    pub cost: String,
    pub principal: String,
    pub label: String,
    pub revision: String,
    pub rate: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_query_plan_view(item: &QueryPlan) -> QueryPlanView {
    QueryPlanView {
        code: item.code.clone(),
        title: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        principal: text::truncate(&item.principal, 64),
        label: item.label.clone(),
        revision: item.revision.to_string(),
        rate: format!("{:.2}", item.rate),
        timezone: text::truncate(&item.timezone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_plan_views(items: &[QueryPlan]) -> Vec<QueryPlanView> {
    let mut views: Vec<QueryPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_plan_view(item));
    }
    views
}

pub fn sum_query_plan_view(view: &QueryPlanView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_cost + (parsed_revision as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_plan("code-981", "title-220");
        let view = to_query_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
