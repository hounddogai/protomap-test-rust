use super::*;

/// How screens and exports show a fallback quote schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteScheduleView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub starts_at: String,
    pub active: String,
    pub rotation_days_text: String,
    pub budget_text: String,
    pub principal: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_fallback_quote_schedule_view(item: &FallbackQuoteSchedule) -> FallbackQuoteScheduleView {
    FallbackQuoteScheduleView {
        code: item.code.clone(),
        name: item.name.clone(),
        namespace: item.namespace.clone(),
        starts_at: item.starts_at.to_string(),
        active: format!("{}", item.active),
        rotation_days_text: item.rotation_days.to_string(),
        budget_text: format!("{:.2}", item.budget),
        principal: item.principal.clone(),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_quote_schedule_views(items: &[FallbackQuoteSchedule]) -> Vec<FallbackQuoteScheduleView> {
    let mut views: Vec<FallbackQuoteScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_quote_schedule_view(item));
    }
    views
}

pub fn sum_fallback_quote_schedule_view(view: &FallbackQuoteScheduleView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_rotation_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_quote_schedule("code-833", "name-563");
        let view = to_fallback_quote_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_quote_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
