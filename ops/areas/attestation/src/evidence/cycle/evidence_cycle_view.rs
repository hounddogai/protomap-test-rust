use super::*;

/// How screens and exports show a evidence cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycleView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub segment: String,
    pub priority_text: String,
    pub issuer: String,
    pub key_id: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_evidence_cycle_view(item: &EvidenceCycle) -> EvidenceCycleView {
    EvidenceCycleView {
        code: item.code.clone(),
        name: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        segment: item.segment.clone(),
        priority_text: item.priority.to_string(),
        issuer: item.issuer.clone(),
        key_id: item.key_id.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_cycle_views(items: &[EvidenceCycle]) -> Vec<EvidenceCycleView> {
    let mut views: Vec<EvidenceCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_cycle_view(item));
    }
    views
}

pub fn sum_evidence_cycle_view(view: &EvidenceCycleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_priority as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_cycle("code-611", "name-85");
        let view = to_evidence_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
