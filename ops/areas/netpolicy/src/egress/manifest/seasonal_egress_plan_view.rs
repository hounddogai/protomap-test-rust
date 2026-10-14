use super::*;

/// How screens and exports show a seasonal egress plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressPlanView {
    pub code_text: String,
    pub name: String,
    pub duration_minutes: String,
    pub unit: String,
    pub floor_value: String,
    pub key_id: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_seasonal_egress_plan_view(item: &SeasonalEgressPlan) -> SeasonalEgressPlanView {
    SeasonalEgressPlanView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        unit: item.unit.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        key_id: item.key_id.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_egress_plan_views(items: &[SeasonalEgressPlan]) -> Vec<SeasonalEgressPlanView> {
    let mut views: Vec<SeasonalEgressPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_egress_plan_view(item));
    }
    views
}

pub fn sum_seasonal_egress_plan_view(view: &SeasonalEgressPlanView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_floor_value + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_egress_plan("code-714", "name-357");
        let view = to_seasonal_egress_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_egress_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
