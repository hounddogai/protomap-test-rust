use super::*;

use acme_kit::text;

/// How screens and exports show a zone tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTierView {
    pub code_text: String,
    pub label_text: String,
    pub port: String,
    pub country: String,
    pub audience: String,
    pub priority: String,
    pub revision: String,
    pub segment_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_zone_tier_view(item: &ZoneTier) -> ZoneTierView {
    ZoneTierView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        port: item.port.to_string(),
        country: item.country.clone(),
        audience: text::truncate(&item.audience, 40),
        priority: item.priority.to_string(),
        revision: item.revision.to_string(),
        segment_text: item.segment.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_tier_views(items: &[ZoneTier]) -> Vec<ZoneTierView> {
    let mut views: Vec<ZoneTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_tier_view(item));
    }
    views
}

pub fn sum_zone_tier_view(view: &ZoneTierView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_port as f64 + (parsed_priority as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_tier("code-859", "label-96");
        let view = to_zone_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
