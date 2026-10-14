use super::*;

use acme_kit::text;

/// How screens and exports show a silence tier.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTierView {
    pub code: String,
    pub name_text: String,
    pub channel: String,
    pub floor_value: String,
    pub unit_text: String,
    pub audience: String,
    pub key_id: String,
    pub discount: String,
    pub external_ref: String,
    pub pinned: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_silence_tier_view(item: &SilenceTier) -> SilenceTierView {
    SilenceTierView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        channel: item.channel.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        unit_text: text::truncate(&item.unit, 120),
        audience: item.audience.clone(),
        key_id: item.key_id.clone(),
        discount: format!("{:.2}", item.discount),
        external_ref: text::truncate(&item.external_ref, 64),
        pinned: format!("{}", item.pinned),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_tier_views(items: &[SilenceTier]) -> Vec<SilenceTierView> {
    let mut views: Vec<SilenceTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_tier_view(item));
    }
    views
}

pub fn sum_silence_tier_view(view: &SilenceTierView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_floor_value + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_tier("code-190", "name-260");
        let view = to_silence_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
