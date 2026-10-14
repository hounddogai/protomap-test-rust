use super::*;

use acme_kit::text;

/// How screens and exports show a capped freeze exception.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeExceptionView {
    pub code_text: String,
    pub label_text: String,
    pub description: String,
    pub external_ref_text: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_capped_freeze_exception_view(item: &CappedFreezeException) -> CappedFreezeExceptionView {
    CappedFreezeExceptionView {
        code_text: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        description: text::truncate(&item.description, 80),
        external_ref_text: item.external_ref.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_freeze_exception_views(items: &[CappedFreezeException]) -> Vec<CappedFreezeExceptionView> {
    let mut views: Vec<CappedFreezeExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_freeze_exception_view(item));
    }
    views
}

pub fn sum_capped_freeze_exception_view(view: &CappedFreezeExceptionView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_freeze_exception("code-911", "label-599");
        let view = to_capped_freeze_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_freeze_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
