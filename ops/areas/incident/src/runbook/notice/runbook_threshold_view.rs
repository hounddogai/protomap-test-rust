use super::*;

/// How screens and exports show a runbook threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThresholdView {
    pub code_text: String,
    pub name: String,
    pub expires_at_text: String,
    pub capacity: String,
    pub host_text: String,
    pub maximum: String,
    pub pinned: String,
    pub ends_at_text: String,
    pub external_ref_text: String,
    pub locale: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_runbook_threshold_view(item: &RunbookThreshold) -> RunbookThresholdView {
    RunbookThresholdView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        expires_at_text: item.expires_at.to_string(),
        capacity: item.capacity.to_string(),
        host_text: item.host.clone(),
        maximum: format!("{:.2}", item.maximum),
        pinned: format!("{}", item.pinned),
        ends_at_text: item.ends_at.to_string(),
        external_ref_text: item.external_ref.clone(),
        locale: item.locale.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_threshold_views(items: &[RunbookThreshold]) -> Vec<RunbookThresholdView> {
    let mut views: Vec<RunbookThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_threshold_view(item));
    }
    views
}

pub fn sum_runbook_threshold_view(view: &RunbookThresholdView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_capacity as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_threshold("code-102", "name-649");
        let view = to_runbook_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
