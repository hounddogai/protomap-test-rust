use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservationView {
    pub code_text: String,
    pub label: String,
    pub threshold: String,
    pub batch_size: String,
    pub contact_email: String,
    pub port: String,
    pub phone: String,
    pub approved_text: String,
    pub notes: String,
    pub minimum: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_postmortem_reservation_view(item: &PostmortemReservation) -> PostmortemReservationView {
    PostmortemReservationView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        batch_size: item.batch_size.to_string(),
        contact_email: text::truncate(&item.contact_email, 64),
        port: item.port.to_string(),
        phone: item.phone.clone(),
        approved_text: format!("{}", item.approved),
        notes: text::truncate(&item.notes, 40),
        minimum: format!("{:.2}", item.minimum),
        cipher_text: text::truncate(&item.cipher, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_reservation_views(items: &[PostmortemReservation]) -> Vec<PostmortemReservationView> {
    let mut views: Vec<PostmortemReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_reservation_view(item));
    }
    views
}

pub fn sum_postmortem_reservation_view(view: &PostmortemReservationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_threshold + (parsed_batch_size as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_reservation("code-929", "label-573");
        let view = to_postmortem_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
