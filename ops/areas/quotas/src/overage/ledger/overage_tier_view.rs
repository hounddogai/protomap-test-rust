use super::*;

/// How screens and exports show a overage tier.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageTierView {
    pub code_text: String,
    pub label_text: String,
    pub audience: String,
    pub pinned_text: String,
    pub expires_at_text: String,
    pub amount_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_overage_tier_view(item: &OverageTier) -> OverageTierView {
    OverageTierView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        audience: item.audience.clone(),
        pinned_text: format!("{}", item.pinned),
        expires_at_text: item.expires_at.to_string(),
        amount_text: format!("{:.2}", item.amount),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_tier_views(items: &[OverageTier]) -> Vec<OverageTierView> {
    let mut views: Vec<OverageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_tier_view(item));
    }
    views
}

pub fn sum_overage_tier_view(view: &OverageTierView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_amount + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_tier("code-956", "label-794");
        let view = to_overage_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
