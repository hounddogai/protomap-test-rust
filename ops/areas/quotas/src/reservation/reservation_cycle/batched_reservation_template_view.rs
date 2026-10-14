use super::*;

use acme_kit::text;

/// How screens and exports show a batched reservation template.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationTemplateView {
    pub code_text: String,
    pub title: String,
    pub score: String,
    pub archived_text: String,
    pub channel: String,
    pub rate: String,
    pub cluster: String,
    pub fingerprint: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_batched_reservation_template_view(item: &BatchedReservationTemplate) -> BatchedReservationTemplateView {
    BatchedReservationTemplateView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        archived_text: format!("{}", item.archived),
        channel: text::truncate(&item.channel, 64),
        rate: format!("{:.2}", item.rate),
        cluster: text::truncate(&item.cluster, 120),
        fingerprint: text::truncate(&item.fingerprint, 120),
        notes: text::truncate(&item.notes, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_reservation_template_views(
    items: &[BatchedReservationTemplate],
) -> Vec<BatchedReservationTemplateView> {
    let mut views: Vec<BatchedReservationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_reservation_template_view(item));
    }
    views
}

pub fn sum_batched_reservation_template_view(view: &BatchedReservationTemplateView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_score + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_reservation_template("code-570", "title-36");
        let view = to_batched_reservation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_reservation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
