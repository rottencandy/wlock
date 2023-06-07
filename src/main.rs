use std::{thread, time::Duration};

use wayland_client::{protocol::{wl_registry, wl_compositor, wl_subcompositor, wl_shm, wl_seat, wl_keyboard, wl_pointer}, Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::ext::session_lock::v1::client::{ext_session_lock_manager_v1, ext_session_lock_v1};

fn main() -> () {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut app_data = AppData {
        locked: false,
        compositor: None,
        seat: None,
        seat_ptr: None,
        seat_kb: None,
        subcompositor: None,
        shm: None,
        lock_mgr: None,
    };
    event_queue.roundtrip(&mut app_data).unwrap();

    if app_data.compositor.is_none()
        || app_data.seat.is_none()
            || app_data.subcompositor.is_none()
            || app_data.shm.is_none()
            || app_data.lock_mgr.is_none() {
                panic!("Required protocols missing!");
            }

    let lock = app_data.lock_mgr.as_ref().unwrap().lock(&qh, ());
    event_queue.roundtrip(&mut app_data).unwrap();

    println!("Sleeping...");
    thread::sleep(Duration::from_millis(4000));
    println!("Attempting unlock.");
    lock.unlock_and_destroy();
    event_queue.roundtrip(&mut app_data).unwrap();
    println!("Successful!!!");
}

struct AppData {
    locked: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    seat: Option<wl_seat::WlSeat>,
    seat_ptr: Option<wl_pointer::WlPointer>,
    seat_kb: Option<wl_keyboard::WlKeyboard>,
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    shm: Option<wl_shm::WlShm>,
    lock_mgr: Option<ext_session_lock_manager_v1::ExtSessionLockManagerV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
        ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            ..
        } = event
        {
            // println!("[{}] {}", name, interface);
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ());
                    state.compositor = Some(compositor);
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                    state.seat = Some(seat);
                }
                "wl_subcompositor" => {
                    let subcompositor =
                        registry.bind::<wl_subcompositor::WlSubcompositor, _, _>(name, 1, qh, ());
                    state.subcompositor = Some(subcompositor);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());
                    state.shm = Some(shm);
                }
                "ext_session_lock_manager_v1" => {
                    let lock_mgr = registry.bind::<ext_session_lock_manager_v1::ExtSessionLockManagerV1, _, _>(name, 1, qh, ());
                    state.lock_mgr = Some(lock_mgr);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // no event
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for AppData {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
        ) {
        // no event
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
            ..
        } = event {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                state.seat_kb = Some(seat.get_keyboard(qh, ()));
            }
            if capabilities.contains(wl_seat::Capability::Pointer) {
                state.seat_ptr = Some(seat.get_pointer(qh, ()));
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                // todo
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_pointer::WlPointer,
        _: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // todo
    }
}

impl Dispatch<wl_subcompositor::WlSubcompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_subcompositor::WlSubcompositor,
        _: wl_subcompositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // no event
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // we ignore wl_shm events in this example
    }
}

impl Dispatch<ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ext_session_lock_manager_v1::ExtSessionLockManagerV1,
        _: ext_session_lock_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
        ) {
        // no event
    }
}

impl Dispatch<ext_session_lock_v1::ExtSessionLockV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &ext_session_lock_v1::ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
        ) {
        match event {
            ext_session_lock_v1::Event::Finished => {
                panic!("Unable to lock session!");
            }
            ext_session_lock_v1::Event::Locked => {
                state.locked = true;
                println!("Session successfully locked!!");

            }
            _ => {}
        }
    }
}