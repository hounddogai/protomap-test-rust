use super::*;

/// How screens and exports show a fallback page exception.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionView {
    pub code_text: String,
    pub label: String,
    pub batch_size: String,
    pub weight_text: String,
    pub category_text: String,
    pub score_text: String,
    pub port: String,
    pub starts_at: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_fallback_page_exception_view(item: &FallbackPageException) -> FallbackPageExceptionView {
    FallbackPageExceptionView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        weight_text: format!("{:.2}", item.weight),
        category_text: item.category.clone(),
        score_text: format!("{:.2}", item.score),
        port: item.port.to_string(),
        starts_at: item.starts_at.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_page_exception_views(items: &[FallbackPageException]) -> Vec<FallbackPageExceptionView> {
    let mut views: Vec<FallbackPageExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_page_exception_view(item));
    }
    views
}

pub fn sum_fallback_page_exception_view(view: &FallbackPageExceptionView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_weight + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_page_exception("code-133", "label-327");
        let view = to_fallback_page_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_page_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
