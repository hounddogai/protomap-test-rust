use super::*;

/// How screens and exports show a capped rollout milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutMilestoneView {
    pub code: String,
    pub label: String,
    pub issuer_text: String,
    pub updated_at: String,
    pub country: String,
    pub amount: String,
    pub duration_minutes_text: String,
    pub cluster: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_capped_rollout_milestone_view(item: &CappedRolloutMilestone) -> CappedRolloutMilestoneView {
    CappedRolloutMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer_text: item.issuer.clone(),
        updated_at: item.updated_at.to_string(),
        country: item.country.clone(),
        amount: format!("{:.2}", item.amount),
        duration_minutes_text: item.duration_minutes.to_string(),
        cluster: item.cluster.clone(),
        tenant_id: item.tenant_id.clone(),
        contact_email: item.contact_email.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_rollout_milestone_views(items: &[CappedRolloutMilestone]) -> Vec<CappedRolloutMilestoneView> {
    let mut views: Vec<CappedRolloutMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_rollout_milestone_view(item));
    }
    views
}

pub fn sum_capped_rollout_milestone_view(view: &CappedRolloutMilestoneView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_amount + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_rollout_milestone("code-825", "label-318");
        let view = to_capped_rollout_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_rollout_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
