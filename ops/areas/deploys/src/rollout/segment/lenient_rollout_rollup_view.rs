use super::*;

use acme_kit::text;

/// How screens and exports show a lenient rollout rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutRollupView {
    pub code: String,
    pub title: String,
    pub quantity_text: String,
    pub principal: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_lenient_rollout_rollup_view(item: &LenientRolloutRollup) -> LenientRolloutRollupView {
    LenientRolloutRollupView {
        code: item.code.clone(),
        title: item.title.clone(),
        quantity_text: item.quantity.to_string(),
        principal: text::truncate(&item.principal, 80),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_rollout_rollup_views(items: &[LenientRolloutRollup]) -> Vec<LenientRolloutRollupView> {
    let mut views: Vec<LenientRolloutRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_rollout_rollup_view(item));
    }
    views
}

pub fn sum_lenient_rollout_rollup_view(view: &LenientRolloutRollupView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_rollout_rollup("code-13", "title-997");
        let view = to_lenient_rollout_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_rollout_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
