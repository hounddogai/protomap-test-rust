use super::*;

/// How screens and exports show a sink request.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRequestView {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub currency_text: String,
    pub version: String,
    pub limit: String,
    pub weight_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_sink_request_view(item: &SinkRequest) -> SinkRequestView {
    SinkRequestView {
        code: item.code.clone(),
        label: item.label.clone(),
        cipher: item.cipher.clone(),
        currency_text: item.currency.clone(),
        version: item.version.to_string(),
        limit: item.limit.to_string(),
        weight_text: format!("{:.2}", item.weight),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_request_views(items: &[SinkRequest]) -> Vec<SinkRequestView> {
    let mut views: Vec<SinkRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_request_view(item));
    }
    views
}

pub fn sum_sink_request_view(view: &SinkRequestView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_version as f64 + (parsed_limit as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_request("code-753", "label-720");
        let view = to_sink_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
