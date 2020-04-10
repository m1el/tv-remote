use serde_derive::{Deserialize, Serialize};
use winapi::um::winuser::{self as wu, SendInput, INPUT};

use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct Keypress {
    key: String,
}

struct Key {
    code: i32,
    extended: bool,
    alt: bool,
    ctrl: bool,
    shift: bool,
}

fn key_from_str(s: &str) -> Option<Key> {
    let alt = false;
    let ctrl = false;
    let mut shift = false;
    let mut extended = false;

    let code = match s {
        "play" => wu::VK_PLAY,
        "stop" => wu::VK_MEDIA_STOP,
        "play/pause" => wu::VK_MEDIA_PLAY_PAUSE,
        "mute" => wu::VK_VOLUME_MUTE,
        "vol-" => wu::VK_VOLUME_DOWN,
        "vol+" => wu::VK_VOLUME_UP,
        "next" => wu::VK_MEDIA_NEXT_TRACK,
        "prev" => wu::VK_MEDIA_PREV_TRACK,
        "seek_exact+1" => {
            shift = true;
            extended = true;
            wu::VK_RIGHT
        }
        "seek_exact-1" => {
            shift = true;
            extended = true;
            wu::VK_LEFT
        }
        "seek_exact+5" => {
            shift = true;
            extended = true;
            wu::VK_UP
        }
        "seek_exact-5" => {
            shift = true;
            extended = true;
            wu::VK_DOWN
        }
        "seek+5" => {
            extended = true;
            wu::VK_RIGHT
        }
        "seek-5" => {
            extended = true;
            wu::VK_LEFT
        }
        "seek+60" => {
            extended = true;
            wu::VK_UP
        }
        "seek-60" => {
            extended = true;
            wu::VK_DOWN
        }
        _ => return None
    };
    Some(Key { code, extended, alt, ctrl, shift })
}

fn send_key(key: &str) {
    fn mk_key(code: i32, flags: u32) -> INPUT {
        unsafe {
            let mut input = std::mem::zeroed::<INPUT>();
            input.type_ = wu::INPUT_KEYBOARD;
            let ki = input.u.ki_mut();
            ki.wVk = code as _;
            ki.dwFlags = flags;
            input
        }
    }

    let key = if let Some(key) = key_from_str(key) {
        key
    } else {
        return
    };
    let mut inputs = Vec::new();

    if key.alt { inputs.push(mk_key(wu::VK_MENU, 0)); }
    if key.ctrl { inputs.push(mk_key(wu::VK_CONTROL, 0)); }
    if key.shift { inputs.push(mk_key(wu::VK_SHIFT, 0)); }
    let flags = if key.extended { wu::KEYEVENTF_EXTENDEDKEY } else { 0 };
    inputs.push(mk_key(key.code, flags));
    inputs.push(mk_key(key.code, flags | wu::KEYEVENTF_KEYUP));
    if key.shift { inputs.push(mk_key(wu::VK_SHIFT, wu::KEYEVENTF_KEYUP)); }
    if key.ctrl { inputs.push(mk_key(wu::VK_CONTROL, wu::KEYEVENTF_KEYUP)); }
    if key.alt { inputs.push(mk_key(wu::VK_MENU, wu::KEYEVENTF_KEYUP)); }

    let rv = unsafe {
        SendInput(
            inputs.len() as _, inputs.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as _)
    };
}

const CLIENT: &str = include_str!("../client/index.html");

#[tokio::main]
async fn main() {
    let index = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::html(CLIENT));

    let dynamic = warp::get()
        .and(warp::path("dyn"))
        .and(warp::path::end())
        .and(warp::fs::dir("client"));

    // POST /keypress  {"key":"Plus"}
    let promote = warp::post()
        .and(warp::path("keypress"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|key: Keypress| {
            send_key(&key.key);
            warp::reply::json(&key)
        });

    warp::serve(promote.or(index).or(dynamic))
        .run(([0, 0, 0, 0], 3030)).await
}
