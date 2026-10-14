use super::*;

/// How screens and exports show a default peering record.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringRecordView {
    pub code_text: String,
    pub name: String,
    pub limit: String,
    pub cluster: String,
    pub ceiling: String,
    pub sequence: String,
    pub cipher: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_default_peering_record_view(item: &DefaultPeeringRecord) -> DefaultPeeringRecordView {
    DefaultPeeringRecordView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        limit: item.limit.to_string(),
        cluster: item.cluster.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        sequence: item.sequence.to_string(),
        cipher: item.cipher.clone(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_peering_record_views(items: &[DefaultPeeringRecord]) -> Vec<DefaultPeeringRecordView> {
    let mut views: Vec<DefaultPeeringRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_peering_record_view(item));
    }
    views
}

pub fn sum_default_peering_record_view(view: &DefaultPeeringRecordView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ceiling + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_peering_record("code-25", "name-25");
        let view = to_default_peering_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_peering_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
