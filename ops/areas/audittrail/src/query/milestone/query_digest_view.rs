use super::*;

use acme_kit::text;

/// How screens and exports show a query digest.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryDigestView {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub maximum: String,
    pub duration_minutes_text: String,
    pub scope: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_query_digest_view(item: &QueryDigest) -> QueryDigestView {
    QueryDigestView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        timezone: item.timezone.clone(),
        maximum: format!("{:.2}", item.maximum),
        duration_minutes_text: item.duration_minutes.to_string(),
        scope: text::truncate(&item.scope, 40),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_digest_views(items: &[QueryDigest]) -> Vec<QueryDigestView> {
    let mut views: Vec<QueryDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_digest_view(item));
    }
    views
}

pub fn sum_query_digest_view(view: &QueryDigestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_maximum + (parsed_duration_minutes as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_digest("code-20", "title-731");
        let view = to_query_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
