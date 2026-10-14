use super::*;

use acme_kit::text;

/// How screens and exports show a primary usage tier.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageTierView {
    pub code_text: String,
    pub title: String,
    pub label: String,
    pub cluster_text: String,
    pub name: String,
    pub summary: String,
    pub window_days: String,
    pub channel: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_primary_usage_tier_view(item: &PrimaryUsageTier) -> PrimaryUsageTierView {
    PrimaryUsageTierView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        label: item.label.clone(),
        cluster_text: item.cluster.clone(),
        name: text::truncate(&item.name, 40),
        summary: item.summary.clone(),
        window_days: item.window_days.to_string(),
        channel: text::truncate(&item.channel, 64),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_usage_tier_views(items: &[PrimaryUsageTier]) -> Vec<PrimaryUsageTierView> {
    let mut views: Vec<PrimaryUsageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_usage_tier_view(item));
    }
    views
}

pub fn sum_primary_usage_tier_view(view: &PrimaryUsageTierView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_usage_tier("code-483", "title-489");
        let view = to_primary_usage_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_usage_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
