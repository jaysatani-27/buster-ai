use std::time::SystemTime;

use sentry::{capture_event, protocol::Event, User};
use uuid::Uuid;

pub fn send_sentry_error(message: &String, user_id: Option<&Uuid>) {
    let user: Option<User> = match user_id {
        Some(id) => Some(User {
            id: Some(id.to_string()),
            ..Default::default()
        }),
        None => None,
    };

    let event = Event {
        message: Some(message.to_string()),
        level: sentry::Level::Error,
        event_id: Uuid::new_v4(),
        timestamp: SystemTime::now(),
        user,
        ..Default::default()
    };
    capture_event(event);
}
