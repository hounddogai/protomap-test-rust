use super::*;

use acme_kit::text;

/// How screens and exports show a primary reservation template.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateView {
    pub code_text: String,
    pub title: String,
    pub window_days: String,
    pub phone: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_primary_reservation_template_view(item: &PrimaryReservationTemplate) -> PrimaryReservationTemplateView {
    PrimaryReservationTemplateView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        phone: item.phone.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_reservation_template_views(
    items: &[PrimaryReservationTemplate],
) -> Vec<PrimaryReservationTemplateView> {
    let mut views: Vec<PrimaryReservationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_reservation_template_view(item));
    }
    views
}

pub fn sum_primary_reservation_template_view(view: &PrimaryReservationTemplateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_reservation_template("code-201", "title-779");
        let view = to_primary_reservation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_reservation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
