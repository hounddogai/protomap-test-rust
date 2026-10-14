use super::*;

use acme_kit::text;

/// How screens and exports show a reservation template.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplateView {
    pub code: String,
    pub title_text: String,
    pub expires_at: String,
    pub rate: String,
    pub rotation_days: String,
    pub revision_text: String,
    pub priority: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_reservation_template_view(item: &ReservationTemplate) -> ReservationTemplateView {
    ReservationTemplateView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        expires_at: item.expires_at.to_string(),
        rate: format!("{:.2}", item.rate),
        rotation_days: item.rotation_days.to_string(),
        revision_text: item.revision.to_string(),
        priority: item.priority.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_template_views(items: &[ReservationTemplate]) -> Vec<ReservationTemplateView> {
    let mut views: Vec<ReservationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_template_view(item));
    }
    views
}

pub fn sum_reservation_template_view(view: &ReservationTemplateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_rate + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_template("code-816", "title-483");
        let view = to_reservation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
