use super::*;

/// How screens and exports show a redaction window.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionWindowView {
    pub code_text: String,
    pub title: String,
    pub score_text: String,
    pub principal: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_redaction_window_view(item: &RedactionWindow) -> RedactionWindowView {
    RedactionWindowView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        score_text: format!("{:.2}", item.score),
        principal: item.principal.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_window_views(items: &[RedactionWindow]) -> Vec<RedactionWindowView> {
    let mut views: Vec<RedactionWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_window_view(item));
    }
    views
}

pub fn sum_redaction_window_view(view: &RedactionWindowView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_score + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_window("code-628", "title-890");
        let view = to_redaction_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
