use super::*;

/// How screens and exports show a seasonal alert entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertEntryView {
    pub code: String,
    pub name_text: String,
    pub limit: String,
    pub cluster: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_seasonal_alert_entry_view(item: &SeasonalAlertEntry) -> SeasonalAlertEntryView {
    SeasonalAlertEntryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        limit: item.limit.to_string(),
        cluster: item.cluster.clone(),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_alert_entry_views(items: &[SeasonalAlertEntry]) -> Vec<SeasonalAlertEntryView> {
    let mut views: Vec<SeasonalAlertEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_alert_entry_view(item));
    }
    views
}

pub fn sum_seasonal_alert_entry_view(view: &SeasonalAlertEntryView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_alert_entry("code-790", "name-581");
        let view = to_seasonal_alert_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_alert_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
