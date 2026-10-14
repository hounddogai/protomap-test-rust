use super::*;

/// How screens and exports show a canary baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBaselineView {
    pub code: String,
    pub title_text: String,
    pub grace_days: String,
    pub active: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_canary_baseline_view(item: &CanaryBaseline) -> CanaryBaselineView {
    CanaryBaselineView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        active: format!("{}", item.active),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_baseline_views(items: &[CanaryBaseline]) -> Vec<CanaryBaselineView> {
    let mut views: Vec<CanaryBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_baseline_view(item));
    }
    views
}

pub fn sum_canary_baseline_view(view: &CanaryBaselineView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_baseline("code-990", "title-858");
        let view = to_canary_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
