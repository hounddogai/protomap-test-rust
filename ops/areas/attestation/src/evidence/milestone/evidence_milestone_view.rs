use super::*;

/// How screens and exports show a evidence milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub host: String,
    pub score: String,
    pub threshold: String,
    pub summary_text: String,
    pub contact_email: String,
    pub currency_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_evidence_milestone_view(item: &EvidenceMilestone) -> EvidenceMilestoneView {
    EvidenceMilestoneView {
        code: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        host: item.host.clone(),
        score: format!("{:.2}", item.score),
        threshold: format!("{:.2}", item.threshold),
        summary_text: item.summary.clone(),
        contact_email: item.contact_email.clone(),
        currency_text: item.currency.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_milestone_views(items: &[EvidenceMilestone]) -> Vec<EvidenceMilestoneView> {
    let mut views: Vec<EvidenceMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_milestone_view(item));
    }
    views
}

pub fn sum_evidence_milestone_view(view: &EvidenceMilestoneView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_score + parsed_threshold + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_milestone("code-883", "title-731");
        let view = to_evidence_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
