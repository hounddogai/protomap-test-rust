use super::*;

use acme_kit::text;

/// How screens and exports show a lenient reservation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationDigestView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub revision: String,
    pub approved_text: String,
    pub weight: String,
    pub region: String,
    pub version: String,
    pub reviewer: String,
    pub host: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_lenient_reservation_digest_view(item: &LenientReservationDigest) -> LenientReservationDigestView {
    LenientReservationDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        summary: text::truncate(&item.summary, 120),
        revision: item.revision.to_string(),
        approved_text: format!("{}", item.approved),
        weight: format!("{:.2}", item.weight),
        region: text::truncate(&item.region, 120),
        version: item.version.to_string(),
        reviewer: item.reviewer.clone(),
        host: text::truncate(&item.host, 64),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_reservation_digest_views(items: &[LenientReservationDigest]) -> Vec<LenientReservationDigestView> {
    let mut views: Vec<LenientReservationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_reservation_digest_view(item));
    }
    views
}

pub fn sum_lenient_reservation_digest_view(view: &LenientReservationDigestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_revision as f64 + parsed_weight + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_reservation_digest("code-526", "title-728");
        let view = to_lenient_reservation_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_reservation_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
