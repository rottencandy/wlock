use wayland_client::Connection;
use xkbcommon::xkb::Context;
mod app_data;

fn main() -> () {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut app_data = app_data::AppData {
        locked: false,
        running: false,
        compositor: None,
        seat: None,
        seat_ptr: None,
        seat_kb: None,
        subcompositor: None,
        shm: None,
        surfaces: vec![],
        lock_mgr: None,

        xkb_context: Context::new(0),
        xkb_keymap: None,
        xkb_state: None,

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

    app_data.running = true;
    event_queue.flush().unwrap();

    //println!("Sleeping...");
    //thread::sleep(Duration::from_millis(4000));

    for mut s in &mut app_data.surfaces {
        let surf = app_data.compositor.as_ref().unwrap().create_surface(&qh, ());
        let child = app_data.compositor.as_ref().unwrap().create_surface(&qh, ());
        let subsurface = app_data.subcompositor.as_ref().unwrap().get_subsurface(&child, &surf, &qh, ());
        subsurface.set_sync();
        let lock_surf = lock.get_lock_surface(&surf, &s.output, &qh, ());
        s.surface = Some(surf);
        s.child = Some(child);
        s.subsurface = Some(subsurface);
        s.lock_surface = Some(lock_surf);
    }
    event_queue.roundtrip(&mut app_data).unwrap();

    //while app_data.locked {
    //    event_queue.blocking_dispatch(&mut app_data).unwrap();
    //}

    lock.unlock_and_destroy();
    event_queue.roundtrip(&mut app_data).unwrap();
    println!("Successfully unlocked!");
}
