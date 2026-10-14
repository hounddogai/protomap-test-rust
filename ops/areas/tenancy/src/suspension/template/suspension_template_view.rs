use super::*;

use acme_kit::text;

/// How screens and exports show a suspension template.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateView {
    pub code: String,
    pub title: String,
    pub expires_at: String,
    pub capacity_text: String,
    pub notes: String,
    pub revision: String,
    pub source: String,
    pub reviewer: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_suspension_template_view(item: &SuspensionTemplate) -> SuspensionTemplateView {
    SuspensionTemplateView {
        code: item.code.clone(),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        capacity_text: item.capacity.to_string(),
        notes: item.notes.clone(),
        revision: item.revision.to_string(),
        source: text::truncate(&item.source, 40),
        reviewer: item.reviewer.clone(),
        issuer: text::truncate(&item.issuer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_template_views(items: &[SuspensionTemplate]) -> Vec<SuspensionTemplateView> {
    let mut views: Vec<SuspensionTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_template_view(item));
    }
    views
}

pub fn sum_suspension_template_view(view: &SuspensionTemplateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_capacity as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_template("code-171", "title-793");
        let view = to_suspension_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
