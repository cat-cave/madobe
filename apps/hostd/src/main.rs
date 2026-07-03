#![doc = "madobe host daemon bootstrap executable."]
#![forbid(unsafe_code)]

fn main() {
    println!("{}", hostd::status_line());
}
