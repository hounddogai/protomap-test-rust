use super::*;

use acme_kit::text;

/// How screens and exports show a tiered allowlist draft.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistDraftView {
    pub code_text: String,
    pub name: String,
    pub ratio: String,
    pub active_text: String,
    pub channel: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_tiered_allowlist_draft_view(item: &TieredAllowlistDraft) -> TieredAllowlistDraftView {
    TieredAllowlistDraftView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        ratio: format!("{:.2}", item.ratio),
        active_text: format!("{}", item.active),
        channel: item.channel.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_allowlist_draft_views(items: &[TieredAllowlistDraft]) -> Vec<TieredAllowlistDraftView> {
    let mut views: Vec<TieredAllowlistDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_allowlist_draft_view(item));
    }
    views
}

pub fn sum_tiered_allowlist_draft_view(view: &TieredAllowlistDraftView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ratio + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_allowlist_draft("code-173", "name-912");
        let view = to_tiered_allowlist_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_allowlist_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
