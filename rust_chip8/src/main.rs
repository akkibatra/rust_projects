mod cpu;
use cpu::{Cpu, SCREEN_WIDTH, SCREEN_HEIGHT};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const SCALE: u32 = 15;

fn main() -> Result<(), String> {
    // 1. Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Rust CHIP-8 Emulator", SCREEN_WIDTH as u32 * SCALE, SCREEN_HEIGHT as u32 * SCALE)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    // 2. Initialize our CPU
    let mut cpu = Cpu::new();

    let rom_path = "pong.rom";
    if !std::path::Path::new(rom_path).exists() {
        panic!("ERROR: ROM file not found at {}", rom_path);
    }

    cpu.load_rom(rom_path);

    // 3. The Main Loop
    'running: loop {
        // Handle Events (Closing the window)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some(k) = map_key(key) { cpu.keypad[k] = true; }
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                    if let Some(k) = map_key(key) { cpu.keypad[k] = false; }
                }
                _ => {}
            }
        }

        // Execute a CPU cycle
        for _ in 0..10 {
            cpu.tick();
        }

        cpu.update_timers();

        // Draw the screen
        draw_screen(&cpu, &mut canvas);

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

fn map_key(keycode: Keycode) -> Option<usize>{
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),

        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),

        _ => None
    }
}

fn draw_screen(cpu: &Cpu, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for y in 0..32 {
        for x in 0..64 {
            if cpu.display[y * 64 + x] == 1 {
                // Draw a scaled rectangle for each pixel
                let rect = sdl2::rect::Rect::new(
                    (x as u32 * SCALE) as i32,
                    (y as u32 * SCALE) as i32,
                    SCALE,
                    SCALE,
                );
                canvas.fill_rect(rect).unwrap();
            }
        }
    }
    canvas.present();
}