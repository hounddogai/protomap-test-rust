use super::*;

use acme_kit::text;

/// How screens and exports show a runbook template.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateView {
    pub code_text: String,
    pub name: String,
    pub unit: String,
    pub margin: String,
    pub checksum: String,
    pub priority: String,
    pub reviewer: String,
    pub title: String,
    pub external_ref: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_runbook_template_view(item: &RunbookTemplate) -> RunbookTemplateView {
    RunbookTemplateView {
        code_text: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        unit: item.unit.clone(),
        margin: format!("{:.2}", item.margin),
        checksum: item.checksum.clone(),
        priority: item.priority.to_string(),
        reviewer: item.reviewer.clone(),
        title: item.title.clone(),
        external_ref: item.external_ref.clone(),
        country: text::truncate(&item.country, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_template_views(items: &[RunbookTemplate]) -> Vec<RunbookTemplateView> {
    let mut views: Vec<RunbookTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_template_view(item));
    }
    views
}

pub fn sum_runbook_template_view(view: &RunbookTemplateView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_margin + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_template("code-956", "name-657");
        let view = to_runbook_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
