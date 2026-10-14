use super::*;

/// How screens and exports show a scheduled silence budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudgetView {
    pub code: String,
    pub name_text: String,
    pub created_at: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_scheduled_silence_budget_view(item: &ScheduledSilenceBudget) -> ScheduledSilenceBudgetView {
    ScheduledSilenceBudgetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        created_at: item.created_at.to_string(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_silence_budget_views(items: &[ScheduledSilenceBudget]) -> Vec<ScheduledSilenceBudgetView> {
    let mut views: Vec<ScheduledSilenceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_silence_budget_view(item));
    }
    views
}

pub fn sum_scheduled_silence_budget_view(view: &ScheduledSilenceBudgetView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_silence_budget("code-918", "name-875");
        let view = to_scheduled_silence_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_silence_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
