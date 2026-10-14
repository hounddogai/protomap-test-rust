use super::*;

/// How screens and exports show a grant budget.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantBudgetView {
    pub code: String,
    pub name: String,
    pub region_text: String,
    pub host_text: String,
    pub archived: String,
    pub batch_size_text: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_grant_budget_view(item: &GrantBudget) -> GrantBudgetView {
    GrantBudgetView {
        code: item.code.clone(),
        name: item.name.clone(),
        region_text: item.region.clone(),
        host_text: item.host.clone(),
        archived: format!("{}", item.archived),
        batch_size_text: item.batch_size.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_budget_views(items: &[GrantBudget]) -> Vec<GrantBudgetView> {
    let mut views: Vec<GrantBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_budget_view(item));
    }
    views
}

pub fn sum_grant_budget_view(view: &GrantBudgetView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_budget("code-610", "name-359");
        let view = to_grant_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
