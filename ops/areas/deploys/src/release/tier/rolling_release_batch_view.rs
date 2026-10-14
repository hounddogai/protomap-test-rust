use super::*;

use acme_kit::text;

/// How screens and exports show a rolling release batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseBatchView {
    pub code_text: String,
    pub label: String,
    pub sequence_text: String,
    pub audience: String,
    pub fingerprint: String,
    pub weight: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_rolling_release_batch_view(item: &RollingReleaseBatch) -> RollingReleaseBatchView {
    RollingReleaseBatchView {
        code_text: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 40),
        sequence_text: item.sequence.to_string(),
        audience: item.audience.clone(),
        fingerprint: item.fingerprint.clone(),
        weight: format!("{:.2}", item.weight),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_release_batch_views(items: &[RollingReleaseBatch]) -> Vec<RollingReleaseBatchView> {
    let mut views: Vec<RollingReleaseBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_release_batch_view(item));
    }
    views
}

pub fn sum_rolling_release_batch_view(view: &RollingReleaseBatchView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_weight + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_release_batch("code-584", "label-967");
        let view = to_rolling_release_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_release_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
