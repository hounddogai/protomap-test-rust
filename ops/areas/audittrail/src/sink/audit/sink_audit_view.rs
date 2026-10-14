use super::*;

/// How screens and exports show a sink audit.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAuditView {
    pub code_text: String,
    pub label_text: String,
    pub margin_text: String,
    pub floor_value: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_sink_audit_view(item: &SinkAudit) -> SinkAuditView {
    SinkAuditView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        margin_text: format!("{:.2}", item.margin),
        floor_value: format!("{:.2}", item.floor_value),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_audit_views(items: &[SinkAudit]) -> Vec<SinkAuditView> {
    let mut views: Vec<SinkAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_audit_view(item));
    }
    views
}

pub fn sum_sink_audit_view(view: &SinkAuditView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_margin + parsed_floor_value + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_audit("code-562", "label-444");
        let view = to_sink_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
