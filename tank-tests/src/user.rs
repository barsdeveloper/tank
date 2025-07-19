use std::{collections::BTreeMap, sync::LazyLock};
use tank::{
    Connection, Entity, Error, Passive, expr,
    stream::{StreamExt, TryStreamExt},
};
use time::macros::datetime;
use tokio::sync::Mutex;
use uuid::Uuid;

static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
#[derive(Entity, Debug, Clone)]
#[tank(schema = "testing", name = "user_profiles")]
pub struct UserProfile {
    #[tank(primary_key, name = "user_id")]
    pub id: Passive<Uuid>,
    #[tank(unique)]
    pub username: String,
    #[tank(unique)]
    pub email: String,
    pub full_name: Option<String>,
    #[tank(default = "0")]
    pub follower_count: u32,
    pub is_active: bool,
    pub last_login: Option<time::PrimitiveDateTime>,
    pub preferences: Option<BTreeMap<String, String>>,
}

pub async fn users<C: Connection>(connection: &mut C) {
    let _lock = MUTEX.lock().await;

    // 1. SETUP: Clean up any previous state and create a fresh table.
    let result = UserProfile::drop_table(connection, true, false).await;
    assert!(
        result.is_ok(),
        "Failed to UserProfile::drop_table: {:?}",
        result.unwrap_err()
    );

    let result = UserProfile::create_table(connection, false, true).await;
    assert!(
        result.is_ok(),
        "Failed to UserProfile::create_table: {:?}",
        result.unwrap_err()
    );

    // 2. CREATE: Prepare and insert a batch of user profiles.
    let users_to_create = vec![
        UserProfile {
            id: Uuid::parse_str("a1a1a1a1-a1a1-a1a1-a1a1-a1a1a1a1a1a1")
                .unwrap()
                .into(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            full_name: Some("Alice Wonderland".into()),
            follower_count: 1200,
            is_active: true,
            last_login: Some(datetime!(2025-07-15 10:00:00)),
            preferences: Some(BTreeMap::from_iter([("theme".into(), "dark".into())])),
        },
        UserProfile {
            id: Uuid::parse_str("b2b2b2b2-b2b2-b2b2-b2b2-b2b2b2b2b2b2")
                .unwrap()
                .into(),
            username: "bob".into(),
            email: "bob@example.com".into(),
            full_name: Some("Bob Builder".into()),
            follower_count: 99,
            is_active: false,
            last_login: None,
            preferences: Some(BTreeMap::from_iter([("theme".into(), "light".into())])),
        },
        UserProfile {
            id: Uuid::parse_str("c3c3c3c3-c3c3-c3c3-c3c3-c3c3c3c3c3c3")
                .unwrap()
                .into(),
            username: "charlie".into(),
            email: "charlie@example.com".into(),
            full_name: None,
            follower_count: 5000,
            is_active: true,
            last_login: Some(datetime!(2025-07-16 11:30:00)),
            preferences: None,
        },
        UserProfile {
            id: Uuid::parse_str("d4d4d4d4-d4d4-d4d4-d4d4-d4d4d4d4d4d4")
                .unwrap()
                .into(),
            username: "diana".into(),
            email: "diana@example.com".into(),
            full_name: Some("Diana Prince".into()),
            follower_count: 15000,
            is_active: true,
            last_login: None,
            preferences: Some(BTreeMap::from_iter([(
                "notifications".into(),
                "off".into(),
            )])),
        },
        UserProfile {
            id: Uuid::parse_str("e5e5e5e5-e5e5-e5e5-e5e5-e5e5e5e5e5e5")
                .unwrap()
                .into(),
            username: "eve".into(),
            email: "eve@example.com".into(),
            full_name: Some("Eve".into()),
            follower_count: 1,
            is_active: false,
            last_login: Some(datetime!(2024-01-01 00:00:00)),
            preferences: None,
        },
    ];

    let result = UserProfile::insert_many(connection, users_to_create.iter()).await;
    assert!(
        result.is_ok(),
        "Failed to insert users: {:?}",
        result.unwrap_err()
    );
    assert_eq!(result.unwrap().rows_affected, 5);

    // 3. READ: Test various filtering conditions.
    // Find active users (should be 3: alice, charlie, diana)
    let active_users = UserProfile::find_many(connection, &expr!(is_active), None)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(active_users.len(), 3);

    // Find users with more than 1000 followers (should be 3: alice, charlie, diana)
    let popular_users = UserProfile::find_many(connection, &expr!(follower_count > 1000), None)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(popular_users.len(), 3);

    // 4. UPDATE: Find a user, modify them, and save.
    let mut bob = UserProfile::find_many(connection, &expr!(username == "bob"), None)
        .try_collect::<Vec<_>>()
        .await
        .and_then(|v| {
            if v.len() == 1 {
                Ok(v)
            } else {
                Err(Error::msg("The result does not have 1 row"))
            }
        })
        .expect("Query for Bob failed")
        .into_iter()
        .next()
        .unwrap();

    bob.is_active = true;
    bob.full_name = Some("Robert Builder".into());
    bob.last_login = Some(datetime!(2025-07-17 20:00:00));

    let result = bob.save(connection).await;
    assert!(
        result.is_ok(),
        "Failed to save Bob: {:?}",
        result.unwrap_err()
    );

    // Verify Bob's update
    let updated_bob = UserProfile::find_pk(connection, &bob.primary_key())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_bob.is_active, true);
    assert_eq!(updated_bob.full_name, Some("Robert Builder".into()));
    assert!(updated_bob.last_login.is_some());

    // Now there should be 4 active users
    let active_users_after_update = UserProfile::find_many(connection, &expr!(is_active), None)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(active_users_after_update.len(), 4);

    // 5. DELETE (Single): Find a user and delete them.
    let eve = UserProfile::find_one(connection, &expr!(username == "eve"))
        .await
        .unwrap()
        .unwrap();
    let result = eve.delete(connection).await;
    assert!(
        result.is_ok(),
        "Failed to delete Eve: {:?}",
        result.unwrap_err()
    );

    // Verify Eve is gone
    let maybe_eve = UserProfile::find_pk(connection, &eve.primary_key())
        .await
        .unwrap();
    assert!(maybe_eve.is_none(), "Eve should have been deleted");
    let total_users = UserProfile::find_many(connection, &true, None)
        .count()
        .await;
    assert_eq!(total_users, 4, "There should be 4 users remaining");

    // 6. DELETE (Batch): Delete users based on a filter.
    // Delete all users who have not logged in (only Diana remains)
    let result = UserProfile::delete_many(connection, &expr!(last_login IS NULL)).await;
    assert!(
        result.is_ok(),
        "Failed to batch delete users: {:?}",
        result.unwrap_err()
    );
    assert_eq!(result.unwrap().rows_affected, 1);

    // Verify only 3 users are left (alice, bob, charlie)
    let final_users = UserProfile::find_many(connection, &true, None)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(final_users.len(), 3);

    let final_usernames = final_users
        .into_iter()
        .map(|u| u.username)
        .collect::<std::collections::HashSet<_>>();
    let expected_usernames =
        std::collections::HashSet::from_iter(["alice".into(), "bob".into(), "charlie".into()]);
    assert_eq!(final_usernames, expected_usernames);
}
