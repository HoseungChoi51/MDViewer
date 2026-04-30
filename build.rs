use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/katex/katex.min.css");
    println!("cargo:rerun-if-changed=assets/katex/fonts");
    println!("cargo:rerun-if-changed=build.rs");

    let css = fs::read_to_string("assets/katex/katex.min.css")
        .expect("read assets/katex/katex.min.css");

    let mut out = String::with_capacity(css.len() + 700_000);
    let mut rest = css.as_str();
    while let Some(start) = rest.find("url(fonts/") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 4..];
        let end = after.find(')').expect("malformed url(...) in katex.min.css");
        let path = &after[..end];
        if path.ends_with(".woff2") {
            let bytes = fs::read(format!("assets/katex/{}", path))
                .unwrap_or_else(|e| panic!("read {}: {}", path, e));
            out.push_str("url(data:font/woff2;base64,");
            out.push_str(&b64encode(&bytes));
            out.push(')');
        } else {
            out.push_str(&rest[start..start + 4 + end + 1]);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    fs::write(Path::new(&out_dir).join("katex.inlined.css"), out)
        .expect("write katex.inlined.css");
}

fn b64encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = (u32::from(bytes[i]) << 16)
            | (u32::from(bytes[i + 1]) << 8)
            | u32::from(bytes[i + 2]);
        s.push(T[((n >> 18) & 0x3f) as usize] as char);
        s.push(T[((n >> 12) & 0x3f) as usize] as char);
        s.push(T[((n >> 6) & 0x3f) as usize] as char);
        s.push(T[(n & 0x3f) as usize] as char);
        i += 3;
    }
    match bytes.len() - i {
        1 => {
            let n = u32::from(bytes[i]) << 16;
            s.push(T[((n >> 18) & 0x3f) as usize] as char);
            s.push(T[((n >> 12) & 0x3f) as usize] as char);
            s.push_str("==");
        }
        2 => {
            let n = (u32::from(bytes[i]) << 16) | (u32::from(bytes[i + 1]) << 8);
            s.push(T[((n >> 18) & 0x3f) as usize] as char);
            s.push(T[((n >> 12) & 0x3f) as usize] as char);
            s.push(T[((n >> 6) & 0x3f) as usize] as char);
            s.push('=');
        }
        _ => {}
    }
    s
}
