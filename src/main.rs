use std::time::Duration;
use std::process::Command;
use dbus::blocking::Connection;
use dbus::message::MatchRule;
use dbus::Message;



// This programs implements the equivalent of running the "dbus-monitor" tool
fn main() {

    let use_system_bus = std::env::args().into_iter().any(|a| a == "--system");

    let conn = (if use_system_bus { Connection::new_system() } else { Connection::new_session() }).expect("D-Bus connection failed");

    
    if let Err(_) = is_server_already_present(&conn) {
        /* will return Err if server did not answer, meaning we can take the spot */
        

        conn.request_name("org.freedesktop.Notifications", false, true, false)
            .expect("Could not request Notifications");

        let mut rule = MatchRule::new();
        rule.path = Some("/org/freedesktop/Notifications".into());
        
        conn.add_match(rule, handle_message)
            .expect("Could not AddMatch over notifications");

        loop {
            conn.process(Duration::from_millis(1000)).unwrap();
        }
    }
    
    
    
}

fn is_server_already_present(conn: &Connection) -> Result<bool, Box<dyn std::error::Error>> {
    let proxy = conn.with_proxy("org.freedesktop.Notifications", "/org/freedesktop/Notifications", Duration::from_millis(500));

    let (srvinfo, author, version, _): (String,String,String,String,) = proxy.method_call("org.freedesktop.Notifications", "GetServerInformation", ())?;
    
    println!("[DEBUG] {} - {} - v{}", srvinfo, author, version);

    Ok(true)
}

fn reply_server_information(conn: &Connection, msg: &Message) -> bool {
    let reply = msg.method_return()
        .append_ref(&["notrs".to_string(), "vaelio <archelio@protonmail.com>".to_string(), "0.1.0".to_string(), "1.2".to_string()]);

    match conn.channel().send(reply) {
        Ok(_) => {
            println!("Sent reply !");
            true
        },
        Err(_) => {
            println!("Reply failed");
            false
        },
    }
}

fn reply_capabilities(conn: &Connection, msg: &Message) -> bool {
    let args = vec!["actions".to_string(), "body".to_string()];
    let reply = msg.method_return()
        .append1(args);
    println!("[DEBUG] {:?}", msg);
    match conn.channel().send(reply) {
        Ok(_) => {
            println!("Sent reply !");
            true
        },
        Err(_) => {
            println!("Reply failed");
            false
        },
    }
}


fn handle_message(_: (), conn: &Connection, msg: &Message) -> bool {
    println!("[DEBUG] {:?}", msg);
    let member = String::from_utf8_lossy(msg.member().unwrap().as_bytes()).to_string();
    match member.as_ref() {
        "GetServerInformation" => reply_server_information(conn, msg),
        "GetCapabilities" => reply_capabilities(conn, msg),
        "Notify"=> notify(conn, msg),
        "CloseNotification" => close(conn, msg),
        _ => true,
    }
}

fn close(conn: &Connection, msg: &Message) -> bool {
    let reply = msg.method_return();
    
    match conn.channel().send(reply) {
        Ok(_) => {
            println!("Sent reply !");
            true
        },
        Err(_) => {
            println!("Reply failed");
            false
        },
    }
}

fn notify(conn: &Connection, msg: &Message) -> bool {
    let args = msg.get_items();

    let pname: &str = args[0].inner().unwrap();
    let summary: &str = args[3].inner().unwrap();
    let content: &str = args[4].inner().unwrap();

    Command::new("hyprctl")
        .arg("notify")
        .arg("-1")
        .arg("10000")
        .arg("rgb(505050)")
        .arg(format!("[{}]: {} - {}", pname, summary, content))
        .output()
        .expect("Could not send notifications using hyprctl notify");

    let reply = msg.method_return()
        .append_ref(&[2u32]);

    match conn.channel().send(reply) {
        Ok(_) => {
            println!("Sent reply !");
            true
        },
        Err(_) => {
            println!("Reply failed");
            false
        },
    }
}