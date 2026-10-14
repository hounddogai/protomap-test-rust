use super::*;

/// How screens and exports show a legacy isolation template.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationTemplateView {
    pub code_text: String,
    pub title: String,
    pub contact_email: String,
    pub floor_value: String,
    pub summary: String,
    pub key_id: String,
    pub ceiling_text: String,
    pub currency: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_legacy_isolation_template_view(item: &LegacyIsolationTemplate) -> LegacyIsolationTemplateView {
    LegacyIsolationTemplateView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        contact_email: item.contact_email.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        summary: item.summary.clone(),
        key_id: item.key_id.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        currency: item.currency.clone(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_isolation_template_views(items: &[LegacyIsolationTemplate]) -> Vec<LegacyIsolationTemplateView> {
    let mut views: Vec<LegacyIsolationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_isolation_template_view(item));
    }
    views
}

pub fn sum_legacy_isolation_template_view(view: &LegacyIsolationTemplateView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_floor_value + parsed_ceiling + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_isolation_template("code-114", "title-834");
        let view = to_legacy_isolation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_isolation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
