use crate::pam_wrapper::Author;

use std::process;
use std::sync::Mutex;
use std::env;
use std::path::Path;
use std::time::{ Duration, Instant };

use users::os::unix::UserExt;
use x11rb::protocol::randr::ConnectionExt;
use x11rb::connection::Connection;

const DISPLAY: &str = ":1";
const VT: &str = "vt01";

static X_SERVER: Mutex<Option<process::Child>> = Mutex::new(None);
static X_SERVER_TIMEOUT: Duration = Duration::from_millis(15000);

pub fn start_x_server() {
    let mut x_server = X_SERVER.lock().unwrap();
    if x_server.is_some() {
        return;
    }
    std::env::set_var("DISPLAY", DISPLAY);
    let child = process::Command::new("/usr/lib/Xorg")
        .arg("-nolisten").arg("tcp")
        .arg(DISPLAY).arg(VT)
        .arg("-keeptty") //.arg("-auth").arg("/usr/share/himmel_display_manager/xauthority")
        .spawn().expect("Could not start the X server");
    *x_server = Some(child);

    let start = Instant::now();
    loop {
        match x11rb::connect(None) {
            Ok(_) => break,
            Err(e) => println!("Conn err: {e:?}"),
        }
        println!("{:?}", start.elapsed());

        if start.elapsed() > X_SERVER_TIMEOUT {
            panic!("X Server timeout");
        }
        std::thread::sleep(Duration::from_millis(500));
    };
}

pub fn stop_x_server() {
    let mut x_server_lock = X_SERVER.lock().unwrap();
    if let Some(mut s) = x_server_lock.take() {
        s.kill().expect("Could not kill the X server");
    }
}

pub fn start_session(mut author: Author, username: String) -> process::Child {
    let user = users::get_user_by_name(&username).expect("Could not find user");
    author.put_env("HOME", user.home_dir());
    author.put_env("PWD", user.home_dir());
    author.put_env("SHELL", user.shell());
    author.put_env("USER", user.name());
    author.put_env("LOGNAME", user.name());
    author.put_env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/bin:/bin");
    author.put_env("MAIL", format!("/var/spool/mail/{}", user.name().to_string_lossy()));
    author.put_env("XAUTHORITY", user.home_dir().join(".Xauthority"));

    process::Command::new(user.shell())
        .arg("-c").arg("/bin/bash --login .xinitrc")
        .current_dir(user.home_dir())
        .spawn().expect("Could not start session")
}
