use super::*;

/// How screens and exports show a primary evidence entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntryView {
    pub code_text: String,
    pub label_text: String,
    pub cluster: String,
    pub retries: String,
    pub burst: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_primary_evidence_entry_view(item: &PrimaryEvidenceEntry) -> PrimaryEvidenceEntryView {
    PrimaryEvidenceEntryView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        cluster: item.cluster.clone(),
        retries: item.retries.to_string(),
        burst: item.burst.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_evidence_entry_views(items: &[PrimaryEvidenceEntry]) -> Vec<PrimaryEvidenceEntryView> {
    let mut views: Vec<PrimaryEvidenceEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_evidence_entry_view(item));
    }
    views
}

pub fn sum_primary_evidence_entry_view(view: &PrimaryEvidenceEntryView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_burst as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_evidence_entry("code-365", "label-748");
        let view = to_primary_evidence_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_evidence_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
