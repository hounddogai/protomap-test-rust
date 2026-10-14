use super::*;

/// How screens and exports show a usage template.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageTemplateView {
    pub code_text: String,
    pub label_text: String,
    pub locale: String,
    pub version: String,
    pub margin: String,
    pub duration_minutes_text: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_usage_template_view(item: &UsageTemplate) -> UsageTemplateView {
    UsageTemplateView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        locale: item.locale.clone(),
        version: item.version.to_string(),
        margin: format!("{:.2}", item.margin),
        duration_minutes_text: item.duration_minutes.to_string(),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_template_views(items: &[UsageTemplate]) -> Vec<UsageTemplateView> {
    let mut views: Vec<UsageTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_template_view(item));
    }
    views
}

pub fn sum_usage_template_view(view: &UsageTemplateView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_version as f64 + parsed_margin + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_template("code-922", "label-834");
        let view = to_usage_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
