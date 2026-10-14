use super::*;

/// How screens and exports show a capped budget burn schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleView {
    pub code: String,
    pub label: String,
    pub duration_minutes: String,
    pub audience: String,
    pub segment: String,
    pub grace_days: String,
    pub title_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_capped_budget_burn_schedule_view(item: &CappedBudgetBurnSchedule) -> CappedBudgetBurnScheduleView {
    CappedBudgetBurnScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        audience: item.audience.clone(),
        segment: item.segment.clone(),
        grace_days: item.grace_days.to_string(),
        title_text: item.title.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_budget_burn_schedule_views(items: &[CappedBudgetBurnSchedule]) -> Vec<CappedBudgetBurnScheduleView> {
    let mut views: Vec<CappedBudgetBurnScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_budget_burn_schedule_view(item));
    }
    views
}

pub fn sum_capped_budget_burn_schedule_view(view: &CappedBudgetBurnScheduleView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_budget_burn_schedule("code-728", "label-321");
        let view = to_capped_budget_burn_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_budget_burn_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
