use super::*;

use acme_kit::text;

/// How screens and exports show a lenient action item override.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverrideView {
    pub code: String,
    pub title: String,
    pub pinned_text: String,
    pub cipher: String,
    pub visible: String,
    pub owner_id: String,
    pub phone: String,
    pub label: String,
    pub channel: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_lenient_action_item_override_view(item: &LenientActionItemOverride) -> LenientActionItemOverrideView {
    LenientActionItemOverrideView {
        code: item.code.clone(),
        title: item.title.clone(),
        pinned_text: format!("{}", item.pinned),
        cipher: text::truncate(&item.cipher, 64),
        visible: format!("{}", item.visible),
        owner_id: text::truncate(&item.owner_id, 64),
        phone: item.phone.clone(),
        label: item.label.clone(),
        channel: text::truncate(&item.channel, 80),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_action_item_override_views(
    items: &[LenientActionItemOverride],
) -> Vec<LenientActionItemOverrideView> {
    let mut views: Vec<LenientActionItemOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_action_item_override_view(item));
    }
    views
}

pub fn sum_lenient_action_item_override_view(view: &LenientActionItemOverrideView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
