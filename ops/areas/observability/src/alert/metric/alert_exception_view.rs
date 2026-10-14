use super::*;

/// How screens and exports show a alert exception.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertExceptionView {
    pub code_text: String,
    pub label_text: String,
    pub external_ref: String,
    pub rotation_days: String,
    pub host: String,
    pub limit_text: String,
    pub issuer: String,
    pub expires_at: String,
    pub ends_at: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_alert_exception_view(item: &AlertException) -> AlertExceptionView {
    AlertExceptionView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        external_ref: item.external_ref.clone(),
        rotation_days: item.rotation_days.to_string(),
        host: item.host.clone(),
        limit_text: item.limit.to_string(),
        issuer: item.issuer.clone(),
        expires_at: item.expires_at.to_string(),
        ends_at: item.ends_at.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_exception_views(items: &[AlertException]) -> Vec<AlertExceptionView> {
    let mut views: Vec<AlertExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_exception_view(item));
    }
    views
}

pub fn sum_alert_exception_view(view: &AlertExceptionView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_exception("code-879", "label-816");
        let view = to_alert_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
