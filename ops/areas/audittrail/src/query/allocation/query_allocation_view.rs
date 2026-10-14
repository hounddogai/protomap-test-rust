use super::*;

/// How screens and exports show a query allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAllocationView {
    pub code_text: String,
    pub name: String,
    pub ends_at: String,
    pub scope: String,
    pub archived: String,
    pub priority_text: String,
    pub margin: String,
    pub issuer: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_query_allocation_view(item: &QueryAllocation) -> QueryAllocationView {
    QueryAllocationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        scope: item.scope.clone(),
        archived: format!("{}", item.archived),
        priority_text: item.priority.to_string(),
        margin: format!("{:.2}", item.margin),
        issuer: item.issuer.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_allocation_views(items: &[QueryAllocation]) -> Vec<QueryAllocationView> {
    let mut views: Vec<QueryAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_allocation_view(item));
    }
    views
}

pub fn sum_query_allocation_view(view: &QueryAllocationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_priority as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_allocation("code-804", "name-268");
        let view = to_query_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
