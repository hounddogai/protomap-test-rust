use super::*;

/// How screens and exports show a strict suspension rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRollupView {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub principal: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_strict_suspension_rollup_view(item: &StrictSuspensionRollup) -> StrictSuspensionRollupView {
    StrictSuspensionRollupView {
        code: item.code.clone(),
        title: item.title.clone(),
        scope: item.scope.clone(),
        principal: item.principal.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_suspension_rollup_views(items: &[StrictSuspensionRollup]) -> Vec<StrictSuspensionRollupView> {
    let mut views: Vec<StrictSuspensionRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_suspension_rollup_view(item));
    }
    views
}

pub fn sum_strict_suspension_rollup_view(view: &StrictSuspensionRollupView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_suspension_rollup("code-388", "title-471");
        let view = to_strict_suspension_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_suspension_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
