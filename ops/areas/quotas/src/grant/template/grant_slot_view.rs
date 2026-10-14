use super::*;

use acme_kit::text;

/// How screens and exports show a grant slot.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSlotView {
    pub code: String,
    pub name: String,
    pub timezone_text: String,
    pub score: String,
    pub threshold: String,
    pub issuer: String,
    pub version_text: String,
    pub capacity_text: String,
    pub floor_value: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_grant_slot_view(item: &GrantSlot) -> GrantSlotView {
    GrantSlotView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 80),
        timezone_text: text::truncate(&item.timezone, 120),
        score: format!("{:.2}", item.score),
        threshold: format!("{:.2}", item.threshold),
        issuer: text::truncate(&item.issuer, 120),
        version_text: item.version.to_string(),
        capacity_text: item.capacity.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_slot_views(items: &[GrantSlot]) -> Vec<GrantSlotView> {
    let mut views: Vec<GrantSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_slot_view(item));
    }
    views
}

pub fn sum_grant_slot_view(view: &GrantSlotView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_score + parsed_threshold + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_slot("code-710", "name-367");
        let view = to_grant_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
