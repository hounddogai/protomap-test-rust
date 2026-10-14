use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal freeze window.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindowView {
    pub code: String,
    pub name_text: String,
    pub contact_email_text: String,
    pub host_text: String,
    pub audience: String,
    pub rotation_days_text: String,
    pub namespace: String,
    pub checksum: String,
    pub amount: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_seasonal_freeze_window_view(item: &SeasonalFreezeWindow) -> SeasonalFreezeWindowView {
    SeasonalFreezeWindowView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        contact_email_text: item.contact_email.clone(),
        host_text: item.host.clone(),
        audience: text::truncate(&item.audience, 80),
        rotation_days_text: item.rotation_days.to_string(),
        namespace: item.namespace.clone(),
        checksum: text::truncate(&item.checksum, 40),
        amount: format!("{:.2}", item.amount),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_freeze_window_views(items: &[SeasonalFreezeWindow]) -> Vec<SeasonalFreezeWindowView> {
    let mut views: Vec<SeasonalFreezeWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_freeze_window_view(item));
    }
    views
}

pub fn sum_seasonal_freeze_window_view(view: &SeasonalFreezeWindowView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_freeze_window("code-100", "name-703");
        let view = to_seasonal_freeze_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_freeze_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
