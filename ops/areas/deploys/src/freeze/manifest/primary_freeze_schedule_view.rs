use super::*;

/// How screens and exports show a primary freeze schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFreezeScheduleView {
    pub code: String,
    pub label: String,
    pub ratio_text: String,
    pub source: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_primary_freeze_schedule_view(item: &PrimaryFreezeSchedule) -> PrimaryFreezeScheduleView {
    PrimaryFreezeScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        source: item.source.clone(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_freeze_schedule_views(items: &[PrimaryFreezeSchedule]) -> Vec<PrimaryFreezeScheduleView> {
    let mut views: Vec<PrimaryFreezeScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_freeze_schedule_view(item));
    }
    views
}

pub fn sum_primary_freeze_schedule_view(view: &PrimaryFreezeScheduleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_freeze_schedule("code-983", "label-851");
        let view = to_primary_freeze_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_freeze_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
