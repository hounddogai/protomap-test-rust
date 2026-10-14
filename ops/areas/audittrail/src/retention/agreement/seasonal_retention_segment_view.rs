use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal retention segment.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegmentView {
    pub code: String,
    pub title_text: String,
    pub sequence: String,
    pub ttl_seconds: String,
    pub tenant_id_text: String,
    pub cipher_text: String,
    pub source_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_seasonal_retention_segment_view(item: &SeasonalRetentionSegment) -> SeasonalRetentionSegmentView {
    SeasonalRetentionSegmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        sequence: item.sequence.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        cipher_text: text::truncate(&item.cipher, 64),
        source_text: item.source.clone(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_retention_segment_views(items: &[SeasonalRetentionSegment]) -> Vec<SeasonalRetentionSegmentView> {
    let mut views: Vec<SeasonalRetentionSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_retention_segment_view(item));
    }
    views
}

pub fn sum_seasonal_retention_segment_view(view: &SeasonalRetentionSegmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_retention_segment("code-884", "title-290");
        let view = to_seasonal_retention_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_retention_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
