use super::*;

/// How screens and exports show a fallback page variance.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVarianceView {
    pub code_text: String,
    pub name: String,
    pub phone_text: String,
    pub summary: String,
    pub rotation_days: String,
    pub rate_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_fallback_page_variance_view(item: &FallbackPageVariance) -> FallbackPageVarianceView {
    FallbackPageVarianceView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        phone_text: item.phone.clone(),
        summary: item.summary.clone(),
        rotation_days: item.rotation_days.to_string(),
        rate_text: format!("{:.2}", item.rate),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_page_variance_views(items: &[FallbackPageVariance]) -> Vec<FallbackPageVarianceView> {
    let mut views: Vec<FallbackPageVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_page_variance_view(item));
    }
    views
}

pub fn sum_fallback_page_variance_view(view: &FallbackPageVarianceView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_rate + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_page_variance("code-386", "name-115");
        let view = to_fallback_page_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_page_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
