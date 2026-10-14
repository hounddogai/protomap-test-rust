use super::*;

/// How screens and exports show a rolling token batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenBatchView {
    pub code: String,
    pub name: String,
    pub duration_minutes: String,
    pub audience: String,
    pub unit: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_rolling_token_batch_view(item: &RollingTokenBatch) -> RollingTokenBatchView {
    RollingTokenBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        audience: item.audience.clone(),
        unit: item.unit.clone(),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_token_batch_views(items: &[RollingTokenBatch]) -> Vec<RollingTokenBatchView> {
    let mut views: Vec<RollingTokenBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_token_batch_view(item));
    }
    views
}

pub fn sum_rolling_token_batch_view(view: &RollingTokenBatchView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_token_batch("code-992", "name-991");
        let view = to_rolling_token_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_token_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
