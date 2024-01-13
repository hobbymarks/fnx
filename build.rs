use git2::Repository;

fn git_hash() -> String {
    if let Ok(repo) = Repository::open(".") {
        if let Ok(head) = repo.head() {
            if let Some(oid) = head.target() {
                return oid.to_string().chars().take(8).collect();
            }
        }
    }

    "unknown".to_string()
}

fn main() {
    let git_hash = git_hash();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
