use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard digest.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigestView {
    pub code: String,
    pub label_text: String,
    pub locale_text: String,
    pub region: String,
    pub timezone: String,
    pub archived: String,
    pub batch_size_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_dashboard_digest_view(item: &DashboardDigest) -> DashboardDigestView {
    DashboardDigestView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        locale_text: item.locale.clone(),
        region: text::truncate(&item.region, 80),
        timezone: item.timezone.clone(),
        archived: format!("{}", item.archived),
        batch_size_text: item.batch_size.to_string(),
        currency: text::truncate(&item.currency, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_digest_views(items: &[DashboardDigest]) -> Vec<DashboardDigestView> {
    let mut views: Vec<DashboardDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_digest_view(item));
    }
    views
}

pub fn sum_dashboard_digest_view(view: &DashboardDigestView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_digest("code-458", "label-951");
        let view = to_dashboard_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
