use super::*;

/// How screens and exports show a token segment.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSegmentView {
    pub code: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub retries: String,
    pub external_ref: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_token_segment_view(item: &TokenSegment) -> TokenSegmentView {
    TokenSegmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        retries: item.retries.to_string(),
        external_ref: item.external_ref.clone(),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_segment_views(items: &[TokenSegment]) -> Vec<TokenSegmentView> {
    let mut views: Vec<TokenSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_segment_view(item));
    }
    views
}

pub fn sum_token_segment_view(view: &TokenSegmentView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_segment("code-819", "label-311");
        let view = to_token_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
