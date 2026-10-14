use super::*;

use acme_kit::text;

/// How screens and exports show a sink template.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateView {
    pub code: String,
    pub title_text: String,
    pub cluster: String,
    pub ceiling_text: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_sink_template_view(item: &SinkTemplate) -> SinkTemplateView {
    SinkTemplateView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        cluster: item.cluster.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_template_views(items: &[SinkTemplate]) -> Vec<SinkTemplateView> {
    let mut views: Vec<SinkTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_template_view(item));
    }
    views
}

pub fn sum_sink_template_view(view: &SinkTemplateView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ceiling + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_template("code-85", "title-439");
        let view = to_sink_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
