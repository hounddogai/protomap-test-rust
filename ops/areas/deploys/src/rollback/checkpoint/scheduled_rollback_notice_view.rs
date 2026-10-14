use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled rollback notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackNoticeView {
    pub code: String,
    pub label: String,
    pub duration_minutes: String,
    pub notes: String,
    pub discount: String,
    pub currency: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_scheduled_rollback_notice_view(item: &ScheduledRollbackNotice) -> ScheduledRollbackNoticeView {
    ScheduledRollbackNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        notes: text::truncate(&item.notes, 120),
        discount: format!("{:.2}", item.discount),
        currency: item.currency.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rollback_notice_views(items: &[ScheduledRollbackNotice]) -> Vec<ScheduledRollbackNoticeView> {
    let mut views: Vec<ScheduledRollbackNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rollback_notice_view(item));
    }
    views
}

pub fn sum_scheduled_rollback_notice_view(view: &ScheduledRollbackNoticeView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_discount + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_rollback_notice("code-295", "label-254");
        let view = to_scheduled_rollback_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_rollback_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
