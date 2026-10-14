use super::*;

/// How screens and exports show a postmortem checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointView {
    pub code_text: String,
    pub name: String,
    pub description: String,
    pub port: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_postmortem_checkpoint_view(item: &PostmortemCheckpoint) -> PostmortemCheckpointView {
    PostmortemCheckpointView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        port: item.port.to_string(),
        segment_text: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_checkpoint_views(items: &[PostmortemCheckpoint]) -> Vec<PostmortemCheckpointView> {
    let mut views: Vec<PostmortemCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_checkpoint_view(item));
    }
    views
}

pub fn sum_postmortem_checkpoint_view(view: &PostmortemCheckpointView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_checkpoint("code-387", "name-515");
        let view = to_postmortem_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
