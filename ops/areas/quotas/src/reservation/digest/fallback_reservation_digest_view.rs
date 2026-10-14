use super::*;

use acme_kit::text;

/// How screens and exports show a fallback reservation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub locale_text: String,
    pub score: String,
    pub country_text: String,
    pub source: String,
    pub margin: String,
    pub burst_text: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_fallback_reservation_digest_view(item: &FallbackReservationDigest) -> FallbackReservationDigestView {
    FallbackReservationDigestView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        locale_text: text::truncate(&item.locale, 64),
        score: format!("{:.2}", item.score),
        country_text: text::truncate(&item.country, 64),
        source: item.source.clone(),
        margin: format!("{:.2}", item.margin),
        burst_text: item.burst.to_string(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_reservation_digest_views(items: &[FallbackReservationDigest]) -> Vec<FallbackReservationDigestView> {
    let mut views: Vec<FallbackReservationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_reservation_digest_view(item));
    }
    views
}

pub fn sum_fallback_reservation_digest_view(view: &FallbackReservationDigestView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_score + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_reservation_digest("code-172", "name-448");
        let view = to_fallback_reservation_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_reservation_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
