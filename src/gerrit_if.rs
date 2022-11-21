//use std::str;
use url::Url;
use regex::Regex;
use log::{debug, info};
use linked_hash_set::LinkedHashSet;

use crate::os;
use crate::git;
use crate::args::Args;
use crate::users::GerritUser;
use crate::users;

use serde_json::Value as Review;

const QUERY_LIMIT: u32 = 10;

// FIXME rename to connection or gerrit url
pub struct GitUrl {
    scheme: String,
    hostname: String,
    pub username: Option<String>,
    port: Option<u16>,
    project: String,
    raw: String,
}

pub struct GerritChange {
    pub conn: GitUrl,
    pub change_id: String,
}

pub struct Gerrit {
    pub git_dir: String,
}

impl Gerrit {
    pub fn new() -> Self {
        let git_dir = git::get_git_directories().unwrap();
        info!("Get git directory: {}", git_dir);
        Self {
            git_dir: git_dir,
        }
    }
}

pub fn get_gerrit_change(args: Args) -> GerritChange {

    let change = GerritChange {
        // FIXME
        conn: parse_gerrit_ssh_params_from_git_url(&args.url.unwrap_or("git@github.com:ponsheng/gerrit-reviewers.git".to_string())),
        change_id: args.change.unwrap_or("NA".to_string()),
    };
    change
}

fn parse_gerrit_ssh_params_from_git_url(remote_url: &str) -> GitUrl {
    let mut ret;

    if remote_url.find("://").is_some() {
        let parsed_url;
        let result = Url::parse(remote_url);
        match result {
            Ok(url) => parsed_url = url,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        }

        ret = GitUrl {
            scheme: {
                let s = parsed_url.scheme();
                // FIXME NA
                if s.is_empty() { "NA".to_string() } else { s.to_string() }
            },
            username: {
                let s = parsed_url.username();
                if s.is_empty() { None } else { Some(s.to_string()) }
            },
            hostname: {
                match parsed_url.host_str() {
                    Some(s) => s.to_string(),
                    None => panic!("Hostname is not found")
                }
            },
            port: parsed_url.port_or_known_default(),
            project: parsed_url.path().to_string(),
            raw: remote_url.to_string(),
        };
    } else {
        // TODO Handle SCP-style addresses
        // e.g. git@github.com:ponsheng/gerrit-reviewers.git
        ret = GitUrl {
            // FIXME
            scheme: "".to_string(),
            username: None,
            hostname: "".to_string(),
            port: None,
            project: "".to_string(),
            raw: remote_url.to_string(),
        };
    }
    
    // TODO Strip leading slash and trailing '.git' form project name
    let re = Regex::new(r"^/|(\.git$)").unwrap();
    ret.project = re.replace_all(&ret.project, "").to_string();

    ret
}

fn query_reviews_over_ssh(remote_url: &str, more_args: &str) -> Vec<serde_json::Value> {
    let url = parse_gerrit_ssh_params_from_git_url(&remote_url);

    let mut query;
    let user_host;

    // status:open
    query = format!("project:{}", url.project);

    match url.username {
        Some(s) => {
            user_host = format!("{}@{}", s, url.hostname);
        },
        None => user_host = format!("{}", url.hostname),
    }

    let port = {
        match url.port {
            Some(p) => format!("-p {}", p),
            None => "".to_string()
        }
    };

    // Set limit
    if QUERY_LIMIT > 0 {
        query.push_str(&format!(" limit: {}", QUERY_LIMIT));
    }

    let cmd = vec!["ssh", "-x", &*port, &*user_host, "gerrit", "query", "--format=JSON", more_args, &*query];
    let resp = os::run_command_exc(cmd).unwrap();

    /*
        Ok(s) => ,
        Err(s) => panic!("{}", s)
    };*/

    let mut changes = Vec::new();
    for line in resp.split('\n') {
        let json: serde_json::Value = serde_json::from_str(line).unwrap();
        if json.get("rowCount").is_some() {
            break;
        }
        changes.push(json);
    }
    
    debug!("Queried {} changes", changes.len());
    
    //debug!(output);
    changes
}

pub fn query_reviews(remote_url: &str, more_args: &str) -> Vec<serde_json::Value> {
    if remote_url.starts_with("http://") || remote_url.starts_with("https://") {
        // TODO
        panic!("HTTP not supported yet");
    } else {
        // ssh
        query_reviews_over_ssh(remote_url, more_args)
    }
}

pub fn add_reviewer(change: &GerritChange, name: &str) -> Result<String, String> {
    // TODO use trait for ssh/http
    if change.conn.scheme.eq("ssh") {
        let reviewers = vec![GerritUser::from_str(name)];
        set_reviewers_over_ssh(change, &reviewers, true)
    } else {
        panic!("Unsupported scheme");
    }
}

pub fn delete_reviewers(change: &GerritChange, reviewers: &Vec<GerritUser>) -> Result<String, String> {
    // TODO use trait for ssh/http
    if change.conn.scheme.eq("ssh") {
        set_reviewers_over_ssh(change, reviewers, false)
    } else {
        panic!("Unsupported scheme");
    }
}

/// NOTE non-owner might not be able to remove reviewers
fn set_reviewers_over_ssh(change: &GerritChange, reviewers: &Vec<GerritUser>, is_add: bool) -> Result<String, String> {
    assert!(reviewers.len() > 0, "Reviewer list is empty");

    let change_id = &change.change_id;
    let url = &change.conn;
    let user_host = {
        match &url.username {
            Some(s) => format!("{}@{}", s, url.hostname),
            None => format!("{}", url.hostname),
        }
    };

    let port = {
        match url.port {
            Some(p) => format!("-p {}", p),
            None => "".to_string()
        }
    };
    let project_str = format!("-p {}", url.project);

    let mut option_str = String::new();

    let cur_reviewers = get_reviewers(change);

    for user in reviewers {
        let mut already_has_user = false;
        for r in &cur_reviewers {
            if user.username.eq(&r.username) {
                already_has_user = true;
            }
        }

        // Check if skip the user
        if is_add && already_has_user {
            println!("{} is already a reviewer", user.username);
            continue;
        } else if !is_add && !already_has_user {
            println!("{} is not a reviewer", user.username);
            continue;
        }

        if is_add {
            option_str.push_str(" --add ");
        } else {
            option_str.push_str(" --remove ");
        }
        option_str.push_str(&user.username.to_string());
    }

    if option_str.len() != 0 {
        let cmd = vec!["ssh", "-x", &*port, &*user_host, "gerrit", "set-reviewers", &*project_str, &*option_str, change_id];
        os::run_command_exc(cmd)
    } else {
        Ok("Nothing happened".to_string())
    }
}

/*
pub fn is_change_open(change_id: &str) -> bool {
    let remote_url = git::get_remote_url("...".to_string());

    let more_args = format!("status:open {}", change_id);
    let reviews = query_reviews(remote_url, &more_args);

    if reviews.len() == 0 {
        return false;
    }

    let obj = reviews.get(0).unwrap();

    obj.get("open").unwrap().as_bool().unwrap()
}
*/

pub fn get_reviewers(change: &GerritChange) -> Vec<GerritUser> {
    let remote_url = &change.conn.raw;
    let more_args = format!("--all-reviewers {}", change.change_id);

    let reviews = query_reviews(remote_url, &more_args);

    let mut reviewer_list = Vec::new();

    for r in reviews {
        if r.get("allReviewers").is_none() {
            continue;
        }
        let reviewers = r["allReviewers"].as_array().unwrap();
        for reviewer in reviewers {
            let user = users::get_git_user(reviewer);
            reviewer_list.push(user);
        }
    }
    reviewer_list
}

pub fn get_user_recent_reviews(conn: &GitUrl, user: &GerritUser) -> Vec<Review> {
    let remote_url = &conn.raw;
    let more_args = format!(" owner:{}", user.username);
    query_reviews(remote_url, &more_args)
}

pub fn get_user_recent_reviewers(conn: &GitUrl, user: &GerritUser) -> Vec<GerritUser> {
    let remote_url = &conn.raw;
    let more_args = format!("--all-reviewers owner:{}", user.username);

    let reviews = query_reviews(remote_url, &more_args);

    
    // Collect reviewers into HashSet
    let mut reviewer_set = LinkedHashSet::new();
    for r in reviews {
        if r.get("allReviewers").is_none() {
            continue;
        }
        let reviewers = r["allReviewers"].as_array().unwrap();
        for reviewer in reviewers {
            reviewer_set.insert(users::get_git_user(reviewer));
        }
    }

    let mut reviewer_list = Vec::new();
    loop {
        let reviewer = match reviewer_set.pop_front() {
            Some(r) => reviewer_list.push(r),
            None => break,
        };
    }

    reviewer_list
}

