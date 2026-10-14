use super::*;

/// How screens and exports show a tiered dashboard digest.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigestView {
    pub code_text: String,
    pub title: String,
    pub channel: String,
    pub maximum: String,
    pub score: String,
    pub grace_days: String,
    pub checksum: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_tiered_dashboard_digest_view(item: &TieredDashboardDigest) -> TieredDashboardDigestView {
    TieredDashboardDigestView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        channel: item.channel.clone(),
        maximum: format!("{:.2}", item.maximum),
        score: format!("{:.2}", item.score),
        grace_days: item.grace_days.to_string(),
        checksum: item.checksum.clone(),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_dashboard_digest_views(items: &[TieredDashboardDigest]) -> Vec<TieredDashboardDigestView> {
    let mut views: Vec<TieredDashboardDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_dashboard_digest_view(item));
    }
    views
}

pub fn sum_tiered_dashboard_digest_view(view: &TieredDashboardDigestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_maximum + parsed_score + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_dashboard_digest("code-994", "title-58");
        let view = to_tiered_dashboard_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_dashboard_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
