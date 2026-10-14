use super::*;

/// How screens and exports show a delegation record.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRecordView {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub port: String,
    pub description: String,
    pub threshold: String,
    pub unit: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_delegation_record_view(item: &DelegationRecord) -> DelegationRecordView {
    DelegationRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        issuer: item.issuer.clone(),
        port: item.port.to_string(),
        description: item.description.clone(),
        threshold: format!("{:.2}", item.threshold),
        unit: item.unit.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_record_views(items: &[DelegationRecord]) -> Vec<DelegationRecordView> {
    let mut views: Vec<DelegationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_record_view(item));
    }
    views
}

pub fn sum_delegation_record_view(view: &DelegationRecordView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_port as f64 + parsed_threshold + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_record("code-150", "name-860");
        let view = to_delegation_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
