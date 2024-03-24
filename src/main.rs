#![windows_subsystem = "windows"]

use std::env;
use std::process::Command;
use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;

use device_query::DeviceQuery;
use device_query::DeviceState;
use tray_icon::Icon;
use tray_icon::TrayIconEvent;
use uiautomation::types::Rect;
use uiautomation::Result;
use uiautomation::UIAutomation;
use uiautomation::UIElement;
use uiautomation::UITreeWalker;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::POINT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::SendMessageA;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONDOWN;
use windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoopBuilder;
use winit::platform::windows::EventLoopBuilderExtWindows;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use tray_icon::TrayIconBuilder;

fn main() {
    println!("power by xiaoeyun");

    println!("bootsing");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let cur_ver = hklm
        .open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
        .unwrap();
    let path: String = cur_ver.get_value("InstallPath").unwrap();
    let gameid = "2291680";

    let _process = Command::new(format!("{}\\steam.exe", path))
        .args(["-applaunch", gameid])
        .args(env::args())
        .spawn()
        .expect("can't start game.");

    println!("waiting mount");
    let element;
    let automation = UIAutomation::new().unwrap();
    loop {
        sleep(Duration::from_secs(1));
        let walker = automation.get_control_view_walker().unwrap();
        let root = automation.get_root_element().unwrap();
        let finded_element = find_element(&walker, &root, 0).unwrap();
        if let Some(el) = finded_element {
            element = el;
            break;
        }
    }

    println!("got handle");
    let rect = element.get_bounding_rectangle().unwrap();
    let handle = element.get_native_window_handle().unwrap().into();

    spawn(move || {
        println!("init wallpaper");
        {
            sleep(Duration::from_secs(18));
            click(&handle, &rect, Some((1350, 700)));
            sleep(Duration::from_millis(300));
            click(&handle, &rect, Some((700, 300)));
            sleep(Duration::from_millis(3000));
            click(&handle, &rect, Some((70, 580)));
            sleep(Duration::from_millis(300));
            click(&handle, &rect, Some((700, 400)));
            sleep(Duration::from_millis(1600));
            click(&handle, &rect, Some((1640, 1030)));
        }
    });

    println!("load event loop");
    let device_state = DeviceState::new();
    let event_loop = EventLoopBuilder::new()
        .with_any_thread(true)
        .build()
        .unwrap();

    println!("load tray");
    let tray = TrayIconBuilder::new()
        .with_icon(Icon::from_resource(1, None).unwrap())
        .with_tooltip("Wallpaper Engine Handler (power by xiaoeyun)\nclickable: false")
        .build()
        .unwrap();

    println!("running event loop");
    println!("waiting tray input");
    let tray_channel = TrayIconEvent::receiver();
    let mut clickable = false;
    let mut hold = false;
    let _ = event_loop.run(move |_event, event_loop| {
        event_loop.set_control_flow(ControlFlow::Poll);

        // clickable
        if clickable {
            let mouse = device_state.get_mouse();
            let clicked = mouse.button_pressed[1];
            if hold != clicked {
                unsafe {
                    SendMessageA(
                        handle,
                        if clicked {
                            WM_LBUTTONDOWN
                        } else {
                            WM_LBUTTONUP
                        },
                        WPARAM::default(),
                        LPARAM::default(),
                    );
                };
            }
            hold = clicked;
        }

        // watching tray input
        if let Ok(_) = tray_channel.try_recv() {
            *&mut clickable = !clickable;
            tray.set_tooltip(Some(format!(
              "Wallpaper Engine Handler (power by xiaoeyun)\nclickable: {}",
                clickable
            )))
            .unwrap();
        }

        sleep(Duration::from_millis(50));
    });

    fn get_cursor() -> POINT {
        let mut point = POINT::default();
        unsafe { GetCursorPos(&mut point).unwrap() };
        point
    }
    fn set_cursor(point: POINT) {
        unsafe { SetCursorPos(point.x, point.y).unwrap() }
    }

    fn click(handle: &HWND, rect: &Rect, xy: Option<(isize, isize)>) {
        let mut orgin = POINT::default();
        if let Some((x, y)) = xy {
            orgin = get_cursor();
            set_cursor(get_point(&rect, x, y));
        }

        let handle = handle.clone();
        unsafe {
            SendMessageA(handle, WM_LBUTTONDOWN, WPARAM::default(), LPARAM::default());
            SendMessageA(handle, WM_LBUTTONUP, WPARAM::default(), LPARAM::default());
        };

        if xy.is_some() {
            set_cursor(orgin);
        }

        fn get_point(rect: &Rect, x: isize, y: isize) -> POINT {
            POINT {
                x: rect.get_left() + (x as i32 * rect.get_width() / 1920),
                y: rect.get_top() + (y as i32 * rect.get_height() / 1080),
            }
        }
    }
}

fn find_element(
    walker: &UITreeWalker,
    element: &UIElement,
    level: usize,
) -> Result<Option<UIElement>> {
    let find_map = ["", "Progman", "WPEAppIntermediateWorker", "UnityWndClass"];

    return match level {
        0 => iter(walker, element, level),
        1..=3 => {
            if find_map[level] != element.get_classname()? {
                return Ok(None);
            }
            if level == 3 {
                return Ok(Some(element.clone()));
            }
            iter(walker, element, level)
        }
        _ => unreachable!(),
    };

    fn iter(walker: &UITreeWalker, element: &UIElement, level: usize) -> Result<Option<UIElement>> {
        let mut result = None;
        if let Ok(child) = walker.get_first_child(&element) {
            result = find_element(walker, &child, level + 1)?;
            if result.is_some() {
                return Ok(result);
            }

            let mut next = child;
            while let Ok(sibling) = walker.get_next_sibling(&next) {
                result = find_element(walker, &sibling, level + 1)?;
                if result.is_some() {
                    break;
                }

                next = sibling;
            }
        }
        Ok(result)
    }
}
