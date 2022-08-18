use git2::{Repository, RepositoryInitOptions, FetchOptions, RemoteCallbacks, Cred, IndexAddOption, PushOptions, Signature};
use merge::do_merge;
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window, enums::Color, input::Input, group::Flex};
use std::{env, path::PathBuf, fs::File, io::Write};
mod merge;


fn make_new_repo(path :&PathBuf, url :&str) {
    let mut repo_conf = RepositoryInitOptions::new();
    repo_conf.origin_url(url).no_reinit(true).initial_head("main");

    let _repo = match Repository::init_opts(path, &repo_conf) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init: {}", e),
    };

    //adding executable to gitignore
    let mut file = File::create(".gitignore").unwrap();
    file.write_all(b"ramus_git\nramus_git.exe").unwrap();
}

fn sync_to_repo(path :&PathBuf, token :&String) { 
    //Fetch
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let mut fetch = FetchOptions::new();
    let mut cbs = RemoteCallbacks::new();

    cbs.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(token.as_str(),token.as_str())
    });

    fetch.remote_callbacks(cbs);
    repo.find_remote("origin").unwrap().fetch(&["main"], Some(&mut fetch), None).expect("Fetch Failed");

    //Merge
    let url = repo.find_remote("origin").unwrap();
    let fetch_head = repo.find_reference("refs/remotes/origin/main").expect("Fetch Head is not there");
    let fetch_commit = repo.annotated_commit_from_fetchhead("Main", url.url().expect("URL not found"),
        &fetch_head.target().expect("OID not found")).expect("Commit didn't work");
    do_merge(&repo, "main", fetch_commit).expect("Merge Failed");

    //Add
    let mut index = repo.index().expect("cannot access index");
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None).expect("fail to add to index");
    index.write().expect("failed to write to index");

    //Commit
    let commit_tree = index.write_tree().expect("Failed to write a tree");
    let sig = Signature::now("Ramus_Git", "_@_").unwrap();
    let head = repo.head().unwrap().target().unwrap();
    let parent = repo.find_commit(head).expect("Failed to find you father LOL");
    repo.commit(Some("HEAD"), &sig, &sig, "Ramus_Git Commit", &repo.find_tree(commit_tree).unwrap(),
        &[&parent]).expect("commit failed");

    //Push
    let mut push_conf = PushOptions::new();
    let mut cbs2 =RemoteCallbacks::new();
    cbs2.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(token.as_str(),token.as_str())
    });
    push_conf.remote_callbacks(cbs2); 
    repo.find_remote("origin").unwrap().push(&["+refs/heads/main","refs/remotes/origin/main"],Some(&mut push_conf)).expect("Push failed");
}


fn main() {
    let app = app::App::default();
    app::set_background_color(20, 20, 20);
    app::background2(20, 20, 20);
    app::set_background2_color(20, 20, 20);
    let mut wind = Window::new(100, 100, 400, 300, "Ramus_Git");
    let flex = Flex::default().with_size(300, 300).column().center_of_parent();
    let _label_new = Frame::default().with_label("Enter Repository Url");
    let input_new = Input::default();
    let mut new_btn = Button::default().with_label("Create Repository");
    let _label_sync = Frame::default().with_label("Enter Github Token");
    let input_sync = Input::default();
    let mut sync_btn = Button::default().with_label("Sync to Repository");

    flex.end();
    wind.set_color(Color::from_hex_str("#413932").unwrap());
    new_btn.set_color(Color::from_hex_str("#f54d27").unwrap());
    sync_btn.set_color(Color::from_hex_str("#f54d27").unwrap());

    wind.end();
    wind.show();
    
    //events
    new_btn.set_callback(move |_btn| {
        let path = env::current_dir();
        make_new_repo(&path.unwrap(), &input_new.value())
    });
    
    sync_btn.set_callback(move |_btn| {
        let path = env::current_dir();
        sync_to_repo(&path.unwrap(), &input_sync.value())
    });
    app.run().unwrap();
}
