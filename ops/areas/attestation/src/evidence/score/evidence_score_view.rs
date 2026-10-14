use super::*;

/// How screens and exports show a evidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreView {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub ceiling: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_evidence_score_view(item: &EvidenceScore) -> EvidenceScoreView {
    EvidenceScoreView {
        code: item.code.clone(),
        name: item.name.clone(),
        cipher: item.cipher.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_score_views(items: &[EvidenceScore]) -> Vec<EvidenceScoreView> {
    let mut views: Vec<EvidenceScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_score_view(item));
    }
    views
}

pub fn sum_evidence_score_view(view: &EvidenceScoreView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ceiling + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_score("code-654", "name-159");
        let view = to_evidence_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
