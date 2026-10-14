use super::*;

use acme_kit::text;

/// How screens and exports show a tiered peering exception.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringExceptionView {
    pub code_text: String,
    pub label_text: String,
    pub starts_at: String,
    pub approved_text: String,
    pub port_text: String,
    pub contact_email: String,
    pub score_text: String,
    pub country: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_tiered_peering_exception_view(item: &TieredPeeringException) -> TieredPeeringExceptionView {
    TieredPeeringExceptionView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        starts_at: item.starts_at.to_string(),
        approved_text: format!("{}", item.approved),
        port_text: item.port.to_string(),
        contact_email: item.contact_email.clone(),
        score_text: format!("{:.2}", item.score),
        country: item.country.clone(),
        issuer: text::truncate(&item.issuer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_peering_exception_views(items: &[TieredPeeringException]) -> Vec<TieredPeeringExceptionView> {
    let mut views: Vec<TieredPeeringExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_peering_exception_view(item));
    }
    views
}

pub fn sum_tiered_peering_exception_view(view: &TieredPeeringExceptionView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_port as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_peering_exception("code-510", "label-214");
        let view = to_tiered_peering_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_peering_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
