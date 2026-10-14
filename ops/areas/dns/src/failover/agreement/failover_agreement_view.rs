use super::*;

/// How screens and exports show a failover agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAgreementView {
    pub code: String,
    pub label_text: String,
    pub version: String,
    pub key_id: String,
    pub rotation_days: String,
    pub title_text: String,
    pub notes: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_failover_agreement_view(item: &FailoverAgreement) -> FailoverAgreementView {
    FailoverAgreementView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        version: item.version.to_string(),
        key_id: item.key_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        title_text: item.title.clone(),
        notes: item.notes.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_agreement_views(items: &[FailoverAgreement]) -> Vec<FailoverAgreementView> {
    let mut views: Vec<FailoverAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_agreement_view(item));
    }
    views
}

pub fn sum_failover_agreement_view(view: &FailoverAgreementView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_version as f64 + (parsed_rotation_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_agreement("code-215", "label-798");
        let view = to_failover_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
