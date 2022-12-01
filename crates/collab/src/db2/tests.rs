use super::*;
use gpui::executor::{Background, Deterministic};
use std::sync::Arc;

macro_rules! test_both_dbs {
    ($postgres_test_name:ident, $sqlite_test_name:ident, $db:ident, $body:block) => {
        #[gpui::test]
        async fn $postgres_test_name() {
            let test_db = TestDb::postgres(Deterministic::new(0).build_background());
            let $db = test_db.db();
            $body
        }

        #[gpui::test]
        async fn $sqlite_test_name() {
            let test_db = TestDb::sqlite(Deterministic::new(0).build_background());
            let $db = test_db.db();
            $body
        }
    };
}

test_both_dbs!(
    test_get_users_by_ids_postgres,
    test_get_users_by_ids_sqlite,
    db,
    {
        let mut user_ids = Vec::new();
        let mut user_metric_ids = Vec::new();
        for i in 1..=4 {
            let user = db
                .create_user(
                    &format!("user{i}@example.com"),
                    false,
                    NewUserParams {
                        github_login: format!("user{i}"),
                        github_user_id: i,
                        invite_count: 0,
                    },
                )
                .await
                .unwrap();
            user_ids.push(user.user_id);
            user_metric_ids.push(user.metrics_id);
        }

        assert_eq!(
            db.get_users_by_ids(user_ids.clone()).await.unwrap(),
            vec![
                User {
                    id: user_ids[0],
                    github_login: "user1".to_string(),
                    github_user_id: Some(1),
                    email_address: Some("user1@example.com".to_string()),
                    admin: false,
                    metrics_id: user_metric_ids[0].parse().unwrap(),
                    ..Default::default()
                },
                User {
                    id: user_ids[1],
                    github_login: "user2".to_string(),
                    github_user_id: Some(2),
                    email_address: Some("user2@example.com".to_string()),
                    admin: false,
                    metrics_id: user_metric_ids[1].parse().unwrap(),
                    ..Default::default()
                },
                User {
                    id: user_ids[2],
                    github_login: "user3".to_string(),
                    github_user_id: Some(3),
                    email_address: Some("user3@example.com".to_string()),
                    admin: false,
                    metrics_id: user_metric_ids[2].parse().unwrap(),
                    ..Default::default()
                },
                User {
                    id: user_ids[3],
                    github_login: "user4".to_string(),
                    github_user_id: Some(4),
                    email_address: Some("user4@example.com".to_string()),
                    admin: false,
                    metrics_id: user_metric_ids[3].parse().unwrap(),
                    ..Default::default()
                }
            ]
        );
    }
);

test_both_dbs!(
    test_get_user_by_github_account_postgres,
    test_get_user_by_github_account_sqlite,
    db,
    {
        let user_id1 = db
            .create_user(
                "user1@example.com",
                false,
                NewUserParams {
                    github_login: "login1".into(),
                    github_user_id: 101,
                    invite_count: 0,
                },
            )
            .await
            .unwrap()
            .user_id;
        let user_id2 = db
            .create_user(
                "user2@example.com",
                false,
                NewUserParams {
                    github_login: "login2".into(),
                    github_user_id: 102,
                    invite_count: 0,
                },
            )
            .await
            .unwrap()
            .user_id;

        let user = db
            .get_user_by_github_account("login1", None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id, user_id1);
        assert_eq!(&user.github_login, "login1");
        assert_eq!(user.github_user_id, Some(101));

        assert!(db
            .get_user_by_github_account("non-existent-login", None)
            .await
            .unwrap()
            .is_none());

        let user = db
            .get_user_by_github_account("the-new-login2", Some(102))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id, user_id2);
        assert_eq!(&user.github_login, "the-new-login2");
        assert_eq!(user.github_user_id, Some(102));
    }
);

test_both_dbs!(
    test_create_access_tokens_postgres,
    test_create_access_tokens_sqlite,
    db,
    {
        let user = db
            .create_user(
                "u1@example.com",
                false,
                NewUserParams {
                    github_login: "u1".into(),
                    github_user_id: 1,
                    invite_count: 0,
                },
            )
            .await
            .unwrap()
            .user_id;

        db.create_access_token_hash(user, "h1", 3).await.unwrap();
        db.create_access_token_hash(user, "h2", 3).await.unwrap();
        assert_eq!(
            db.get_access_token_hashes(user).await.unwrap(),
            &["h2".to_string(), "h1".to_string()]
        );

        db.create_access_token_hash(user, "h3", 3).await.unwrap();
        assert_eq!(
            db.get_access_token_hashes(user).await.unwrap(),
            &["h3".to_string(), "h2".to_string(), "h1".to_string(),]
        );

        db.create_access_token_hash(user, "h4", 3).await.unwrap();
        assert_eq!(
            db.get_access_token_hashes(user).await.unwrap(),
            &["h4".to_string(), "h3".to_string(), "h2".to_string(),]
        );

        db.create_access_token_hash(user, "h5", 3).await.unwrap();
        assert_eq!(
            db.get_access_token_hashes(user).await.unwrap(),
            &["h5".to_string(), "h4".to_string(), "h3".to_string()]
        );
    }
);

test_both_dbs!(test_add_contacts_postgres, test_add_contacts_sqlite, db, {
    let mut user_ids = Vec::new();
    for i in 0..3 {
        user_ids.push(
            db.create_user(
                &format!("user{i}@example.com"),
                false,
                NewUserParams {
                    github_login: format!("user{i}"),
                    github_user_id: i,
                    invite_count: 0,
                },
            )
            .await
            .unwrap()
            .user_id,
        );
    }

    let user_1 = user_ids[0];
    let user_2 = user_ids[1];
    let user_3 = user_ids[2];

    // User starts with no contacts
    assert_eq!(db.get_contacts(user_1).await.unwrap(), &[]);

    // User requests a contact. Both users see the pending request.
    db.send_contact_request(user_1, user_2).await.unwrap();
    assert!(!db.has_contact(user_1, user_2).await.unwrap());
    assert!(!db.has_contact(user_2, user_1).await.unwrap());
    assert_eq!(
        db.get_contacts(user_1).await.unwrap(),
        &[Contact::Outgoing { user_id: user_2 }],
    );
    assert_eq!(
        db.get_contacts(user_2).await.unwrap(),
        &[Contact::Incoming {
            user_id: user_1,
            should_notify: true
        }]
    );

    // User 2 dismisses the contact request notification without accepting or rejecting.
    // We shouldn't notify them again.
    db.dismiss_contact_notification(user_1, user_2)
        .await
        .unwrap_err();
    db.dismiss_contact_notification(user_2, user_1)
        .await
        .unwrap();
    assert_eq!(
        db.get_contacts(user_2).await.unwrap(),
        &[Contact::Incoming {
            user_id: user_1,
            should_notify: false
        }]
    );

    // User can't accept their own contact request
    db.respond_to_contact_request(user_1, user_2, true)
        .await
        .unwrap_err();

    // User accepts a contact request. Both users see the contact.
    db.respond_to_contact_request(user_2, user_1, true)
        .await
        .unwrap();
    assert_eq!(
        db.get_contacts(user_1).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_2,
            should_notify: true,
            busy: false,
        }],
    );
    assert!(db.has_contact(user_1, user_2).await.unwrap());
    assert!(db.has_contact(user_2, user_1).await.unwrap());
    assert_eq!(
        db.get_contacts(user_2).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_1,
            should_notify: false,
            busy: false,
        }]
    );

    // Users cannot re-request existing contacts.
    db.send_contact_request(user_1, user_2).await.unwrap_err();
    db.send_contact_request(user_2, user_1).await.unwrap_err();

    // Users can't dismiss notifications of them accepting other users' requests.
    db.dismiss_contact_notification(user_2, user_1)
        .await
        .unwrap_err();
    assert_eq!(
        db.get_contacts(user_1).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_2,
            should_notify: true,
            busy: false,
        }]
    );

    // Users can dismiss notifications of other users accepting their requests.
    db.dismiss_contact_notification(user_1, user_2)
        .await
        .unwrap();
    assert_eq!(
        db.get_contacts(user_1).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_2,
            should_notify: false,
            busy: false,
        }]
    );

    // Users send each other concurrent contact requests and
    // see that they are immediately accepted.
    db.send_contact_request(user_1, user_3).await.unwrap();
    db.send_contact_request(user_3, user_1).await.unwrap();
    assert_eq!(
        db.get_contacts(user_1).await.unwrap(),
        &[
            Contact::Accepted {
                user_id: user_2,
                should_notify: false,
                busy: false,
            },
            Contact::Accepted {
                user_id: user_3,
                should_notify: false,
                busy: false,
            }
        ]
    );
    assert_eq!(
        db.get_contacts(user_3).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_1,
            should_notify: false,
            busy: false,
        }],
    );

    // User declines a contact request. Both users see that it is gone.
    db.send_contact_request(user_2, user_3).await.unwrap();
    db.respond_to_contact_request(user_3, user_2, false)
        .await
        .unwrap();
    assert!(!db.has_contact(user_2, user_3).await.unwrap());
    assert!(!db.has_contact(user_3, user_2).await.unwrap());
    assert_eq!(
        db.get_contacts(user_2).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_1,
            should_notify: false,
            busy: false,
        }]
    );
    assert_eq!(
        db.get_contacts(user_3).await.unwrap(),
        &[Contact::Accepted {
            user_id: user_1,
            should_notify: false,
            busy: false,
        }],
    );
});

test_both_dbs!(test_metrics_id_postgres, test_metrics_id_sqlite, db, {
    let NewUserResult {
        user_id: user1,
        metrics_id: metrics_id1,
        ..
    } = db
        .create_user(
            "person1@example.com",
            false,
            NewUserParams {
                github_login: "person1".into(),
                github_user_id: 101,
                invite_count: 5,
            },
        )
        .await
        .unwrap();
    let NewUserResult {
        user_id: user2,
        metrics_id: metrics_id2,
        ..
    } = db
        .create_user(
            "person2@example.com",
            false,
            NewUserParams {
                github_login: "person2".into(),
                github_user_id: 102,
                invite_count: 5,
            },
        )
        .await
        .unwrap();

    assert_eq!(db.get_user_metrics_id(user1).await.unwrap(), metrics_id1);
    assert_eq!(db.get_user_metrics_id(user2).await.unwrap(), metrics_id2);
    assert_eq!(metrics_id1.len(), 36);
    assert_eq!(metrics_id2.len(), 36);
    assert_ne!(metrics_id1, metrics_id2);
});

#[test]
fn test_fuzzy_like_string() {
    assert_eq!(Database::fuzzy_like_string("abcd"), "%a%b%c%d%");
    assert_eq!(Database::fuzzy_like_string("x y"), "%x%y%");
    assert_eq!(Database::fuzzy_like_string(" z  "), "%z%");
}

#[gpui::test]
async fn test_fuzzy_search_users() {
    let test_db = TestDb::postgres(build_background_executor());
    let db = test_db.db();
    for (i, github_login) in [
        "California",
        "colorado",
        "oregon",
        "washington",
        "florida",
        "delaware",
        "rhode-island",
    ]
    .into_iter()
    .enumerate()
    {
        db.create_user(
            &format!("{github_login}@example.com"),
            false,
            NewUserParams {
                github_login: github_login.into(),
                github_user_id: i as i32,
                invite_count: 0,
            },
        )
        .await
        .unwrap();
    }

    assert_eq!(
        fuzzy_search_user_names(db, "clr").await,
        &["colorado", "California"]
    );
    assert_eq!(
        fuzzy_search_user_names(db, "ro").await,
        &["rhode-island", "colorado", "oregon"],
    );

    async fn fuzzy_search_user_names(db: &Database, query: &str) -> Vec<String> {
        db.fuzzy_search_users(query, 10)
            .await
            .unwrap()
            .into_iter()
            .map(|user| user.github_login)
            .collect::<Vec<_>>()
    }
}

#[gpui::test]
async fn test_invite_codes() {
    let test_db = TestDb::postgres(build_background_executor());
    let db = test_db.db();

    let NewUserResult { user_id: user1, .. } = db
        .create_user(
            "user1@example.com",
            false,
            NewUserParams {
                github_login: "user1".into(),
                github_user_id: 0,
                invite_count: 0,
            },
        )
        .await
        .unwrap();

    // Initially, user 1 has no invite code
    assert_eq!(db.get_invite_code_for_user(user1).await.unwrap(), None);

    // Setting invite count to 0 when no code is assigned does not assign a new code
    db.set_invite_count_for_user(user1, 0).await.unwrap();
    assert!(db.get_invite_code_for_user(user1).await.unwrap().is_none());

    // User 1 creates an invite code that can be used twice.
    db.set_invite_count_for_user(user1, 2).await.unwrap();
    let (invite_code, invite_count) = db.get_invite_code_for_user(user1).await.unwrap().unwrap();
    assert_eq!(invite_count, 2);

    // User 2 redeems the invite code and becomes a contact of user 1.
    let user2_invite = db
        .create_invite_from_code(&invite_code, "user2@example.com", Some("user-2-device-id"))
        .await
        .unwrap();
    let NewUserResult {
        user_id: user2,
        inviting_user_id,
        signup_device_id,
        metrics_id,
    } = db
        .create_user_from_invite(
            &user2_invite,
            NewUserParams {
                github_login: "user2".into(),
                github_user_id: 2,
                invite_count: 7,
            },
        )
        .await
        .unwrap()
        .unwrap();
    let (_, invite_count) = db.get_invite_code_for_user(user1).await.unwrap().unwrap();
    assert_eq!(invite_count, 1);
    assert_eq!(inviting_user_id, Some(user1));
    assert_eq!(signup_device_id.unwrap(), "user-2-device-id");
    assert_eq!(db.get_user_metrics_id(user2).await.unwrap(), metrics_id);
    assert_eq!(
        db.get_contacts(user1).await.unwrap(),
        [Contact::Accepted {
            user_id: user2,
            should_notify: true,
            busy: false,
        }]
    );
    assert_eq!(
        db.get_contacts(user2).await.unwrap(),
        [Contact::Accepted {
            user_id: user1,
            should_notify: false,
            busy: false,
        }]
    );
    assert_eq!(
        db.get_invite_code_for_user(user2).await.unwrap().unwrap().1,
        7
    );

    // User 3 redeems the invite code and becomes a contact of user 1.
    let user3_invite = db
        .create_invite_from_code(&invite_code, "user3@example.com", None)
        .await
        .unwrap();
    let NewUserResult {
        user_id: user3,
        inviting_user_id,
        signup_device_id,
        ..
    } = db
        .create_user_from_invite(
            &user3_invite,
            NewUserParams {
                github_login: "user-3".into(),
                github_user_id: 3,
                invite_count: 3,
            },
        )
        .await
        .unwrap()
        .unwrap();
    let (_, invite_count) = db.get_invite_code_for_user(user1).await.unwrap().unwrap();
    assert_eq!(invite_count, 0);
    assert_eq!(inviting_user_id, Some(user1));
    assert!(signup_device_id.is_none());
    assert_eq!(
        db.get_contacts(user1).await.unwrap(),
        [
            Contact::Accepted {
                user_id: user2,
                should_notify: true,
                busy: false,
            },
            Contact::Accepted {
                user_id: user3,
                should_notify: true,
                busy: false,
            }
        ]
    );
    assert_eq!(
        db.get_contacts(user3).await.unwrap(),
        [Contact::Accepted {
            user_id: user1,
            should_notify: false,
            busy: false,
        }]
    );
    assert_eq!(
        db.get_invite_code_for_user(user3).await.unwrap().unwrap().1,
        3
    );

    // Trying to reedem the code for the third time results in an error.
    db.create_invite_from_code(&invite_code, "user4@example.com", Some("user-4-device-id"))
        .await
        .unwrap_err();

    // Invite count can be updated after the code has been created.
    db.set_invite_count_for_user(user1, 2).await.unwrap();
    let (latest_code, invite_count) = db.get_invite_code_for_user(user1).await.unwrap().unwrap();
    assert_eq!(latest_code, invite_code); // Invite code doesn't change when we increment above 0
    assert_eq!(invite_count, 2);

    // User 4 can now redeem the invite code and becomes a contact of user 1.
    let user4_invite = db
        .create_invite_from_code(&invite_code, "user4@example.com", Some("user-4-device-id"))
        .await
        .unwrap();
    let user4 = db
        .create_user_from_invite(
            &user4_invite,
            NewUserParams {
                github_login: "user-4".into(),
                github_user_id: 4,
                invite_count: 5,
            },
        )
        .await
        .unwrap()
        .unwrap()
        .user_id;

    let (_, invite_count) = db.get_invite_code_for_user(user1).await.unwrap().unwrap();
    assert_eq!(invite_count, 1);
    assert_eq!(
        db.get_contacts(user1).await.unwrap(),
        [
            Contact::Accepted {
                user_id: user2,
                should_notify: true,
                busy: false,
            },
            Contact::Accepted {
                user_id: user3,
                should_notify: true,
                busy: false,
            },
            Contact::Accepted {
                user_id: user4,
                should_notify: true,
                busy: false,
            }
        ]
    );
    assert_eq!(
        db.get_contacts(user4).await.unwrap(),
        [Contact::Accepted {
            user_id: user1,
            should_notify: false,
            busy: false,
        }]
    );
    assert_eq!(
        db.get_invite_code_for_user(user4).await.unwrap().unwrap().1,
        5
    );

    // An existing user cannot redeem invite codes.
    db.create_invite_from_code(&invite_code, "user2@example.com", Some("user-2-device-id"))
        .await
        .unwrap_err();
    let (_, invite_count) = db.get_invite_code_for_user(user1).await.unwrap().unwrap();
    assert_eq!(invite_count, 1);
}

// #[gpui::test]
// async fn test_signups() {
//     let test_db = PostgresTestDb::new(build_background_executor());
//     let db = test_db.db();

//     // people sign up on the waitlist
//     for i in 0..8 {
//         db.create_signup(Signup {
//             email_address: format!("person-{i}@example.com"),
//             platform_mac: true,
//             platform_linux: i % 2 == 0,
//             platform_windows: i % 4 == 0,
//             editor_features: vec!["speed".into()],
//             programming_languages: vec!["rust".into(), "c".into()],
//             device_id: Some(format!("device_id_{i}")),
//         })
//         .await
//         .unwrap();
//     }

//     assert_eq!(
//         db.get_waitlist_summary().await.unwrap(),
//         WaitlistSummary {
//             count: 8,
//             mac_count: 8,
//             linux_count: 4,
//             windows_count: 2,
//             unknown_count: 0,
//         }
//     );

//     // retrieve the next batch of signup emails to send
//     let signups_batch1 = db.get_unsent_invites(3).await.unwrap();
//     let addresses = signups_batch1
//         .iter()
//         .map(|s| &s.email_address)
//         .collect::<Vec<_>>();
//     assert_eq!(
//         addresses,
//         &[
//             "person-0@example.com",
//             "person-1@example.com",
//             "person-2@example.com"
//         ]
//     );
//     assert_ne!(
//         signups_batch1[0].email_confirmation_code,
//         signups_batch1[1].email_confirmation_code
//     );

//     // the waitlist isn't updated until we record that the emails
//     // were successfully sent.
//     let signups_batch = db.get_unsent_invites(3).await.unwrap();
//     assert_eq!(signups_batch, signups_batch1);

//     // once the emails go out, we can retrieve the next batch
//     // of signups.
//     db.record_sent_invites(&signups_batch1).await.unwrap();
//     let signups_batch2 = db.get_unsent_invites(3).await.unwrap();
//     let addresses = signups_batch2
//         .iter()
//         .map(|s| &s.email_address)
//         .collect::<Vec<_>>();
//     assert_eq!(
//         addresses,
//         &[
//             "person-3@example.com",
//             "person-4@example.com",
//             "person-5@example.com"
//         ]
//     );

//     // the sent invites are excluded from the summary.
//     assert_eq!(
//         db.get_waitlist_summary().await.unwrap(),
//         WaitlistSummary {
//             count: 5,
//             mac_count: 5,
//             linux_count: 2,
//             windows_count: 1,
//             unknown_count: 0,
//         }
//     );

//     // user completes the signup process by providing their
//     // github account.
//     let NewUserResult {
//         user_id,
//         inviting_user_id,
//         signup_device_id,
//         ..
//     } = db
//         .create_user_from_invite(
//             &Invite {
//                 email_address: signups_batch1[0].email_address.clone(),
//                 email_confirmation_code: signups_batch1[0].email_confirmation_code.clone(),
//             },
//             NewUserParams {
//                 github_login: "person-0".into(),
//                 github_user_id: 0,
//                 invite_count: 5,
//             },
//         )
//         .await
//         .unwrap()
//         .unwrap();
//     let user = db.get_user_by_id(user_id).await.unwrap().unwrap();
//     assert!(inviting_user_id.is_none());
//     assert_eq!(user.github_login, "person-0");
//     assert_eq!(user.email_address.as_deref(), Some("person-0@example.com"));
//     assert_eq!(user.invite_count, 5);
//     assert_eq!(signup_device_id.unwrap(), "device_id_0");

//     // cannot redeem the same signup again.
//     assert!(db
//         .create_user_from_invite(
//             &Invite {
//                 email_address: signups_batch1[0].email_address.clone(),
//                 email_confirmation_code: signups_batch1[0].email_confirmation_code.clone(),
//             },
//             NewUserParams {
//                 github_login: "some-other-github_account".into(),
//                 github_user_id: 1,
//                 invite_count: 5,
//             },
//         )
//         .await
//         .unwrap()
//         .is_none());

//     // cannot redeem a signup with the wrong confirmation code.
//     db.create_user_from_invite(
//         &Invite {
//             email_address: signups_batch1[1].email_address.clone(),
//             email_confirmation_code: "the-wrong-code".to_string(),
//         },
//         NewUserParams {
//             github_login: "person-1".into(),
//             github_user_id: 2,
//             invite_count: 5,
//         },
//     )
//     .await
//     .unwrap_err();
// }

fn build_background_executor() -> Arc<Background> {
    Deterministic::new(0).build_background()
}
