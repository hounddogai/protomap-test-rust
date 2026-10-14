use super::*;

/// How screens and exports show a strict limit variance.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVarianceView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub name: String,
    pub window_days: String,
    pub notes_text: String,
    pub timezone_text: String,
    pub owner_id: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_strict_limit_variance_view(item: &StrictLimitVariance) -> StrictLimitVarianceView {
    StrictLimitVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        notes_text: item.notes.clone(),
        timezone_text: item.timezone.clone(),
        owner_id: item.owner_id.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_limit_variance_views(items: &[StrictLimitVariance]) -> Vec<StrictLimitVarianceView> {
    let mut views: Vec<StrictLimitVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_limit_variance_view(item));
    }
    views
}

pub fn sum_strict_limit_variance_view(view: &StrictLimitVarianceView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_window_days as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_limit_variance("code-136", "title-999");
        let view = to_strict_limit_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_limit_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
