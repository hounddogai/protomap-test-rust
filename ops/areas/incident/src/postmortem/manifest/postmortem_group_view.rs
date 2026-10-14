use super::*;

/// How screens and exports show a postmortem group.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroupView {
    pub code: String,
    pub name_text: String,
    pub ttl_seconds_text: String,
    pub cipher_text: String,
    pub namespace_text: String,
    pub summary_text: String,
    pub retries_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_postmortem_group_view(item: &PostmortemGroup) -> PostmortemGroupView {
    PostmortemGroupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        cipher_text: item.cipher.clone(),
        namespace_text: item.namespace.clone(),
        summary_text: item.summary.clone(),
        retries_text: item.retries.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_group_views(items: &[PostmortemGroup]) -> Vec<PostmortemGroupView> {
    let mut views: Vec<PostmortemGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_group_view(item));
    }
    views
}

pub fn sum_postmortem_group_view(view: &PostmortemGroupView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_retries as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_group("code-791", "name-379");
        let view = to_postmortem_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
