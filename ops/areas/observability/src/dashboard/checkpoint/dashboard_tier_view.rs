use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard tier.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTierView {
    pub code: String,
    pub title: String,
    pub channel_text: String,
    pub label: String,
    pub fingerprint: String,
    pub approved: String,
    pub window_days: String,
    pub currency: String,
    pub key_id_text: String,
    pub rate_text: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_dashboard_tier_view(item: &DashboardTier) -> DashboardTierView {
    DashboardTierView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        channel_text: text::truncate(&item.channel, 64),
        label: text::truncate(&item.label, 80),
        fingerprint: text::truncate(&item.fingerprint, 40),
        approved: format!("{}", item.approved),
        window_days: item.window_days.to_string(),
        currency: item.currency.clone(),
        key_id_text: text::truncate(&item.key_id, 40),
        rate_text: format!("{:.2}", item.rate),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_tier_views(items: &[DashboardTier]) -> Vec<DashboardTierView> {
    let mut views: Vec<DashboardTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_tier_view(item));
    }
    views
}

pub fn sum_dashboard_tier_view(view: &DashboardTierView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_tier("code-42", "title-672");
        let view = to_dashboard_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
