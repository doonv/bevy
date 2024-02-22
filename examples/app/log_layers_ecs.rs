//! This example illustrates how to transfer log events from the [`Layer`] to Bevy's ECS.
//!
//! The way we will do this is via a [`mpsc`] channel. [`mpsc`] channels allow 2 unrelated
//! parts of the program to communicate (in this case, [`Layer`]s and Bevy's ECS).
//!
//! Inside the [`update_subscriber`] function we will create a [`mpsc::Sender`] and a [`mpsc::Receiver`] from a
//! [`mpsc::channel`]. The [`Sender`](mpsc::Sender) will go into the [`AdvancedLayer`] and the [`Receiver`](mpsc::Receiver) will
//! go into a non-send resource called [`LogEvents`] (It has to be non-send because [`Receiver`](mpsc::Receiver) is [`!Sync`](Sync)).
//! From there we will use [`transfer_log_events`] to transfer log events from [`LogEvents`] to an ECS event called [`LogEvent`].
//!
//! Finally, after all that we can access the [`LogEvent`] event from our systems and use it.

use std::sync::mpsc;

use bevy::{
    log::tracing_subscriber::{self, layer::SubscriberExt, Layer},
    log::BoxedSubscriber,
    prelude::*,
    utils::tracing,
    utils::tracing::Subscriber,
};

/// A basic message. This is what we will be sending from the [`CaptureLayer`] to [`CapturedLogEvents`] non-send resource.
#[derive(Debug, Event)]
struct LogEvent {
    message: String,
}

/// This non-send resource temporarily stores [`LogEvent`]s before they are
/// written to [`Events<LogEvent>`] by [`transfer_log_events`].
#[derive(Deref, DerefMut)]
struct CapturedLogEvents(mpsc::Receiver<LogEvent>);

/// Transfers information from the [`LogEvents`] resource to [`Events<LogEvent>`](LogEvent).
fn transfer_log_events(reciever: NonSend<CapturedLogEvents>, mut log_events: EventWriter<LogEvent>) {
    // Make sure to use `try_iter()` and not `iter()` to prevent blocking.
    log_events.send_batch(reciever.try_iter());
}

/// This is the [`Layer`] that we will use to capture log events and then send them to Bevy's 
/// ECS via it's [`mpsc::Sender`].
struct CaptureLayer {
    sender: mpsc::Sender<LogEvent>,
}
impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // In order to obtain the log message, we have to create a struct that implements
        // Visit and holds a reference to our string. Then we use the `record` method and
        // the struct to modify the reference to hold the message string.
        let mut message = None;
        event.record(&mut CaptureLayerVisitor(&mut message));
        if let Some(message) = message {
            // You can obtain metadata like this, but we wont use it for this example.
            let _metadata = event.metadata();
            self.sender
                .send(LogEvent { message })
                .expect("LogEvents resource no longer exists!");
        }
    }
}

/// A [`Visit`](tracing::field::Visit)or that records log messages that are transfered to [`CaptureLayer`].
struct CaptureLayerVisitor<'a>(&'a mut Option<String>);
impl tracing::field::Visit for CaptureLayerVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // This if statement filters out unneeded events sometimes show up
        if field.name() == "message" {
            *self.0 = Some(format!("{value:?}"));
        }
    }
}
fn update_subscriber(app: &mut App, subscriber: BoxedSubscriber) -> BoxedSubscriber {
    let (sender, reciever) = mpsc::channel();

    let layer = CaptureLayer { sender };
    let resource = CapturedLogEvents(reciever);

    app.insert_non_send_resource(resource);
    app.add_event::<LogEvent>();
    app.add_systems(Update, transfer_log_events);

    Box::new(subscriber.with(layer))
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            update_subscriber: Some(update_subscriber),
            ..default()
        }))
        .add_systems(Update, (log_system, print_logs))
        .run();
}

fn log_system() {
    // here is how you write new logs at each "log level" (in "most important" to
    // "least important" order)
    error!("something failed");
    warn!("something bad happened that isn't a failure, but thats worth calling out");
    info!("helpful information that is worth printing by default");
    debug!("helpful for debugging");
    trace!("very noisy");
}

// This is how we can read our LogEvents.
// In this example, we're just printing it out, but you could parse or display the events.
fn print_logs(mut events: EventReader<LogEvent>) {
    for event in events.read() {
        dbg!(&event.message);
    }
}
