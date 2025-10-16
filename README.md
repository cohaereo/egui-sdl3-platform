# SDL3 egui Platform Integration

## Example
```rs
let sdl = sdl3::init().expect("Failed to initialize SDL");
let video_subsystem = sdl.video().expect("Failed to initialize video subsystem");

let window = video_subsystem
    .window("egui-sdl3-platform example", 1920, 1080)
    .position_centered()
    .resizable()
    .build()
    .expect("Failed to create window");

let mut egui_sdl3 = egui_sdl3_platform::Platform::new(&sdl, &window)
    .expect("Failed to create egui_sdl3 platform");

let mut event_pump = sdl.event_pump().unwrap();
'app: loop {
    for event in event_pump.poll_iter() {
        match event {
            sdl3::event::Event::Quit { .. } => break 'app,
            _ => {}
        }
        egui_sdl3.handle_event(&event, &video_subsystem);
    }

    let output = {
        let ctx = egui_sdl3.begin_frame(&window);
        // draw the UI using the given egui context

        egui_sdl3.end_frame(&mut sdl.video().unwrap()).unwrap()
    };
    
    // draw the output using your graphics API

    swapchain.present(0, dxgi::PresentFlags::empty());
}
```
See https://github.com/cohaereo/egui-d3d11-rs/blob/main/examples/demo.rs for a full demo using d3d11