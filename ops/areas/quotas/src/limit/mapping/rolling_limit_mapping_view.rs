use super::*;

/// How screens and exports show a rolling limit mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitMappingView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub quota_units: String,
    pub burst: String,
    pub principal_text: String,
    pub quantity: String,
    pub cipher: String,
    pub updated_at: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_rolling_limit_mapping_view(item: &RollingLimitMapping) -> RollingLimitMappingView {
    RollingLimitMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        quota_units: item.quota_units.to_string(),
        burst: item.burst.to_string(),
        principal_text: item.principal.clone(),
        quantity: item.quantity.to_string(),
        cipher: item.cipher.clone(),
        updated_at: item.updated_at.to_string(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_limit_mapping_views(items: &[RollingLimitMapping]) -> Vec<RollingLimitMappingView> {
    let mut views: Vec<RollingLimitMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_limit_mapping_view(item));
    }
    views
}

pub fn sum_rolling_limit_mapping_view(view: &RollingLimitMappingView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_quota_units as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_limit_mapping("code-95", "title-395");
        let view = to_rolling_limit_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_limit_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
