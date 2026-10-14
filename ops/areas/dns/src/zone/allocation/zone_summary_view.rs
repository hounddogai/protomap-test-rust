use super::*;

/// How screens and exports show a zone summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSummaryView {
    pub code_text: String,
    pub label: String,
    pub fingerprint_text: String,
    pub checksum: String,
    pub window_days_text: String,
    pub archived: String,
    pub source: String,
    pub external_ref_text: String,
    pub issuer: String,
    pub sequence_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_zone_summary_view(item: &ZoneSummary) -> ZoneSummaryView {
    ZoneSummaryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        fingerprint_text: item.fingerprint.clone(),
        checksum: item.checksum.clone(),
        window_days_text: item.window_days.to_string(),
        archived: format!("{}", item.archived),
        source: item.source.clone(),
        external_ref_text: item.external_ref.clone(),
        issuer: item.issuer.clone(),
        sequence_text: item.sequence.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_summary_views(items: &[ZoneSummary]) -> Vec<ZoneSummaryView> {
    let mut views: Vec<ZoneSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_summary_view(item));
    }
    views
}

pub fn sum_zone_summary_view(view: &ZoneSummaryView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_sequence as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_summary("code-75", "label-880");
        let view = to_zone_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
