use super::*;

use acme_kit::text;

/// How screens and exports show a lenient slo target.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTargetView {
    pub code: String,
    pub title_text: String,
    pub margin: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_lenient_slo_target_view(item: &LenientSloTarget) -> LenientSloTargetView {
    LenientSloTargetView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        margin: format!("{:.2}", item.margin),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_slo_target_views(items: &[LenientSloTarget]) -> Vec<LenientSloTargetView> {
    let mut views: Vec<LenientSloTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_slo_target_view(item));
    }
    views
}

pub fn sum_lenient_slo_target_view(view: &LenientSloTargetView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_margin + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_slo_target("code-864", "title-670");
        let view = to_lenient_slo_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_slo_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
