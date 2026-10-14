use super::*;

use acme_kit::text;

/// How screens and exports show a sink threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkThresholdView {
    pub code_text: String,
    pub label: String,
    pub port_text: String,
    pub active: String,
    pub cipher_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_sink_threshold_view(item: &SinkThreshold) -> SinkThresholdView {
    SinkThresholdView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        port_text: item.port.to_string(),
        active: format!("{}", item.active),
        cipher_text: item.cipher.clone(),
        checksum_text: text::truncate(&item.checksum, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_threshold_views(items: &[SinkThreshold]) -> Vec<SinkThresholdView> {
    let mut views: Vec<SinkThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_threshold_view(item));
    }
    views
}

pub fn sum_sink_threshold_view(view: &SinkThresholdView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_threshold("code-986", "label-714");
        let view = to_sink_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
