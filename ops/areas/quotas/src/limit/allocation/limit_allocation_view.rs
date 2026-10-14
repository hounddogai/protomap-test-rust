use super::*;

/// How screens and exports show a limit allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationView {
    pub code_text: String,
    pub name_text: String,
    pub duration_minutes: String,
    pub issuer_text: String,
    pub ceiling: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_limit_allocation_view(item: &LimitAllocation) -> LimitAllocationView {
    LimitAllocationView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        issuer_text: item.issuer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_allocation_views(items: &[LimitAllocation]) -> Vec<LimitAllocationView> {
    let mut views: Vec<LimitAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_allocation_view(item));
    }
    views
}

pub fn sum_limit_allocation_view(view: &LimitAllocationView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_allocation("code-879", "name-395");
        let view = to_limit_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
