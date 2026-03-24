#[derive(Debug)]
pub struct Args {
    pub user: Vec<String>,
    pub template: String,
    pub link: Option<String>,
    pub data: String,
}

#[derive(Debug)]
pub enum ParseResult {
    Args(Args),
    Help,
}

fn parse_users(value: &str) -> Result<Vec<String>, String> {
    if value.is_empty() {
        return Err("invalid --user value: empty openid segment".to_string());
    }
    let users: Vec<String> = value.split(',').map(|s| s.to_string()).collect();
    if users.iter().any(|s| s.is_empty()) {
        return Err("invalid --user value: empty openid segment".to_string());
    }
    Ok(users)
}

pub fn parse_args(args: &[String]) -> Result<ParseResult, String> {
    let mut user: Option<Vec<String>> = None;
    let mut template: Option<String> = None;
    let mut link: Option<String> = None;
    let mut data: Option<String> = None;

    let mut i = 1; // skip program name
    while i < args.len() {
        let arg = &args[i];

        if arg == "-h" || arg == "--help" {
            return Ok(ParseResult::Help);
        }

        // handle --key=value syntax
        if let Some((key, value)) = arg.split_once('=') {
            match key {
                "--user" => user = Some(parse_users(value)?),
                "--template" => template = Some(value.to_string()),
                "--link" => link = Some(value.to_string()),
                "--data" => data = Some(value.to_string()),
                _ => return Err(format!("unknown option: {key}")),
            }
            i += 1;
            continue;
        }

        // handle -k value / --key value syntax
        let value = args.get(i + 1).ok_or_else(|| format!("missing value for {arg}"))?;
        match arg.as_str() {
            "-u" | "--user" => user = Some(parse_users(value)?),
            "-t" | "--template" => template = Some(value.to_string()),
            "-l" | "--link" => link = Some(value.to_string()),
            "-d" | "--data" => data = Some(value.to_string()),
            _ => return Err(format!("unknown option: {arg}")),
        }
        i += 2;
    }

    let user = user.ok_or("missing required option: --user")?;
    let template = template.ok_or("missing required option: --template")?;
    let data = data.ok_or("missing required option: --data")?;

    Ok(ParseResult::Args(Args { user, template, link, data }))
}

pub fn print_help() {
    println!(
        "Usage: mpush -u <openid[,openid...]> -t <template_id> [-l <link>] -d <data>\n\
         \n\
         Options:\n\
         \x20 -u, --user       接收者 openid（多个用逗号分隔）\n\
         \x20 -t, --template   模板 ID\n\
         \x20 -l, --link       模板跳转链接\n\
         \x20 -d, --data       消息内容\n\
         \x20 -h, --help       显示帮助信息\n\
         \n\
         Environment:\n\
         \x20 MPUSH_APP_ID      微信公众号 appid\n\
         \x20 MPUSH_APP_SECRET  微信公众号 appsecret"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_all_args() {
        let input = args(&["mpush", "-u", "openid1", "-t", "tmpl1", "-l", "http://example.com", "-d", "hello"]);
        let result = parse_args(&input).unwrap();
        match result {
            ParseResult::Args(a) => {
                assert_eq!(a.user, vec!["openid1"]);
                assert_eq!(a.template, "tmpl1");
                assert_eq!(a.link.as_deref(), Some("http://example.com"));
                assert_eq!(a.data, "hello");
            }
            ParseResult::Help => panic!("expected Args"),
        }
    }

    #[test]
    fn test_parse_long_args() {
        let input = args(&["mpush", "--user", "openid1", "--template", "tmpl1", "--data", "hello"]);
        let result = parse_args(&input).unwrap();
        match result {
            ParseResult::Args(a) => {
                assert_eq!(a.user, vec!["openid1"]);
                assert_eq!(a.template, "tmpl1");
                assert!(a.link.is_none());
                assert_eq!(a.data, "hello");
            }
            ParseResult::Help => panic!("expected Args"),
        }
    }

    #[test]
    fn test_parse_equal_sign_syntax() {
        let input = args(&["mpush", "--user=openid1", "--template=tmpl1", "--data=hello"]);
        let result = parse_args(&input).unwrap();
        match result {
            ParseResult::Args(a) => {
                assert_eq!(a.user, vec!["openid1"]);
                assert_eq!(a.template, "tmpl1");
                assert_eq!(a.data, "hello");
            }
            ParseResult::Help => panic!("expected Args"),
        }
    }

    #[test]
    fn test_parse_help_short() {
        let input = args(&["mpush", "-h"]);
        let result = parse_args(&input).unwrap();
        assert!(matches!(result, ParseResult::Help));
    }

    #[test]
    fn test_parse_help_long() {
        let input = args(&["mpush", "--help"]);
        let result = parse_args(&input).unwrap();
        assert!(matches!(result, ParseResult::Help));
    }

    #[test]
    fn test_parse_help_takes_priority() {
        let input = args(&["mpush", "-u", "openid1", "-h", "-t", "tmpl1", "-d", "hello"]);
        let result = parse_args(&input).unwrap();
        assert!(matches!(result, ParseResult::Help));
    }

    #[test]
    fn test_parse_missing_user() {
        let input = args(&["mpush", "-t", "tmpl1", "-d", "hello"]);
        let result = parse_args(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--user"));
    }

    #[test]
    fn test_parse_missing_template() {
        let input = args(&["mpush", "-u", "openid1", "-d", "hello"]);
        let result = parse_args(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--template"));
    }

    #[test]
    fn test_parse_missing_data() {
        let input = args(&["mpush", "-u", "openid1", "-t", "tmpl1"]);
        let result = parse_args(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--data"));
    }

    #[test]
    fn test_parse_no_args() {
        let input = args(&["mpush"]);
        let result = parse_args(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_flag() {
        let input = args(&["mpush", "--unknown", "val", "-u", "openid1", "-t", "tmpl1", "-d", "hello"]);
        let result = parse_args(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown"));
    }

    #[test]
    fn test_parse_multiple_users() {
        let input = args(&["mpush", "-u", "id1,id2,id3", "-t", "tmpl1", "-d", "hello"]);
        let result = parse_args(&input).unwrap();
        match result {
            ParseResult::Args(a) => {
                assert_eq!(a.user, vec!["id1", "id2", "id3"]);
            }
            ParseResult::Help => panic!("expected Args"),
        }
    }

    #[test]
    fn test_parse_empty_openid_segment() {
        let input = args(&["mpush", "-u", "id1,,id2", "-t", "tmpl1", "-d", "hello"]);
        assert!(parse_args(&input).is_err());

        let input = args(&["mpush", "-u", ",id1", "-t", "tmpl1", "-d", "hello"]);
        assert!(parse_args(&input).is_err());

        let input = args(&["mpush", "-u", "id1,", "-t", "tmpl1", "-d", "hello"]);
        assert!(parse_args(&input).is_err());
    }

    #[test]
    fn test_parse_multiple_users_equals_syntax() {
        let input = args(&["mpush", "--user=id1,id2", "--template=tmpl1", "--data=hello"]);
        let result = parse_args(&input).unwrap();
        match result {
            ParseResult::Args(a) => {
                assert_eq!(a.user, vec!["id1", "id2"]);
            }
            ParseResult::Help => panic!("expected Args"),
        }
    }

    #[test]
    fn test_parse_empty_user_equals() {
        let input = args(&["mpush", "--user=", "--template=tmpl1", "--data=hello"]);
        assert!(parse_args(&input).is_err());
    }

    #[test]
    fn test_parse_users_no_whitespace_trim() {
        let input = args(&["mpush", "-u", "id1, id2", "-t", "tmpl1", "-d", "hello"]);
        let result = parse_args(&input).unwrap();
        match result {
            ParseResult::Args(a) => {
                assert_eq!(a.user, vec!["id1", " id2"]);
            }
            ParseResult::Help => panic!("expected Args"),
        }
    }
}
