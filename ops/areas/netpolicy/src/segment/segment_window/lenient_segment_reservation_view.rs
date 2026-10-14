use super::*;

use acme_kit::text;

/// How screens and exports show a lenient segment reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationView {
    pub code: String,
    pub name: String,
    pub starts_at: String,
    pub version_text: String,
    pub namespace: String,
    pub updated_at: String,
    pub visible: String,
    pub scope_text: String,
    pub notes_text: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_lenient_segment_reservation_view(item: &LenientSegmentReservation) -> LenientSegmentReservationView {
    LenientSegmentReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        version_text: item.version.to_string(),
        namespace: item.namespace.clone(),
        updated_at: item.updated_at.to_string(),
        visible: format!("{}", item.visible),
        scope_text: item.scope.clone(),
        notes_text: item.notes.clone(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_segment_reservation_views(items: &[LenientSegmentReservation]) -> Vec<LenientSegmentReservationView> {
    let mut views: Vec<LenientSegmentReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_segment_reservation_view(item));
    }
    views
}

pub fn sum_lenient_segment_reservation_view(view: &LenientSegmentReservationView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_version as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_segment_reservation("code-496", "name-999");
        let view = to_lenient_segment_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_segment_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
