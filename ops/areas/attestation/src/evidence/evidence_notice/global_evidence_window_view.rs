use super::*;

/// How screens and exports show a global evidence window.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub updated_at_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_global_evidence_window_view(item: &GlobalEvidenceWindow) -> GlobalEvidenceWindowView {
    GlobalEvidenceWindowView {
        code: item.code.clone(),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        updated_at_text: item.updated_at.to_string(),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_evidence_window_views(items: &[GlobalEvidenceWindow]) -> Vec<GlobalEvidenceWindowView> {
    let mut views: Vec<GlobalEvidenceWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_evidence_window_view(item));
    }
    views
}

pub fn sum_global_evidence_window_view(view: &GlobalEvidenceWindowView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_budget + (parsed_updated_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_evidence_window("code-443", "title-583");
        let view = to_global_evidence_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_evidence_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
