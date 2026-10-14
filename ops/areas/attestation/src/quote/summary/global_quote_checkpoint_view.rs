use super::*;

use acme_kit::text;

/// How screens and exports show a global quote checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointView {
    pub code_text: String,
    pub name_text: String,
    pub label_text: String,
    pub locale: String,
    pub margin: String,
    pub active_text: String,
    pub version: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_global_quote_checkpoint_view(item: &GlobalQuoteCheckpoint) -> GlobalQuoteCheckpointView {
    GlobalQuoteCheckpointView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        label_text: item.label.clone(),
        locale: item.locale.clone(),
        margin: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        version: item.version.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_quote_checkpoint_views(items: &[GlobalQuoteCheckpoint]) -> Vec<GlobalQuoteCheckpointView> {
    let mut views: Vec<GlobalQuoteCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_quote_checkpoint_view(item));
    }
    views
}

pub fn sum_global_quote_checkpoint_view(view: &GlobalQuoteCheckpointView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_margin + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_quote_checkpoint("code-955", "name-208");
        let view = to_global_quote_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_quote_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
