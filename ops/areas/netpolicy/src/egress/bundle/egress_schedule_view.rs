use super::*;

/// How screens and exports show a egress schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressScheduleView {
    pub code: String,
    pub name: String,
    pub grace_days: String,
    pub key_id: String,
    pub burst: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_egress_schedule_view(item: &EgressSchedule) -> EgressScheduleView {
    EgressScheduleView {
        code: item.code.clone(),
        name: item.name.clone(),
        grace_days: item.grace_days.to_string(),
        key_id: item.key_id.clone(),
        burst: item.burst.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_schedule_views(items: &[EgressSchedule]) -> Vec<EgressScheduleView> {
    let mut views: Vec<EgressScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_schedule_view(item));
    }
    views
}

pub fn sum_egress_schedule_view(view: &EgressScheduleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_burst as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_schedule("code-741", "name-542");
        let view = to_egress_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
