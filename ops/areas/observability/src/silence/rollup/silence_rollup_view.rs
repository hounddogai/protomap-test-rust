use super::*;

/// How screens and exports show a silence rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceRollupView {
    pub code: String,
    pub name_text: String,
    pub description_text: String,
    pub limit: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_silence_rollup_view(item: &SilenceRollup) -> SilenceRollupView {
    SilenceRollupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        description_text: item.description.clone(),
        limit: item.limit.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_rollup_views(items: &[SilenceRollup]) -> Vec<SilenceRollupView> {
    let mut views: Vec<SilenceRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_rollup_view(item));
    }
    views
}

pub fn sum_silence_rollup_view(view: &SilenceRollupView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_rollup("code-642", "name-711");
        let view = to_silence_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
