use super::*;

/// How screens and exports show a legacy silence allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocationView {
    pub code_text: String,
    pub title: String,
    pub threshold_text: String,
    pub ttl_seconds_text: String,
    pub revision_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_legacy_silence_allocation_view(item: &LegacySilenceAllocation) -> LegacySilenceAllocationView {
    LegacySilenceAllocationView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        revision_text: item.revision.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_silence_allocation_views(items: &[LegacySilenceAllocation]) -> Vec<LegacySilenceAllocationView> {
    let mut views: Vec<LegacySilenceAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_silence_allocation_view(item));
    }
    views
}

pub fn sum_legacy_silence_allocation_view(view: &LegacySilenceAllocationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_threshold + (parsed_ttl_seconds as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_silence_allocation("code-541", "title-153");
        let view = to_legacy_silence_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_silence_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
