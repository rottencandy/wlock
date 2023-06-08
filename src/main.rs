use wayland_client::{protocol::{wl_registry, wl_compositor, wl_subcompositor, wl_shm, wl_seat, wl_keyboard, wl_pointer, wl_output, wl_surface, wl_subsurface}, Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::ext::session_lock::v1::client::{ext_session_lock_manager_v1, ext_session_lock_v1, ext_session_lock_surface_v1};

fn main() -> () {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut app_data = AppData {
        locked: false,
        running: false,
        compositor: None,
        base_surface: None,
        seat: None,
        seat_ptr: None,
        seat_kb: None,
        subcompositor: None,
        shm: None,
        output: None,
        lock_mgr: None,
        lock_surf: None,

        width: 0,
        height: 0,
    };
    event_queue.roundtrip(&mut app_data).unwrap();

    if app_data.compositor.is_none() {
        panic!("compositor protocol missing!");
    }
    if app_data.seat.is_none() {
        panic!("seat protocol missing!");
    }
    if app_data.shm.is_none() {
        panic!("shm protocol missing!");
    }
    if app_data.lock_mgr.is_none() {
        panic!("lock_manager protocol missing!");
    }

    let lock = app_data.lock_mgr.as_ref().unwrap().lock(&qh, ());
    event_queue.roundtrip(&mut app_data).unwrap();

    //println!("Sleeping...");
    //thread::sleep(Duration::from_millis(4000));

    let surface = app_data.compositor.as_ref().unwrap().create_surface(&qh, ());
    let child = app_data.compositor.as_ref().unwrap().create_surface(&qh, ());
    let subsurface = app_data.subcompositor.as_ref().unwrap().get_subsurface(&child, &&surface, &qh, ());
    subsurface.set_sync();
    let lock_surf = lock.get_lock_surface(&surface, app_data.output.as_ref().unwrap(), &qh, ());
    app_data.base_surface = Some(surface);
    app_data.lock_surf = Some(lock_surf);
    event_queue.roundtrip(&mut app_data).unwrap();

    app_data.running = true;
    while app_data.locked {
        event_queue.blocking_dispatch(&mut app_data).unwrap();
    }

    lock.unlock_and_destroy();
    event_queue.roundtrip(&mut app_data).unwrap();
}

struct AppData {
    locked: bool,
    running: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    base_surface: Option<wl_surface::WlSurface>,
    seat: Option<wl_seat::WlSeat>,
    seat_ptr: Option<wl_pointer::WlPointer>,
    seat_kb: Option<wl_keyboard::WlKeyboard>,
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    shm: Option<wl_shm::WlShm>,
    output: Option<wl_output::WlOutput>,
    lock_mgr: Option<ext_session_lock_manager_v1::ExtSessionLockManagerV1>,
    lock_surf: Option<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1>,

    width: u32,
    height: u32,
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
                    let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ());
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
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 1, qh, ());
                    state.output = Some(output);
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
        state: &mut Self,
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

        if let wl_keyboard::Event::Keymap { format, .. } = event {
            if let WEnum::Value(wl_keyboard::KeymapFormat::XkbV1) = format {
            } else {
                panic!("Unknown keymap format!");
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
        // no event
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
    }
}

impl Dispatch<wl_subsurface::WlSubsurface, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_subsurface::WlSubsurface,
        _: wl_subsurface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
        ) {
        // todo
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

impl Dispatch<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, ()> for AppData {
    fn event(
        state: &mut Self,
        lock_surf: &ext_session_lock_surface_v1::ExtSessionLockSurfaceV1,
        event: ext_session_lock_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
        ) {
        if let ext_session_lock_surface_v1::Event::Configure { serial, width, height } = event {
            state.width = width;
            state.height = height;
            lock_surf.ack_configure(serial);
        }
    }
}
