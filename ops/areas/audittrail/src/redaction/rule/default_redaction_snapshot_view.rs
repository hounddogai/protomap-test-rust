use super::*;

use acme_kit::text;

/// How screens and exports show a default redaction snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotView {
    pub code_text: String,
    pub name_text: String,
    pub active: String,
    pub grace_days_text: String,
    pub summary_text: String,
    pub port: String,
    pub namespace: String,
    pub segment_text: String,
    pub scope_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_default_redaction_snapshot_view(item: &DefaultRedactionSnapshot) -> DefaultRedactionSnapshotView {
    DefaultRedactionSnapshotView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        active: format!("{}", item.active),
        grace_days_text: item.grace_days.to_string(),
        summary_text: item.summary.clone(),
        port: item.port.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        segment_text: item.segment.clone(),
        scope_text: text::truncate(&item.scope, 40),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_redaction_snapshot_views(items: &[DefaultRedactionSnapshot]) -> Vec<DefaultRedactionSnapshotView> {
    let mut views: Vec<DefaultRedactionSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_redaction_snapshot_view(item));
    }
    views
}

pub fn sum_default_redaction_snapshot_view(view: &DefaultRedactionSnapshotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_port as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_redaction_snapshot("code-954", "name-251");
        let view = to_default_redaction_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_redaction_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
