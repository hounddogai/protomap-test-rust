use super::*;

/// How screens and exports show a federation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationScheduleView {
    pub code: String,
    pub label_text: String,
    pub namespace: String,
    pub issuer: String,
    pub title: String,
    pub batch_size: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_federation_schedule_view(item: &FederationSchedule) -> FederationScheduleView {
    FederationScheduleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        namespace: item.namespace.clone(),
        issuer: item.issuer.clone(),
        title: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_schedule_views(items: &[FederationSchedule]) -> Vec<FederationScheduleView> {
    let mut views: Vec<FederationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_schedule_view(item));
    }
    views
}

pub fn sum_federation_schedule_view(view: &FederationScheduleView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_schedule("code-418", "label-841");
        let view = to_federation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
