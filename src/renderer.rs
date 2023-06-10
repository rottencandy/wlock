use std::{ptr, ffi::CStr};

use gl::types::{GLenum, GLuint, GLint, GLchar, GLboolean, GLvoid};
use khronos_egl as egl;
use wayland_client::{protocol::{wl_display, wl_surface}, Proxy};

pub fn setup_renderer(display: &wl_display::WlDisplay, surface: &wl_surface::WlSurface, width: i32, height: i32) {
    // Create an EGL API instance.
    let egl = egl::Instance::new(egl::Static);
    egl.bind_api(egl::OPENGL_API).expect("unable to select OpenGL API");
    gl::load_with(|name| egl.get_proc_address(name).unwrap() as *const std::ffi::c_void);

    // Setup EGL.
    let egl_display = setup_egl(&egl, display);
    let (egl_context, egl_config) = create_context(&egl, egl_display);

    // Create a surface.
    // Note that it must be kept alive to the end of execution.
    let _surface = setup_surface(&egl, surface, width, height, egl_display, egl_config, egl_context);
}

fn setup_surface(egl: &egl::Instance<egl::Static>, surface: &wl_surface::WlSurface, width: i32, height: i32, egl_display: egl::Display, egl_config: egl::Config, egl_context: egl::Context) {
    let wl_egl_surface = wayland_egl::WlEglSurface::new(surface.id(), width, height).expect("Unable to init wl_egl_surface");

    let egl_surface = unsafe {
        egl.create_window_surface(
            egl_display,
            egl_config,
            wl_egl_surface.ptr() as egl::NativeWindowType,
            None,
            )
            .expect("unable to create an EGL surface")
    };

    egl.make_current(egl_display, Some(egl_surface), Some(egl_surface), Some(egl_context))
        .expect("unable to bind the context");

    render();

    egl.swap_buffers(egl_display, egl_surface)
        .expect("unable to post the surface content");
}

fn setup_egl(egl: &egl::Instance<egl::Static>, display: &wl_display::WlDisplay) -> egl::Display {
    let egl_display = egl.get_display(display.id().as_ptr() as *mut std::ffi::c_void).unwrap();
    egl.initialize(egl_display).unwrap();

    egl_display
}

fn create_context(egl: &egl::Instance<egl::Static>, display: egl::Display) -> (egl::Context, egl::Config) {
    let attributes = [
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::NONE,
    ];

    let config = egl.choose_first_config(display, &attributes)
        .expect("unable to choose an EGL configuration")
        .expect("no EGL configuration found");

    let context_attributes = [
        egl::CONTEXT_MAJOR_VERSION,
        4,
        egl::CONTEXT_MINOR_VERSION,
        0,
        egl::CONTEXT_OPENGL_PROFILE_MASK,
        egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
        egl::NONE,
    ];

    let context = egl.create_context(display, config, None, &context_attributes)
        .expect("unable to create an EGL context");

    (context, config)
}

fn format_error(e: GLenum) -> &'static str {
    match e {
        gl::NO_ERROR => "No error",
        gl::INVALID_ENUM => "Invalid enum",
        gl::INVALID_VALUE => "Invalid value",
        gl::INVALID_OPERATION => "Invalid operation",
        gl::INVALID_FRAMEBUFFER_OPERATION => "Invalid framebuffer operation",
        gl::OUT_OF_MEMORY => "Out of memory",
        gl::STACK_UNDERFLOW => "Stack underflow",
        gl::STACK_OVERFLOW => "Stack overflow",
        _ => "Unknown error"
    }
}

fn check_gl_errors() {
    unsafe {
        match gl::GetError() {
            gl::NO_ERROR => (),
            e => {
                panic!("OpenGL error: {}", format_error(e))
            }
        }
    }
}

unsafe fn check_shader_status(shader: GLuint) {
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetProgramiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        if len > 0 {
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
                );

            let log = String::from_utf8(buf).unwrap();
            eprintln!("shader compilation log:\n{}", log);
        }

        panic!("shader compilation failed");
    }
}

const VERTEX: &'static [GLint; 8] = &[
    -1, -1,
    1, -1,
    1, 1,
    -1, 1
];

const INDEXES: &'static [GLuint; 4] = &[
    0, 1, 2, 3
];

const VERTEX_SHADER: &[u8] = b"#version 400
in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0f, 1.0f);
}
\0";

const FRAGMENT_SHADER: &[u8] = b"#version 400
out vec4 color;

void main() {
    color = vec4(0.0f, 1.0f, 0.0f, 1.0f);
}
\0";

fn render() {
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        check_gl_errors();
        let src = CStr::from_bytes_with_nul_unchecked(VERTEX_SHADER).as_ptr();
        gl::ShaderSource(vertex_shader, 1, (&[src]).as_ptr(), ptr::null());
        check_gl_errors();
        gl::CompileShader(vertex_shader);
        check_shader_status(vertex_shader);

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        check_gl_errors();
        let src = CStr::from_bytes_with_nul_unchecked(FRAGMENT_SHADER).as_ptr();
        gl::ShaderSource(fragment_shader, 1, (&[src]).as_ptr(), ptr::null());
        check_gl_errors();
        gl::CompileShader(fragment_shader);
        check_shader_status(fragment_shader);

        let program = gl::CreateProgram();
        check_gl_errors();
        gl::AttachShader(program, vertex_shader);
        check_gl_errors();
        gl::AttachShader(program, fragment_shader);
        check_gl_errors();
        gl::LinkProgram(program);
        check_gl_errors();
        gl::UseProgram(program);
        check_gl_errors();

        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        check_gl_errors();
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        check_gl_errors();
        gl::BufferData(
            gl::ARRAY_BUFFER,
            8 * 4,
            VERTEX.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW
            );
        check_gl_errors();

        let mut vertex_input = 0;
        gl::GenVertexArrays(1, &mut vertex_input);
        check_gl_errors();
        gl::BindVertexArray(vertex_input);
        check_gl_errors();
        gl::EnableVertexAttribArray(0);
        check_gl_errors();
        gl::VertexAttribPointer(
            0, 2, gl::INT, gl::FALSE as GLboolean, 0, 0 as *const GLvoid
            );
        check_gl_errors();

        let mut indexes = 0;
        gl::GenBuffers(1, &mut indexes);
        check_gl_errors();
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indexes);
        check_gl_errors();
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            4 * 4,
            INDEXES.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW
            );
        check_gl_errors();

        gl::DrawElements(gl::TRIANGLE_FAN, 4, gl::UNSIGNED_INT, std::ptr::null());
        check_gl_errors();
    }
}