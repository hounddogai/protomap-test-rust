use super::*;

use acme_kit::text;

/// How screens and exports show a redaction checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointView {
    pub code: String,
    pub label: String,
    pub burst: String,
    pub visible: String,
    pub host: String,
    pub port: String,
    pub maximum: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_redaction_checkpoint_view(item: &RedactionCheckpoint) -> RedactionCheckpointView {
    RedactionCheckpointView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        burst: item.burst.to_string(),
        visible: format!("{}", item.visible),
        host: text::truncate(&item.host, 64),
        port: item.port.to_string(),
        maximum: format!("{:.2}", item.maximum),
        source: text::truncate(&item.source, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_checkpoint_views(items: &[RedactionCheckpoint]) -> Vec<RedactionCheckpointView> {
    let mut views: Vec<RedactionCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_checkpoint_view(item));
    }
    views
}

pub fn sum_redaction_checkpoint_view(view: &RedactionCheckpointView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_port as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_checkpoint("code-205", "label-371");
        let view = to_redaction_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
