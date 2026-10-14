use super::*;

use acme_kit::text;

/// How screens and exports show a retention rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollupView {
    pub code: String,
    pub title_text: String,
    pub ttl_seconds_text: String,
    pub timezone: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_retention_rollup_view(item: &RetentionRollup) -> RetentionRollupView {
    RetentionRollupView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 120),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        timezone: item.timezone.clone(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_rollup_views(items: &[RetentionRollup]) -> Vec<RetentionRollupView> {
    let mut views: Vec<RetentionRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_rollup_view(item));
    }
    views
}

pub fn sum_retention_rollup_view(view: &RetentionRollupView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_rollup("code-557", "title-941");
        let view = to_retention_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
