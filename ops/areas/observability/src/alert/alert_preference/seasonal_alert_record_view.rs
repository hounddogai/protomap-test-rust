use super::*;

/// How screens and exports show a seasonal alert record.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertRecordView {
    pub code: String,
    pub name_text: String,
    pub budget: String,
    pub limit_text: String,
    pub host: String,
    pub maximum: String,
    pub batch_size: String,
    pub contact_email: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_seasonal_alert_record_view(item: &SeasonalAlertRecord) -> SeasonalAlertRecordView {
    SeasonalAlertRecordView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        limit_text: item.limit.to_string(),
        host: item.host.clone(),
        maximum: format!("{:.2}", item.maximum),
        batch_size: item.batch_size.to_string(),
        contact_email: item.contact_email.clone(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_alert_record_views(items: &[SeasonalAlertRecord]) -> Vec<SeasonalAlertRecordView> {
    let mut views: Vec<SeasonalAlertRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_alert_record_view(item));
    }
    views
}

pub fn sum_seasonal_alert_record_view(view: &SeasonalAlertRecordView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_budget + (parsed_limit as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_alert_record("code-571", "name-98");
        let view = to_seasonal_alert_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_alert_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
