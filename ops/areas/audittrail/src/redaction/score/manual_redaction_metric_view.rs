use super::*;

/// How screens and exports show a manual redaction metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricView {
    pub code_text: String,
    pub title: String,
    pub currency: String,
    pub fingerprint_text: String,
    pub amount: String,
    pub priority: String,
    pub visible: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_manual_redaction_metric_view(item: &ManualRedactionMetric) -> ManualRedactionMetricView {
    ManualRedactionMetricView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        currency: item.currency.clone(),
        fingerprint_text: item.fingerprint.clone(),
        amount: format!("{:.2}", item.amount),
        priority: item.priority.to_string(),
        visible: format!("{}", item.visible),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_redaction_metric_views(items: &[ManualRedactionMetric]) -> Vec<ManualRedactionMetricView> {
    let mut views: Vec<ManualRedactionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_redaction_metric_view(item));
    }
    views
}

pub fn sum_manual_redaction_metric_view(view: &ManualRedactionMetricView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_amount + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_redaction_metric("code-822", "title-22");
        let view = to_manual_redaction_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_redaction_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
