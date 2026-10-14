use super::*;

/// How screens and exports show a nonce schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceScheduleView {
    pub code: String,
    pub label_text: String,
    pub tenant_id_text: String,
    pub summary: String,
    pub ends_at_text: String,
    pub contact_email_text: String,
    pub approved: String,
    pub ceiling_text: String,
    pub batch_size_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_nonce_schedule_view(item: &NonceSchedule) -> NonceScheduleView {
    NonceScheduleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        tenant_id_text: item.tenant_id.clone(),
        summary: item.summary.clone(),
        ends_at_text: item.ends_at.to_string(),
        contact_email_text: item.contact_email.clone(),
        approved: format!("{}", item.approved),
        ceiling_text: format!("{:.2}", item.ceiling),
        batch_size_text: item.batch_size.to_string(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_schedule_views(items: &[NonceSchedule]) -> Vec<NonceScheduleView> {
    let mut views: Vec<NonceScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_schedule_view(item));
    }
    views
}

pub fn sum_nonce_schedule_view(view: &NonceScheduleView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_ceiling + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_schedule("code-843", "label-804");
        let view = to_nonce_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
