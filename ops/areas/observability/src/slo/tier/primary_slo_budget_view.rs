use super::*;

/// How screens and exports show a primary slo budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudgetView {
    pub code: String,
    pub title_text: String,
    pub grace_days_text: String,
    pub rate_text: String,
    pub batch_size: String,
    pub capacity: String,
    pub pinned: String,
    pub ratio: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_primary_slo_budget_view(item: &PrimarySloBudget) -> PrimarySloBudgetView {
    PrimarySloBudgetView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        grace_days_text: item.grace_days.to_string(),
        rate_text: format!("{:.2}", item.rate),
        batch_size: item.batch_size.to_string(),
        capacity: item.capacity.to_string(),
        pinned: format!("{}", item.pinned),
        ratio: format!("{:.2}", item.ratio),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_slo_budget_views(items: &[PrimarySloBudget]) -> Vec<PrimarySloBudgetView> {
    let mut views: Vec<PrimarySloBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_slo_budget_view(item));
    }
    views
}

pub fn sum_primary_slo_budget_view(view: &PrimarySloBudgetView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_rate + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_slo_budget("code-821", "title-457");
        let view = to_primary_slo_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_slo_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
