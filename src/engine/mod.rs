use crossterm::{execute, style::*};
use rhai::plugin::*;

pub fn assets_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap().join("DONT_OPEN/pc")
}

pub fn path<T: Into<std::path::PathBuf>>(path: T) -> std::path::PathBuf {
    assets_path().join(path.into())
}

#[export_module]
mod rhai_std {
    use rhai::*;

    pub mod term {
        use crossterm::event::{KeyCode, KeyModifiers};
        use crossterm::{cursor::*, execute};

        #[derive(Clone)]
        pub struct CursorPosition(pub u16, pub u16);

        #[rhai_fn(global, get = "column")]
        pub fn get_column(cursor_position: &mut CursorPosition) -> i64 {
            cursor_position.0 as i64
        }

        #[rhai_fn(global, get = "row")]
        pub fn get_row(cursor_position: &mut CursorPosition) -> i64 {
            cursor_position.1 as i64
        }

        #[rhai_fn(global, set = "column")]
        pub fn set_column(cursor_position: &mut CursorPosition, column: i64) {
            cursor_position.0 = column as u16;
        }

        #[rhai_fn(global, set = "row")]
        pub fn set_row(cursor_position: &mut CursorPosition, row: i64) {
            cursor_position.1 = row as u16;
        }

        pub fn get_cursor_position() -> CursorPosition {
            let (x, y) = crossterm::cursor::position().unwrap();

            CursorPosition(x, y)
        }

        pub fn set_cursor_position(cursor_position: &mut CursorPosition) {
            execute!(
                std::io::stdout(),
                MoveTo(cursor_position.0, cursor_position.1),
            )
            .unwrap();
        }

        #[derive(Clone)]
        pub struct KeyEvent(pub crossterm::event::KeyEvent);

        #[rhai_fn(global)]
        pub fn code(key_event: &mut KeyEvent) -> String {
            match key_event.0.code {
                KeyCode::Backspace => String::from("Backspace"),
                KeyCode::Enter => String::from("Enter"),
                KeyCode::Left => String::from("Left"),
                KeyCode::Right => String::from("Right"),
                KeyCode::Up => String::from("Up"),
                KeyCode::Down => String::from("Down"),
                KeyCode::Home => String::from("Home"),
                KeyCode::End => String::from("End"),
                KeyCode::PageUp => String::from("PageUp"),
                KeyCode::PageDown => String::from("PageDown"),
                KeyCode::Tab => String::from("Tab"),
                KeyCode::BackTab => String::from("BackTab"),
                KeyCode::Delete => String::from("Delete"),
                KeyCode::Insert => String::from("Insert"),
                KeyCode::F(num) => format!("F{}", num),
                KeyCode::Char(_) => String::from("Char"),
                KeyCode::Null => String::from("Null"),
                KeyCode::Esc => String::from("Esc"),
            }
        }

        #[rhai_fn(global)]
        pub fn char(key_event: &mut KeyEvent) -> String {
            match key_event.0.code {
                KeyCode::Char(c) => String::from(c),
                _ => String::from(""),
            }
        }

        #[rhai_fn(global)]
        pub fn shift(key_event: &mut KeyEvent) -> bool {
            key_event.0.modifiers.contains(KeyModifiers::SHIFT)
        }

        #[rhai_fn(global)]
        pub fn ctrl(key_event: &mut KeyEvent) -> bool {
            key_event.0.modifiers.contains(KeyModifiers::CONTROL)
        }

        #[rhai_fn(global)]
        pub fn alt(key_event: &mut KeyEvent) -> bool {
            key_event.0.modifiers.contains(KeyModifiers::ALT)
        }

        pub fn read_key() -> KeyEvent {
            loop {
                match crossterm::event::read().unwrap() {
                    crossterm::event::Event::Key(key_event) => {
                        return KeyEvent(key_event);
                    }
                    _ => {}
                }
            }
        }
    }

    pub mod sys {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        };
        use std::thread;

        pub fn keys_pressed() -> Map {
            let device_state = device_query::DeviceState::new();

            device_state
                .query_keymap()
                .into_iter()
                .map(|key_code| (format!("{}", key_code).into(), ().into()))
                .collect()
        }

        pub fn sleep(duration: f64) {
            std::thread::sleep(std::time::Duration::from_secs_f64(duration));
        }

        #[derive(Clone)]
        pub struct ProgramHandle {
            pub handle: Arc<Mutex<Option<thread::JoinHandle<Result<(), Box<EvalAltResult>>>>>>,
            pub running: Arc<AtomicBool>,
        }

        #[rhai_fn(global)]
        pub fn running(program_handle: &mut ProgramHandle) -> bool {
            let running = program_handle.running.load(Ordering::SeqCst);

            running
        }

        #[rhai_fn(global)]
        pub fn terminate(program_handle: &mut ProgramHandle) -> String {
            program_handle.running.store(false, Ordering::SeqCst);

            let mut handle = program_handle.handle.lock().unwrap();

            let handle = std::mem::replace(&mut *handle, None);

            match handle {
                Some(handle) => match handle.join() {
                    Ok(result) => match result {
                        Ok(_) => String::new(),
                        Err(err) => format!("{:?}", err),
                    },
                    Err(err) => format!("{:?}", err),
                },
                None => String::new(),
            }
        }

        #[rhai_fn(return_raw)]
        pub fn run(path: &str, env: Map) -> Result<Dynamic, Box<EvalAltResult>> {
            let mut engine = crate::engine::engine();

            let path: String = path.into();
            let running = Arc::new(AtomicBool::new(true));
            let thread_running = running.clone();
            let handle = thread::spawn(move || {
                let running = thread_running.clone();

                engine.on_progress(move |_ops| {
                    let running = running.load(Ordering::SeqCst);

                    if !running {
                        panic!("Program terminated");
                    }

                    None
                });

                let engine = engine;

                let mut scope = Scope::new();

                scope.push_constant("ENV", env);

                let result =
                    engine.eval_file_with_scope::<()>(&mut scope, super::super::path(path));

                thread_running.store(false, Ordering::SeqCst);

                result
            });

            Ok(Dynamic::from(ProgramHandle {
                handle: Arc::new(Mutex::new(Some(handle))),
                running,
            }))
        }

        #[rhai_fn(return_raw)]
        pub fn open(path: &str) -> Result<Dynamic, Box<EvalAltResult>> {
            match open::that(super::super::path(path)) {
                Ok(_) => Ok(().into()),
                Err(_err) => Err(format!("Failed to open: {}, {:?}", path, _err).into()),
            }
        }
    }

    pub mod fs {
        use std::io::prelude::*;

        pub fn is_dir(path: &str) -> bool {
            super::super::path(path.clone()).is_dir()
        }

        #[rhai_fn(return_raw)]
        pub fn dir(path: &str) -> Result<Dynamic, Box<EvalAltResult>> {
            let files: Vec<Dynamic> = std::fs::read_dir(super::super::path(path))
                .map_err(|_err| format!("Directory: '{}' not found", path))?
                .map(|entry| {
                    let dir = entry.unwrap();

                    dir.file_name().into_string().unwrap().into()
                })
                .collect();

            Ok(files.into())
        }

        pub fn exists(path: &str) -> bool {
            super::super::path(path).exists()
        }

        #[rhai_fn(return_raw)]
        pub fn write_file(path: &str, content: &str) -> Result<Dynamic, Box<EvalAltResult>> {
            let mut file = std::fs::File::create(super::super::path(path))
                .map_err(|_err| format!("File not found: {}", path))?;

            file.write(content.as_bytes())
                .map_err(|_err| format!("Failed to write to file: {}", path))?;

            Ok(().into())
        }

        #[rhai_fn(return_raw)]
        pub fn read_file(path: &str) -> Result<Dynamic, Box<EvalAltResult>> {
            let mut file = std::fs::File::open(super::super::path(path))
                .map_err(|_err| format!("File not found: {}", path))?;

            let mut buf = String::new();

            file.read_to_string(&mut buf)
                .map_err(|_err| format!("Failed to read file: {}", path))?;

            Ok(buf.into())
        }
    }
}

pub fn engine() -> Engine {
    let mut engine = Engine::new();

    engine.on_print(|msg| {
        execute!(std::io::stdout(), Print(msg.to_string()),).unwrap();
    });

    let std = exported_module!(rhai_std);

    engine.register_static_module("std", std.into());

    engine
}
