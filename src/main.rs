//use std::str;

//use log::{debug};
//use regex::Regex;

mod tex_ui;
mod args;
mod gerrit_if;
mod git;
mod os;
mod local_list;

// RUST_LOG=debug,info,warn

fn usage() {
    println!("Hello, git-reviewer!");
}

/*
fn list_reviews() {
    let remote_url = git::get_remote_url("...".to_string());
    debug!("remote url: {}", remote_url);

    let reviews = gerrit_if::query_reviews(remote_url, "");
    for r in reviews {
        let project = r["project"].as_str().unwrap();
        let branch = r["branch"].as_str().unwrap();
        let subject = r["subject"].as_str().unwrap();
        let owner = git::get_git_user(&r["owner"]);
        println!("{}/{} | {} | {}", project, branch, owner.username, subject);
    }
}
*/

// project, group, recent reviews
fn _get_candidate_reviewers() {

}

/*
fn list_reviewers() {
    /*
    for r in get_reviewers(None) {
        println!("{}", r.username);
    }
    */
    println!("Current reviewers:");
    let cur_reviewers = gerrit_if::get_reviewers(Some("I1db9608c85fe80b30ffa881b60968d9695a463ff"));
    for r in cur_reviewers {
        println!("    |{}", r.username);
    }

    let _candicates = gerrit_if::get_reviewers(None);
}*/

fn _main() -> i32 {

    usage();
    let arg = args::parse();
    env_logger::Builder::new()
        .filter_level(arg.verbose.log_level_filter())
        .init();
    //gerrit_if::init(arg);
    tex_ui::init(arg);
    
    /*

    let _git_dir = get_git_directories();

    let change_id = "I1db9608c85fe80b30ffa881b60968d9695a463ff";
    list_reviews();
    list_reviewers();

    for id in get_unmerged_local_changes() {
        println!("{}", id);
    }
    */


    0
}

fn main() {
    let ret = _main();

    std::process::exit(ret);
}

