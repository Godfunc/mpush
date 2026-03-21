use std::time::Duration;
use ureq::{Agent, AgentBuilder};

const BASE_URL: &str = "https://api.weixin.qq.com";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(10);

fn agent() -> Agent {
    AgentBuilder::new()
        .timeout_connect(CONNECT_TIMEOUT)
        .timeout_read(READ_TIMEOUT)
        .build()
}

pub fn get_access_token(app_id: &str, app_secret: &str) -> Result<String, String> {
    let url = format!(
        "{BASE_URL}/cgi-bin/token?grant_type=client_credential&appid={app_id}&secret={app_secret}"
    );
    let resp = agent()
        .get(&url)
        .call()
        .map_err(|e| format!("network_error: {e}"))?;

    let body = resp.into_string().map_err(|e| format!("read_error: {e}"))?;

    if let Some(token) = extract_json_string(&body, "access_token") {
        return Ok(token);
    }

    let errcode = extract_json_i64(&body, "errcode").unwrap_or(-1);
    let errmsg = extract_json_string(&body, "errmsg").unwrap_or_default();
    Err(format!("token_failed errcode={errcode} errmsg={errmsg}"))
}

pub fn send_template_message(
    access_token: &str,
    user: &str,
    template_id: &str,
    link: Option<&str>,
    data: &str,
) -> Result<i64, String> {
    let url = format!("{BASE_URL}/cgi-bin/message/template/send?access_token={access_token}");
    let body = build_request_body(user, template_id, link, data);

    let resp = agent()
        .post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map_err(|e| format!("network_error: {e}"))?;

    let resp_body = resp.into_string().map_err(|e| format!("read_error: {e}"))?;
    let errcode = extract_json_i64(&resp_body, "errcode").unwrap_or(-1);

    if errcode == 0 {
        let msgid = extract_json_i64(&resp_body, "msgid").unwrap_or(0);
        return Ok(msgid);
    }

    let errmsg = extract_json_string(&resp_body, "errmsg").unwrap_or_default();
    Err(format!("errcode={errcode} errmsg={errmsg}"))
}

fn build_request_body(user: &str, template_id: &str, link: Option<&str>, data: &str) -> String {
    let url_field = match link {
        Some(l) => format!(r#","url":"{}""#, escape_json(l)),
        None => String::new(),
    };
    format!(
        r#"{{"touser":"{user}","template_id":"{tmpl}"{url_field},"data":{{"msg":{{"value":"{data}"}}}}}}"#,
        user = escape_json(user),
        tmpl = escape_json(template_id),
        data = escape_json(data),
    )
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{}""#, key);
    let start = json.find(&pattern)?;
    let after_key = start + pattern.len();
    let rest = json[after_key..].trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_json_i64(json: &str, key: &str) -> Option<i64> {
    let pattern = format!(r#""{}""#, key);
    let start = json.find(&pattern)?;
    let after_key = start + pattern.len();
    let rest = json[after_key..].trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request_body_with_link() {
        let body = build_request_body("oid123", "tmpl456", Some("http://example.com"), "hello");
        let body: String = body.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(body.contains(r#""touser":"oid123""#));
        assert!(body.contains(r#""template_id":"tmpl456""#));
        assert!(body.contains(r#""url":"http://example.com""#));
        assert!(body.contains(r#""msg":{"value":"hello"}"#));
    }

    #[test]
    fn test_build_request_body_without_link() {
        let body = build_request_body("oid123", "tmpl456", None, "hello");
        assert!(!body.contains("url"));
    }

    #[test]
    fn test_build_request_body_escapes_quotes() {
        let body = build_request_body("oid", "tmpl", None, r#"say "hi""#);
        assert!(body.contains(r#"say \"hi\""#));
    }

    #[test]
    fn test_extract_json_string_found() {
        let json = r#"{"access_token":"abc123","expires_in":7200}"#;
        assert_eq!(extract_json_string(json, "access_token"), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_json_string_not_found() {
        let json = r#"{"errcode":40013,"errmsg":"invalid appid"}"#;
        assert_eq!(extract_json_string(json, "access_token"), None);
    }

    #[test]
    fn test_extract_json_string_errmsg() {
        let json = r#"{"errcode":40013,"errmsg":"invalid appid"}"#;
        assert_eq!(extract_json_string(json, "errmsg"), Some("invalid appid".to_string()));
    }

    #[test]
    fn test_extract_json_i64_found() {
        let json = r#"{"errcode":0,"errmsg":"ok","msgid":200228332}"#;
        assert_eq!(extract_json_i64(json, "errcode"), Some(0));
        assert_eq!(extract_json_i64(json, "msgid"), Some(200228332));
    }

    #[test]
    fn test_extract_json_i64_negative() {
        let json = r#"{"errcode":-1,"errmsg":"system error"}"#;
        assert_eq!(extract_json_i64(json, "errcode"), Some(-1));
    }

    #[test]
    fn test_extract_json_i64_not_found() {
        let json = r#"{"access_token":"abc"}"#;
        assert_eq!(extract_json_i64(json, "errcode"), None);
    }

    #[test]
    fn test_escape_json_special_chars() {
        assert_eq!(escape_json(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(escape_json("line1\nline2"), r#"line1\nline2"#);
    }

    #[test]
    fn test_escape_json_no_special() {
        assert_eq!(escape_json("hello world"), "hello world");
    }
}
