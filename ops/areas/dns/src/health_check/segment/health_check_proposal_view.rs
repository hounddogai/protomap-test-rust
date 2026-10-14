use super::*;

use acme_kit::text;

/// How screens and exports show a health check proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckProposalView {
    pub code: String,
    pub label: String,
    pub ratio_text: String,
    pub batch_size: String,
    pub cost_text: String,
    pub host: String,
    pub archived_text: String,
    pub unit: String,
    pub retries: String,
    pub attempts_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_health_check_proposal_view(item: &HealthCheckProposal) -> HealthCheckProposalView {
    HealthCheckProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        batch_size: item.batch_size.to_string(),
        cost_text: format!("{:.2}", item.cost),
        host: item.host.clone(),
        archived_text: format!("{}", item.archived),
        unit: text::truncate(&item.unit, 40),
        retries: item.retries.to_string(),
        attempts_text: item.attempts.to_string(),
        phone: text::truncate(&item.phone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_proposal_views(items: &[HealthCheckProposal]) -> Vec<HealthCheckProposalView> {
    let mut views: Vec<HealthCheckProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_proposal_view(item));
    }
    views
}

pub fn sum_health_check_proposal_view(view: &HealthCheckProposalView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ratio + (parsed_batch_size as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_proposal("code-658", "label-754");
        let view = to_health_check_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
