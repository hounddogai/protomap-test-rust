use super::*;

/// How screens and exports show a pending freeze reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeReservationView {
    pub code: String,
    pub name: String,
    pub version_text: String,
    pub rotation_days: String,
    pub batch_size: String,
    pub threshold: String,
    pub active: String,
    pub ratio_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_pending_freeze_reservation_view(item: &PendingFreezeReservation) -> PendingFreezeReservationView {
    PendingFreezeReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        rotation_days: item.rotation_days.to_string(),
        batch_size: item.batch_size.to_string(),
        threshold: format!("{:.2}", item.threshold),
        active: format!("{}", item.active),
        ratio_text: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_freeze_reservation_views(items: &[PendingFreezeReservation]) -> Vec<PendingFreezeReservationView> {
    let mut views: Vec<PendingFreezeReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_freeze_reservation_view(item));
    }
    views
}

pub fn sum_pending_freeze_reservation_view(view: &PendingFreezeReservationView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_version as f64 + (parsed_rotation_days as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_freeze_reservation("code-870", "name-18");
        let view = to_pending_freeze_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_freeze_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
