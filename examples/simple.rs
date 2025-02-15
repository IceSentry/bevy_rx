use std::time::Instant;

use bevy_app::{prelude::*, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;
use bevy_rx::prelude::*;

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::run_once(), ReactiveExtensionsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run()
}

#[derive(Debug, PartialEq)]
struct Button {
    active: bool,
}
impl Button {
    const OFF: Self = Button { active: false };
    const ON: Self = Button { active: true };
}

#[derive(Debug, PartialEq)]
struct Lock {
    unlocked: bool,
}

impl Lock {
    /// A lock will only unlock if both of its buttons are active
    fn two_buttons(buttons: (&Button, &Button)) -> Self {
        let unlocked = buttons.0.active && buttons.1.active;
        Self { unlocked }
    }
}

fn setup(mut commands: Commands, mut reactor: Reactor) {
    let button1 = reactor.add_signal(Button::OFF);
    let button2 = reactor.add_signal(Button::OFF);
    commands.spawn(reactor.add_derived((button1, button2), Lock::two_buttons));
    commands.spawn(button1);
    commands.spawn(button2);
}

fn update(mut reactor: Reactor, buttons: Query<&Signal<Button>>, lock: Query<&Derived<Lock>>) {
    // Signals and derivations from the ECS
    let mut buttons = buttons.iter().copied();
    let [button1, button2] = [buttons.next().unwrap(), buttons.next().unwrap()];
    let lock1 = *lock.single();
    reactor.send_signal(button1, Button::ON);

    let start = Instant::now();
    reactor.send_signal(button2, Button::ON);
    dbg!(start.elapsed());

    assert!(reactor.read(lock1).unlocked);

    let start = Instant::now();
    for _ in 0..1_000_000 {
        reactor.send_signal(button1, Button::ON); // diffing prevents triggering a recompute
    }
    dbg!(start.elapsed() / 1_000_000);

    let start = Instant::now();
    for i in 1..=1_000_000 {
        reactor.send_signal(button1, Button { active: i % 2 == 0 });
    }
    dbg!(start.elapsed() / 1_000_000);

    let button3 = reactor.add_signal(Button::OFF); // We can add a new signal locally
    let lock2 = reactor.add_derived((button1, button3), Lock::two_buttons); // Local and ECS signals
    reactor.send_signal(button3, Button::ON);
    let start = Instant::now();
    for _ in 0..1_000_000 {
        if !reactor.read(lock2).unlocked {
            dbg!("nope");
        }
    }
    dbg!(start.elapsed() / 1);

    assert!(reactor.read(lock2).unlocked);
}
