extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mc;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;
const SEED: u32 = 219734390;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()



// == // Modify and complete the function below for the first task
// unsafe fn FUNCTION_NAME(ARGUMENT_NAME: &Vec<f32>, ARGUMENT_NAME: &Vec<u32>) -> u32 { }

fn main() {
    if !std::path::Path::new("./points.txt").exists() {
        eprint!("File points.txt not found. Generating point cloud . . . ");
        use noise::{NoiseFn, Perlin};
        use std::io::Write;
        let perlin = Perlin::new(SEED);
        let nfreq = 0.2;
        let mut f = std::fs::File::create("./points.txt").unwrap();

        writeln!(f, "129 129 129");
        (0..129).for_each(|i|{
            (0..129).for_each(|j|{
                (0..129).for_each(|k|{
                    write!(f, "{:.4} ", (perlin.get([i as f64 * nfreq, j as f64 * nfreq, k as f64 * nfreq]) + 1.0) / 2.0);}
                );}
            );}
        );
        eprintln!("done.")
    }

    // Set up the necessary objects to deal with windows and event handling
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Marching Cubes")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &event_loop).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CCW);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            eprintln!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            eprintln!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            eprintln!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        use std::io::{prelude::*, BufReader};
        use std::fs::File;
        let f = File::open("./points.txt").unwrap();
        let mut f = BufReader::new(f);
        let mut bd = String::new();
        let _ = f.read_line(&mut bd);
        let bd = bd.trim().split_ascii_whitespace().map(|s| s.parse::<usize>().unwrap()).collect::<Vec<_>>();
        let mut f = f.split(b' ');
        let f = f.map(|b| std::str::from_utf8(&b.unwrap()).unwrap().parse::<f64>().unwrap()).collect::<Vec<_>>();
        eprintln!("Loaded {}/{} points", f.len(), bd[0]*bd[1]*bd[2]);
        println!("{} {} {} {}", f[0], f[1], f[2], f[3]);
        let points = (0..bd[0]).map(|i|
            (0..bd[1]).map(|j|
                (0..bd[2]).map(|k|
                    f[i*bd[1]*bd[2]+j*bd[1]+k]
                    //std::str::from_utf8(&f.next().unwrap().unwrap()).unwrap().parse::<f64>().unwrap()
                    //(perlin.get([i as f64 * nfreq, j as f64 * nfreq, k as f64 * nfreq]) + 1.0) / 2.0
                ).collect::<Vec<_>>()
            ).collect::<Vec<_>>()
        ).collect::<Vec<_>>();

        use rayon::prelude::*;
        let s = 8;
        let chunks = (0..s*s*s).map(|i|{
            eprintln!("MC on chunk ({},{},{})",(i as usize/ (s*s))*16,((i as usize/s)%s)*16,(i as usize%s)*16);
            mc::marching_cubes(((i as usize/ (s*s))*16,((i as usize/s)%s)*16,(i as usize%s)*16), 0.5, &points, 0.4)
        }).collect::<Vec<_>>();
        let chunks = chunks.into_iter().map(|m| {

            let vertices = m.vertices;
            let indices = m.indices;
            let normals = m.normals;
            let cube_ic = m.index_count;

            //---------------------------------------------------------------------/
            // Set up VAO
            //---------------------------------------------------------------------/
            unsafe {
                let mut vao = 0;
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);

                let mut ibo = 0;
                gl::GenBuffers(1, &mut ibo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    byte_size_of_array(&indices),
                    pointer_to_array(&indices) as *const _,
                    gl::STATIC_DRAW
                );

                let mut vbo = 0;
                gl::GenBuffers(1, &mut vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    byte_size_of_array(&vertices),
                    pointer_to_array(&vertices) as *const _,
                    gl::STATIC_DRAW
                );

                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

                let mut nbo = 0;
                gl::GenBuffers(1, &mut nbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    byte_size_of_array(&normals),
                    pointer_to_array(&normals) as *const _,
                    gl::STATIC_DRAW
                );

                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
                (vao, m.index_count)
            }
        }).collect::<Vec<_>>();
        eprintln!("VAOS: {:?}", chunks);


        //let m = mc::mc_test();


        let mut grid_vao = Vec::new();
        let mut grid_model_mat = Vec::new();
        let grid_ic = 36;
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    let m = mc::Mesh::cube(glm::vec3(16.0,16.0,16.0), glm::vec2(1.0,1.0), true, false, glm::vec3(1.0,1.0,1.0), glm::vec4(0.0, 0.0, 0.0, 1.0));
                    let vertices = m.vertices;
                    let indices = m.indices;
                    let normals = m.normals;
                    let vao = unsafe {
                        let mut vao = 0;
                        gl::GenVertexArrays(1, &mut vao);
                        gl::BindVertexArray(vao);

                        let mut ibo = 0;
                        gl::GenBuffers(1, &mut ibo);
                        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
                        gl::BufferData(
                            gl::ELEMENT_ARRAY_BUFFER,
                            byte_size_of_array(&indices),
                            pointer_to_array(&indices) as *const _,
                            gl::STATIC_DRAW
                        );

                        let mut vbo = 0;
                        gl::GenBuffers(1, &mut vbo);
                        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                        gl::BufferData(
                            gl::ARRAY_BUFFER,
                            byte_size_of_array(&vertices),
                            pointer_to_array(&vertices) as *const _,
                            gl::STATIC_DRAW
                        );

                        gl::EnableVertexAttribArray(0);
                        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

                        let mut nbo = 0;
                        gl::GenBuffers(1, &mut nbo);
                        gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
                        gl::BufferData(
                            gl::ARRAY_BUFFER,
                            byte_size_of_array(&normals),
                            pointer_to_array(&normals) as *const _,
                            gl::STATIC_DRAW
                        );

                        gl::EnableVertexAttribArray(1);
                        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
                        vao
                    };
                    grid_vao.push(vao);
                    grid_model_mat.push(
                        glm::translate(
                            &glm::identity(),
                            &glm::vec3(8.0+16.0*i as f32, 8.0+16.0*j as f32, 8.0+16.0*k as f32)
                        )
                    );

                }
            }
        }

        // Basic usage of shader helper
        // The code below returns a shader object, which contains the field .program_id
        // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path), but it only needs to be called once
        // shader::ShaderBuilder::new().attach_file("./path/to/shader").link();
        let sh = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
        };
        unsafe { sh.activate() };
        let mut mouse_pos = (0.0, 0.0);

        let u_time = unsafe { sh.get_uniform_location("u_time") };

        let u_color = unsafe { sh.get_uniform_location("u_color") };
        let u_aspect = unsafe { sh.get_uniform_location("u_aspect") };
        let u_screen_w = unsafe { sh.get_uniform_location("u_screen_w") };
        let u_screen_h = unsafe { sh.get_uniform_location("u_screen_h") };
        let u_mouse_x = unsafe { sh.get_uniform_location("u_mouse_x") };
        let u_mouse_y = unsafe { sh.get_uniform_location("u_mouse_y") };
        let u_mvp = unsafe { sh.get_uniform_location("u_mvp") };
        let u_model = unsafe { sh.get_uniform_location("u_model") };
        let u_view = unsafe { sh.get_uniform_location("u_view") };

        // Just adjust aspect ratio
        // let mvp = glm::scale(&glm::identity(), &glm::vec3(1.0, (SCREEN_W / SCREEN_H) as _, 1.0));
        let mvp = glm::ortho(0.0f32, SCREEN_W as _, 0.0, SCREEN_H as _, 0.0, 1.0);

        let aspect = SCREEN_W as f32 / SCREEN_H as f32;

        let perspective_mat: glm::Mat4 = glm::perspective(
            aspect,
            1.8,       // field of view
            0.1, // near
            500.0   // far
        );

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                        },
                        VirtualKeyCode::D => {
                        },


                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {



                *delta = (0.0, 0.0);
            }
            let mid = 32.0f32;
            let view_mat = glm::look_at(&glm::vec3(mid+mid*2.0*elapsed.cos(),mid*2.5,mid+mid*2.0*elapsed.sin()), &glm::vec3(mid,mid,mid), &glm::vec3(0.0, 1.0, 0.0));

            let mvp: glm::TMat4<f32> = perspective_mat * view_mat;

            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here
                let model_mat: glm::Mat4 = glm::identity();
                gl::UniformMatrix4fv(u_view, 1, gl::FALSE, view_mat.as_ptr());
                gl::UniformMatrix4fv(u_model, 1, gl::FALSE, model_mat.as_ptr());
                gl::UniformMatrix4fv(u_mvp, 1, gl::FALSE, mvp.as_ptr());
                gl::Uniform1f(u_time, elapsed);
                gl::Uniform1f(u_aspect, SCREEN_W as f32 / SCREEN_H as f32);
                gl::Uniform1ui(u_screen_w, SCREEN_W);
                gl::Uniform1ui(u_screen_h, SCREEN_H);
                gl::Uniform1f(u_mouse_x, mouse_pos.0);
                gl::Uniform1f(u_mouse_y, mouse_pos.1);

                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                gl::Disable(gl::CULL_FACE);
                for (vao, ic) in chunks.iter() {

                    gl::Uniform4f(u_color, 1.0, 0.0, 1.0, 1.0);
                    gl::BindVertexArray(*vao);
                    gl::DrawElements(gl::TRIANGLES, *ic, gl::UNSIGNED_INT, std::ptr::null());
                }

                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                for (vao, model) in grid_vao.iter().zip(&grid_model_mat) {
                    let mvp = mvp * model;
                    gl::UniformMatrix4fv(u_mvp, 1, gl::FALSE, mvp.as_ptr());
                    gl::Uniform4f(u_color, 0.0, 0.0, 0.0, 0.5);

                    gl::BindVertexArray(*vao);
                    gl::DrawElements(gl::TRIANGLES, grid_ic, gl::UNSIGNED_INT, std::ptr::null());

                }
                gl::Enable(gl::CULL_FACE);



            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
