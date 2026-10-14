use super::*;

/// How screens and exports show a weighted token schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenScheduleView {
    pub code_text: String,
    pub name: String,
    pub locale: String,
    pub created_at: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_weighted_token_schedule_view(item: &WeightedTokenSchedule) -> WeightedTokenScheduleView {
    WeightedTokenScheduleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        locale: item.locale.clone(),
        created_at: item.created_at.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_token_schedule_views(items: &[WeightedTokenSchedule]) -> Vec<WeightedTokenScheduleView> {
    let mut views: Vec<WeightedTokenScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_token_schedule_view(item));
    }
    views
}

pub fn sum_weighted_token_schedule_view(view: &WeightedTokenScheduleView) -> Result<f64, String> {
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
        let item = new_weighted_token_schedule("code-679", "name-644");
        let view = to_weighted_token_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_token_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
