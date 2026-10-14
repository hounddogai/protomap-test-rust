use super::*;

/// How screens and exports show a fallback binding adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustmentView {
    pub code: String,
    pub name_text: String,
    pub score: String,
    pub checksum: String,
    pub burst_text: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_fallback_binding_adjustment_view(item: &FallbackBindingAdjustment) -> FallbackBindingAdjustmentView {
    FallbackBindingAdjustmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        score: format!("{:.2}", item.score),
        checksum: item.checksum.clone(),
        burst_text: item.burst.to_string(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_binding_adjustment_views(items: &[FallbackBindingAdjustment]) -> Vec<FallbackBindingAdjustmentView> {
    let mut views: Vec<FallbackBindingAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_binding_adjustment_view(item));
    }
    views
}

pub fn sum_fallback_binding_adjustment_view(view: &FallbackBindingAdjustmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_score + (parsed_burst as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_binding_adjustment("code-825", "name-51");
        let view = to_fallback_binding_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_binding_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
