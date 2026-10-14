use super::*;

/// How screens and exports show a freeze schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeScheduleView {
    pub code_text: String,
    pub title_text: String,
    pub checksum: String,
    pub summary_text: String,
    pub category: String,
    pub audience: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_freeze_schedule_view(item: &FreezeSchedule) -> FreezeScheduleView {
    FreezeScheduleView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        checksum: item.checksum.clone(),
        summary_text: item.summary.clone(),
        category: item.category.clone(),
        audience: item.audience.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_schedule_views(items: &[FreezeSchedule]) -> Vec<FreezeScheduleView> {
    let mut views: Vec<FreezeScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_schedule_view(item));
    }
    views
}

pub fn sum_freeze_schedule_view(view: &FreezeScheduleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_schedule("code-274", "title-914");
        let view = to_freeze_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
