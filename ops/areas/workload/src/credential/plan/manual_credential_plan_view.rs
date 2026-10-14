use super::*;

/// How screens and exports show a manual credential plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialPlanView {
    pub code: String,
    pub label: String,
    pub duration_minutes: String,
    pub source_text: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_manual_credential_plan_view(item: &ManualCredentialPlan) -> ManualCredentialPlanView {
    ManualCredentialPlanView {
        code: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        source_text: item.source.clone(),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_credential_plan_views(items: &[ManualCredentialPlan]) -> Vec<ManualCredentialPlanView> {
    let mut views: Vec<ManualCredentialPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_credential_plan_view(item));
    }
    views
}

pub fn sum_manual_credential_plan_view(view: &ManualCredentialPlanView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_credential_plan("code-441", "label-307");
        let view = to_manual_credential_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_credential_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
