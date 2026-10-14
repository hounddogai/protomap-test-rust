use super::*;

/// How screens and exports show a scheduled slo record.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub external_ref_text: String,
    pub issuer: String,
    pub rate: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_scheduled_slo_record_view(item: &ScheduledSloRecord) -> ScheduledSloRecordView {
    ScheduledSloRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        external_ref_text: item.external_ref.clone(),
        issuer: item.issuer.clone(),
        rate: format!("{:.2}", item.rate),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_slo_record_views(items: &[ScheduledSloRecord]) -> Vec<ScheduledSloRecordView> {
    let mut views: Vec<ScheduledSloRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_slo_record_view(item));
    }
    views
}

pub fn sum_scheduled_slo_record_view(view: &ScheduledSloRecordView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_weight + parsed_rate + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_slo_record("code-684", "name-490");
        let view = to_scheduled_slo_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_slo_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
