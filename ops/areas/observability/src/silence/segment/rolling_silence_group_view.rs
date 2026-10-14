use super::*;

use acme_kit::text;

/// How screens and exports show a rolling silence group.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceGroupView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub batch_size_text: String,
    pub revision: String,
    pub ttl_seconds: String,
    pub namespace_text: String,
    pub status_label: String,
}

pub fn to_rolling_silence_group_view(item: &RollingSilenceGroup) -> RollingSilenceGroupView {
    RollingSilenceGroupView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        minimum: format!("{:.2}", item.minimum),
        batch_size_text: item.batch_size.to_string(),
        revision: item.revision.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        namespace_text: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_silence_group_views(items: &[RollingSilenceGroup]) -> Vec<RollingSilenceGroupView> {
    let mut views: Vec<RollingSilenceGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_silence_group_view(item));
    }
    views
}

pub fn sum_rolling_silence_group_view(view: &RollingSilenceGroupView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_minimum + (parsed_batch_size as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_silence_group("code-642", "name-551");
        let view = to_rolling_silence_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_silence_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
