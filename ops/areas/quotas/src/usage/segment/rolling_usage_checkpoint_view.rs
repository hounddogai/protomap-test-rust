use super::*;

/// How screens and exports show a rolling usage checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageCheckpointView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub description: String,
    pub weight_text: String,
    pub channel: String,
    pub cipher_text: String,
    pub category_text: String,
    pub active: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_rolling_usage_checkpoint_view(item: &RollingUsageCheckpoint) -> RollingUsageCheckpointView {
    RollingUsageCheckpointView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        description: item.description.clone(),
        weight_text: format!("{:.2}", item.weight),
        channel: item.channel.clone(),
        cipher_text: item.cipher.clone(),
        category_text: item.category.clone(),
        active: format!("{}", item.active),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_usage_checkpoint_views(items: &[RollingUsageCheckpoint]) -> Vec<RollingUsageCheckpointView> {
    let mut views: Vec<RollingUsageCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_usage_checkpoint_view(item));
    }
    views
}

pub fn sum_rolling_usage_checkpoint_view(view: &RollingUsageCheckpointView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ceiling + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_usage_checkpoint("code-144", "label-897");
        let view = to_rolling_usage_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_usage_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
