use super::*;

/// How screens and exports show a seasonal reservation target.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTargetView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub rotation_days: String,
    pub name: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_seasonal_reservation_target_view(item: &SeasonalReservationTarget) -> SeasonalReservationTargetView {
    SeasonalReservationTargetView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        rotation_days: item.rotation_days.to_string(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_reservation_target_views(items: &[SeasonalReservationTarget]) -> Vec<SeasonalReservationTargetView> {
    let mut views: Vec<SeasonalReservationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_reservation_target_view(item));
    }
    views
}

pub fn sum_seasonal_reservation_target_view(view: &SeasonalReservationTargetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_reservation_target("code-314", "label-452");
        let view = to_seasonal_reservation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_reservation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
