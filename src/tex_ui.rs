use std::io;

use crate::gerrit_if;
use crate::gerrit_if::{GerritChange, GerritUser, GitUrl};
use crate::args::Args;
use crate::groups;

trait Option {
    fn action(&self, change: &GerritChange);
    fn get_desc(&self) -> &str {
        ""
    }
}

// ShowCurReviewers
struct ShowCurReviewers;
impl ShowCurReviewers {
    fn run(change :&GerritChange) {
        println!("ShowCurReviewers");
        let reviewers = gerrit_if::get_reviewers(change);

        if reviewers.len() == 0 {
            println!("* No reviewers!");
        }
        for r in reviewers {
            println!("* {}", r.as_string());
        }
    }
}

impl Option for ShowCurReviewers {
    fn action(&self, change :&GerritChange) {
        ShowCurReviewers::run(change);
    }

    fn get_desc(&self) -> &str {
        "Show current reviewers of the change"
    }
}

// AddReviewers
struct AddReviewers;
impl AddReviewers {
    fn run(change: &GerritChange, name: &str) {
        println!("Adding reviewers");

        match gerrit_if::add_reviewer(change, name) {
            Ok(_) => {
                ShowCurReviewers::run(change);
            },
            Err(err) => println!("{}", err),
        }
    }
}

impl Option for AddReviewers {
    fn action(&self, change: &GerritChange) {
        let mut name = String::new();
        println!("Input 1 reviewer name:");
        io::stdin().read_line(&mut name).expect("Failed to read line");
        AddReviewers::run(change, name.trim());
    }

    fn get_desc(&self) -> &str {
        "Add reviewers to the change"
    }
}

// Delete all reviewers
struct ClearReviewers;
impl ClearReviewers {
    fn run(change: &GerritChange) {
        let reviewers = gerrit_if::get_reviewers(change);

        if reviewers.len() == 0 {
            println!("No reviewers now");
            return;
        }

        match gerrit_if::delete_reviewers(change, &reviewers) {
            Ok(_) => {
                ShowCurReviewers::run(change);
            },
            Err(err) => println!("{}", err),
        }
    }
}
impl Option for ClearReviewers {
    fn action(&self, change: &GerritChange) {
        ClearReviewers::run(change);
    }

    fn get_desc(&self) -> &str {
        "Remove all reviewers"
    }
}

// Get recent reviews of user
struct ShowRecentReviews;
impl ShowRecentReviews {
    fn run(url: &GitUrl, user: &GerritUser) {
        let reviews = gerrit_if::get_user_recent_reviews(url, user);
        for r in reviews {
            let project = r["project"].as_str().unwrap();
            let branch = r["branch"].as_str().unwrap();
            let subject = r["subject"].as_str().unwrap();
            let owner = r["owner"].as_object().unwrap()["username"].as_str().unwrap();
            println!("{}/{} | {} | {}", project, branch, owner, subject);
        }
    }
}
impl Option for ShowRecentReviews {
    fn action(&self, change: &GerritChange) {
        // query
        // show recent for user
        let user = GerritUser::from_str(&change.conn.username.as_ref().unwrap());
        ShowRecentReviews::run(&change.conn, &user);
    }
    fn get_desc(&self) -> &str {
        "Show recent reviews of the user"
    }
}

// Get recent reviews of user
struct ShowRecentReviewers;
impl ShowRecentReviewers {
    fn run(url: &GitUrl, user: &GerritUser) {
        let reviewers = gerrit_if::get_user_recent_reviewers(url, user);
        for r in reviewers {
            println!("* {}", r.as_string());
        }
    }
}
impl Option for ShowRecentReviewers {
    fn action(&self, change: &GerritChange) {
        // query
        // show recent for user
        let user = GerritUser::from_str(&change.conn.username.as_ref().unwrap());
        ShowRecentReviewers::run(&change.conn, &user);
    }
    fn get_desc(&self) -> &str {
        "Show recent reviewers of the user"
    }
}

// Add reviewers from candidates
struct AddFromCandidate;
impl Option for AddFromCandidate {
    fn action(&self, change: &GerritChange) {
        let user = GerritUser::from_str(&change.conn.username.as_ref().unwrap());
        let cur_reviewers = gerrit_if::get_reviewers(change);
        let mut candidates = gerrit_if::get_user_recent_reviewers(&change.conn, &user);

        ShowCurReviewers::run(change);

        // Remove exist reviewers
        for r in cur_reviewers {
            for (pos, c) in candidates.iter().enumerate() {
                if r.username.eq(&c.username) {
                    candidates.remove(pos);
                    break;
                }
            }
        }
        
        println!("Candidates: ");
        for (pos, c) in candidates.iter().enumerate() {
            println!("  {}: {}", pos + 1, c.as_string());
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let mut sel = 0;
        match input.trim().parse::<usize>() {
            Ok(_sel) => {
                if _sel <= candidates.len() {
                    sel = _sel - 1;
                } else {
                    println!("Invalid option");
                    return;
                }
            },
            Err(_) => {
                println!("Invalid input");
                return;
            }
        }

        AddReviewers::run(change, &candidates[sel].username);
    }
    fn get_desc(&self) -> &str {
        "Add reviewers from candidates"
    }
}

// ShowGroups
struct ShowGroups;
impl Option for ShowGroups {
    fn action(&self, change: &GerritChange) {
        let gs = groups::get_groups();

        for g in gs {
            println!("Group: {}", &g.name);
            for u in &g.users {
                println!("* {}", u.as_string());
            }
        }
    }
    fn get_desc(&self) -> &str {
        "Show all groups"
    }
}


pub fn init(arg: Args) {
    println!("Text UI init");

    let change = gerrit_if::get_gerrit_change(arg);
    
    // TODO loop prompt
    loop {
        let again = prompt(&change);
        if !again {
            break;
        }
    }
}

struct Options {
    list: Vec<Box<dyn Option>>,
}
impl Options {
    fn new() -> Self {
        Self {list: vec![]}
    }

    fn add(&mut self, option: Box<dyn Option>) {
        self.list.push(option);
    }
}

fn prompt(change: &GerritChange) -> bool {

    let mut options = Options::new();

    // Append the list
    options.add(Box::new(ShowCurReviewers));
    options.add(Box::new(AddReviewers));
    options.add(Box::new(ClearReviewers));
    options.add(Box::new(ShowRecentReviews));
    options.add(Box::new(ShowRecentReviewers));
    options.add(Box::new(AddFromCandidate));
    options.add(Box::new(ShowGroups));

    println!("---------------------");
    println!("Options:");
    for (pos, opt) in options.list.iter().enumerate() {
        println!("  {}: {}", pos + 1, opt.get_desc());
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    match input.trim().parse::<usize>() {
        Ok(sel) => {
            if sel <= options.list.len() {
                options.list[sel - 1].action(change);
                true
            } else {
                println!("Invalid option");
                false
            }
        },
        Err(_) => {
            println!("Invalid input");
            false
        }
    }
}

