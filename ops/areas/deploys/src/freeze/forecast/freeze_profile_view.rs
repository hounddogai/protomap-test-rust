use super::*;

use acme_kit::text;

/// How screens and exports show a freeze profile.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProfileView {
    pub code: String,
    pub name_text: String,
    pub window_days_text: String,
    pub amount: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_freeze_profile_view(item: &FreezeProfile) -> FreezeProfileView {
    FreezeProfileView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        window_days_text: item.window_days.to_string(),
        amount: format!("{:.2}", item.amount),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_profile_views(items: &[FreezeProfile]) -> Vec<FreezeProfileView> {
    let mut views: Vec<FreezeProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_profile_view(item));
    }
    views
}

pub fn sum_freeze_profile_view(view: &FreezeProfileView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_amount + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_profile("code-728", "name-445");
        let view = to_freeze_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
