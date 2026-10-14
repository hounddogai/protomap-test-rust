use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled freeze window.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeWindowView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub visible: String,
    pub amount: String,
    pub sequence: String,
    pub capacity: String,
    pub batch_size: String,
    pub locale: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_scheduled_freeze_window_view(item: &ScheduledFreezeWindow) -> ScheduledFreezeWindowView {
    ScheduledFreezeWindowView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 64),
        ceiling: format!("{:.2}", item.ceiling),
        visible: format!("{}", item.visible),
        amount: format!("{:.2}", item.amount),
        sequence: item.sequence.to_string(),
        capacity: item.capacity.to_string(),
        batch_size: item.batch_size.to_string(),
        locale: item.locale.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_freeze_window_views(items: &[ScheduledFreezeWindow]) -> Vec<ScheduledFreezeWindowView> {
    let mut views: Vec<ScheduledFreezeWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_freeze_window_view(item));
    }
    views
}

pub fn sum_scheduled_freeze_window_view(view: &ScheduledFreezeWindowView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ceiling + parsed_amount + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_freeze_window("code-797", "label-172");
        let view = to_scheduled_freeze_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_freeze_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
