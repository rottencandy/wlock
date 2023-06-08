use wayland_client::Connection;
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

    app_data.running = true;
    event_queue.flush().unwrap();

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

    //while app_data.locked {
    //    event_queue.blocking_dispatch(&mut app_data).unwrap();
    //}

    lock.unlock_and_destroy();
    event_queue.roundtrip(&mut app_data).unwrap();
    println!("Successfully unlocked!");
}
