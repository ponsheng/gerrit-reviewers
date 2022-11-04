use std::str;
use std::process::Command;

use log::{info, debug, warn, trace};
use url::Url;
use regex::Regex;

// RUST_LOG=debug,info,warn

struct GitUrl {
    _scheme: Option<String>,
    hostname: String,
    username: Option<String>,
    port: Option<u16>,
    project: String,
}

struct GitUser {
    _email: String,
    name: String,
    username: String,
}

struct _GitUserGroup {
    users: Vec<GitUser>,
    name: String,
    is_unity: bool,
}

fn usage() {
    println!("Hello, git-reviewer!");
}

fn run_command_exc(cmd_vec : Vec<&str>) -> Result<String, String> {

    trace!("Running: {}", cmd_vec.join(" "));

    let cmd = cmd_vec[0];
    let result = Command::new(cmd)
        .args(&cmd_vec[1..])
        .output()
        .expect("failed to execute process");

    if !result.status.success() {
        let err_msg = str::from_utf8(&result.stderr).expect("Invalid UTF8-8 sequence");
        return Err(err_msg.to_string())
    }

    let out = str::from_utf8(&result.stdout).expect("Invalid UTF8-8 sequence");
    Ok(out.to_string())
}

fn git_config_get_value(section : &str, option : &str) -> Result<String, String> {
    let name = format!("{}.{}", section, option);
    let cmd = vec!["git", "config", "--get", &*name];

    let result = run_command_exc(cmd);

    result
}

// FIXME
fn alias_url(url : String, _rewrite_push : bool) -> String {
    url
}

// FIXME remote not respected
fn get_remote_url(_remote : String) -> String {
    let push_url;
    let ret = git_config_get_value("remote.origin", "pushurl");
    match ret {
        Ok(s) => push_url = alias_url(s, false),

        Err(_) => {
            let ret = git_config_get_value("remote.origin", "url");
            match ret {
                Ok(s) => push_url = alias_url(s, true),
                Err(_) => panic!("Failed to get remote url")
            }
        }
    }
    debug!("Found origin Push URL: {}", push_url);
    push_url
}

fn parse_gerrit_ssh_params_from_git_url(remote_url: String) -> GitUrl {
    let mut ret;

    if remote_url.find("://").is_some() {
        let parsed_url;
        let result = Url::parse(&remote_url);
        match result {
            Ok(url) => parsed_url = url,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        }

        ret = GitUrl {
            _scheme: {
                let s = parsed_url.scheme();
                if s.is_empty() { None } else { Some(s.to_string()) }
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
        };
    } else {
        // TODO Handle SCP-style addresses
        panic!("Unreachable");
    }
    
    // TODO Strip leading slash and trailing '.git' form project name
    let re = Regex::new(r"^/|(\.git$)").unwrap();
    ret.project = re.replace_all(&ret.project, "").to_string();

    ret
}

fn query_reviews_over_ssh(remote_url: String, more_args: &str) -> Vec<serde_json::Value> {
    let url = parse_gerrit_ssh_params_from_git_url(remote_url);

    let mut query;
    let user_host;

    // status:open
    query = format!("project:{}", url.project);

    let username = String::new();
    match url.username {
        Some(s) => {
            user_host = format!("{}@{}", s, url.hostname);
            query.push_str(&format!(" owner:{}", s));
        },
        None => user_host = format!("{}", url.hostname),
    }

    let port = {
        match url.port {
            Some(p) => format!("-p {}", p),
            None => "".to_string()
        }
    };

    let cmd = vec!["ssh", "-x", &*port, &*user_host, "gerrit", "query", "--format=JSON", more_args, &*query];
    let resp = run_command_exc(cmd).unwrap();

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

fn query_reviews(remote_url: String, more_args: &str) -> Vec<serde_json::Value> {
    if remote_url.starts_with("http://") || remote_url.starts_with("https://") {
        // TODO
        panic!("HTTP not supported yet");
    } else {
        // ssh
        query_reviews_over_ssh(remote_url, more_args)
    }
}

fn list_reviews() {
    let remote_url = get_remote_url("...".to_string());
    debug!("remote url: {}", remote_url);

    let reviews = query_reviews(remote_url, "");
    for r in reviews {
        let project = r["project"].as_str().unwrap();
        let branch = r["branch"].as_str().unwrap();
        let subject = r["subject"].as_str().unwrap();
        let owner = get_git_user(&r["owner"]);
        println!("{}/{} | {} | {}", project, branch, owner.username, subject);
    }
}

fn get_git_user(json_val: &serde_json::Value) -> GitUser {
    let info = GitUser {
        username: json_val["username"].as_str().unwrap_or_default().to_string(),
        _email: json_val["email"].as_str().unwrap_or_default().to_string(),
        name: json_val["name"].as_str().unwrap_or_default().to_string(),
    };
    return info;
}

fn get_reviewers(change_id: Option<&str>) -> Vec<GitUser> {
    let more_args = match change_id {
        None => "--all-reviewers".to_string(),
        Some(id) => format!("--all-reviewers {}", id),
    };

    let remote_url = get_remote_url("...".to_string());
    let reviews = query_reviews(remote_url, &more_args);

    let mut reviewer_list = Vec::new();
    for r in reviews {
        if r.get("allReviewers").is_none() {
            continue;
        }
        let reviewers = r["allReviewers"].as_array().unwrap();
        for reviewer in reviewers {
            reviewer_list.push(get_git_user(reviewer));
        }
    }
    reviewer_list
}

fn add_reviewers_over_ssh(remote_url: String, new_reviewers: &Vec<GitUser>, change_id: &str) {
    let url = parse_gerrit_ssh_params_from_git_url(remote_url);
    let user_host = {
        match url.username {
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

    assert!(new_reviewers.len() > 0);
    let mut option_str = String::new();

    let cur_reviewers = get_reviewers(Some(change_id));

    for user in new_reviewers {
        for r in &cur_reviewers {
            if user.name.eq(&r.name) {
                println!("{} is already a reviewer", user.name);
                continue;
            }
        }
        option_str.push_str(" -a ");
        option_str.push_str(&user.name.to_string());
    }

    if option_str.len() != 0 {
        let cmd = vec!["ssh", "-x", &*port, &*user_host, "gerrit", "set-reviewers", &*project_str, &*option_str, change_id];
        match run_command_exc(cmd) {
            Ok(_) => (),
            Err(stderr) => println!("{}", stderr),
        }

    } else {
        println!("Nothing happened");
    }
}

// project, group, recent reviews
fn _get_candidate_reviewers() {

}

fn list_reviewers() {
    /*
    for r in get_reviewers(None) {
        println!("{}", r.username);
    }
    */
    println!("Current reviewers:");
    let cur_reviewers = get_reviewers(Some("I1db9608c85fe80b30ffa881b60968d9695a463ff"));
    for r in cur_reviewers {
        println!("    |{}", r.username);
    }

    let _candicates = get_reviewers(None);


}


fn get_git_directories() -> (String, String) {
    let ret = run_command_exc(vec!["git", "rev-parse", "--show-toplevel", "--git-dir"]);
    match ret {
        Ok(s) => {
            let mut lines = s.split('\n');
            let top_dir = lines.nth(0).expect("Vec out of bound").to_string();
            let git_dir = lines.nth(1).expect("Vec out of bound").to_string();
            (top_dir, git_dir)
        },
        Err(_) => panic!("Cannot find .git directory")
    }
}

// Return "" if command failed
fn get_local_commit_message(git_ref: &str) -> String {
    let cmd = vec!["git", "show", "-s", "--format=medium", git_ref];
    match run_command_exc(cmd) {
        Ok(stdout) => stdout,
        Err(stderr) => "".to_string(),
    }
}

// Return None if failed to use git_ref
// Return "" if ChangeID not found
fn get_local_commit_change_id(git_ref: &str) -> Option<String> {
    let re = Regex::new(r"^\s*Change-Id: ([A-Za-z0-9]+)$").unwrap();

    let message = get_local_commit_message(git_ref);

    if message.len() == 0 {
        return None;
    }

    let mut ret = "".to_string();
    for line in message.split('\n') {
        match re.captures(line) {
            Some(caps) => {
                let id = caps.get(1).unwrap().as_str();
                if ret.len() > 0 {
                    warn!("Multiple Change-Id in commit");
                    continue;
                }
                ret = id.to_string();
            },
            None => continue,
        }
    }
    Some(ret)
}

fn is_change_open(change_id: &str) -> bool {
    let remote_url = get_remote_url("...".to_string());

    let more_args = format!("status:open {}", change_id);
    let reviews = query_reviews(remote_url, &more_args);

    if reviews.len() == 0 {
        return false;
    }

    let obj = reviews.get(0).unwrap();

    obj.get("open").unwrap().as_bool().unwrap()
}

// Max: 8
fn get_unmerged_local_changes() -> Vec<String> {
    info!("Searching for target commit");

    let mut commits = Vec::new();

    for backward_index in 0..8 {
        let git_ref = format!("HEAD~{}", backward_index);
        info!("Ref: {}", git_ref);
        match get_local_commit_change_id(&git_ref) {
            Some(id) => {
                if id.len() == 0 {
                    continue;
                }
                info!("    Change id:{}", id);
                if is_change_open(&id) {
                    commits.push(id);
                }
            },
            None => break,
        }
    }
    commits
}

fn _main() -> i32 {

    usage();

    let _git_dir = get_git_directories();

    let change_id = "I1db9608c85fe80b30ffa881b60968d9695a463ff";
    list_reviews();
    list_reviewers();

    for id in get_unmerged_local_changes() {
        println!("{}", id);
    }

    0
}

fn main() {


    env_logger::init();
    
    let ret = _main();

    std::process::exit(ret);
}
