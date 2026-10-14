use super::*;

use acme_kit::text;

/// How screens and exports show a silence batch.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBatchView {
    pub code: String,
    pub label_text: String,
    pub port_text: String,
    pub category: String,
    pub limit: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_silence_batch_view(item: &SilenceBatch) -> SilenceBatchView {
    SilenceBatchView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        port_text: item.port.to_string(),
        category: text::truncate(&item.category, 120),
        limit: item.limit.to_string(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_batch_views(items: &[SilenceBatch]) -> Vec<SilenceBatchView> {
    let mut views: Vec<SilenceBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_batch_view(item));
    }
    views
}

pub fn sum_silence_batch_view(view: &SilenceBatchView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_port as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_batch("code-808", "label-773");
        let view = to_silence_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
