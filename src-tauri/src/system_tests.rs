//! Checks against the real Antigravity installation on this machine.
//!
//! Run with `cargo test -- --ignored --nocapture` on a machine where
//! Antigravity is installed and signed in. They only read: nothing is
//! written to the keychain and no process is stopped. Output never includes
//! tokens or secrets.

use crate::antigravity::install::Install;
use crate::antigravity::{live, oauth_client, process};
use crate::google::{USER_AGENT, cloudcode, oauth};

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap()
}

fn mask(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) => format!("{}…@{domain}", user.chars().take(2).collect::<String>()),
        None => "?".into(),
    }
}

#[tokio::test]
#[ignore = "needs a local Antigravity installation"]
async fn reads_installation_sign_in_and_quota() {
    let install = Install::locate().expect("Antigravity installation");
    println!("install: {}", install.root.display());
    println!("processes: {:?}", process::scan(Some(&install)));

    let http = http();
    let client = oauth_client::discover(&http, &install)
        .await
        .expect("OAuth client");
    println!("client: {client:?}");

    let bundle = live::read().unwrap().expect("Antigravity is signed in");
    let claims = bundle.claims().expect("ID token claims");
    println!(
        "signed in: {} (auth_method {:?})",
        mask(claims.email.as_deref().unwrap_or("")),
        bundle.auth_method
    );

    let grant = oauth::refresh(&http, &client, &bundle.token.refresh_token)
        .await
        .expect("refresh");
    println!(
        "refresh: expires_in={} id_token={}",
        grant.expires_in,
        grant.id_token.is_some()
    );
    if let Some(claims) = grant.id_token.as_deref().and_then(oauth::IdClaims::parse) {
        assert_eq!(claims.sub, bundle.claims().unwrap().sub);
    }

    let groups = cloudcode::quota(&http, &grant.access_token)
        .await
        .expect("quota");
    for group in &groups {
        let pct =
            |w: &Option<cloudcode::QuotaWindow>| w.as_ref().map(|w| (w.remaining * 100.0).round());
        println!(
            "quota {:?}: 5h={:?}% weekly={:?}%",
            group.key,
            pct(&group.five_hour),
            pct(&group.weekly)
        );
    }
    assert!(!groups.is_empty());
    println!(
        "plan: {:?}",
        cloudcode::plan(&http, &grant.access_token)
            .await
            .expect("plan")
    );
}
