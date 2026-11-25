#![cfg_attr(target_os = "none", no_std, no_main)]

#[cfg(not(target_os = "none"))]
fn main() {}

#[cfg(target_os = "none")]
mod app {
    use cmf_wp2_app::{Api, App, Batch, Event, RESUME, lv::*};
    use cmf_wp2_ui::stock::{self, Theme};
    use companion_rpc as rpc;

    const NOTE_URL: &str = "https://pastebin.com";
    const MAX_LEN: usize = 2048;

    cmf_wp2_app::entry!(NotesApp);
    cmf_wp2_app::app_icon!();

    struct NotesApp {
        api: &'static Api,
        buf: [u8; MAX_LEN],
        len: usize,
        req_id: u8,
        status: &'static str,
    }

    impl App for NotesApp {
        fn init(api: &'static Api) -> Self {
            let mut app = Self {
                api,
                buf: [0; MAX_LEN],
                len: 0,
                req_id: 1,
                status: "Sync",
            };
            let n = api.load(&mut app.buf);
            if n > 0 && n <= MAX_LEN {
                app.len = n;
            } else {
                let demo = b"Press Sync below to download notes.";
                app.buf[..demo.len()].copy_from_slice(demo);
                app.len = demo.len();
            }
            app
        }

        fn event(&mut self, ev: Event) {
            if ev == RESUME {
                self.draw();
            }
        }

        fn message(&mut self, m: &[u8]) {
            let mut body = [0u8; MAX_LEN];
            if let Ok(n) = rpc::decode_http_body(m, &mut body) {
                if n > 0 {
                    let take = n.min(MAX_LEN);
                    self.buf[..take].copy_from_slice(&body[..take]);
                    self.len = take;
                    self.api.save(&self.buf[..take]);
                    self.status = "Updated!";
                }
                self.draw();
            }
        }
    }

    impl NotesApp {
        fn sync(&mut self) {
            let mut out = [0u8; rpc::MSG_MAX];
            if let Ok(n) = rpc::encode_http_get(&mut out, self.req_id, NOTE_URL) {
                self.req_id = self.req_id.wrapping_add(1);
                if self.api.send(&out[..n]) >= 0 {
                    self.status = "Loading...";
                } else {
                    self.status = "No phone";
                }
            }
            self.draw();
        }

        fn draw(&mut self) {
            let mut b = Batch::new(self.api);
            let page = stock::page(&mut b, Theme::Dark);
            stock::title(&mut b, page, "Notes");

            let text = core::str::from_utf8(&self.buf[..self.len]).unwrap_or("Error");
            let lbl = b.label(page);
            b.set_text(lbl, text);
            b.set_width(lbl, 410);
            b.set_long_mode(lbl, LongMode::Wrap);

            let btn = stock::button(&mut b, page, self.status);
            b.on_click(btn, || {});
            b.flush();
        }
    }
}
