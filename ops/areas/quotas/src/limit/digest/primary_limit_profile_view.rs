use super::*;

use acme_kit::text;

/// How screens and exports show a primary limit profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub cluster: String,
    pub rate: String,
    pub batch_size: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_primary_limit_profile_view(item: &PrimaryLimitProfile) -> PrimaryLimitProfileView {
    PrimaryLimitProfileView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        cluster: item.cluster.clone(),
        rate: format!("{:.2}", item.rate),
        batch_size: item.batch_size.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_limit_profile_views(items: &[PrimaryLimitProfile]) -> Vec<PrimaryLimitProfileView> {
    let mut views: Vec<PrimaryLimitProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_limit_profile_view(item));
    }
    views
}

pub fn sum_primary_limit_profile_view(view: &PrimaryLimitProfileView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_rate + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_limit_profile("code-385", "title-384");
        let view = to_primary_limit_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_limit_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
