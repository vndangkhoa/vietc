// SPDX-License-Identifier: MIT
//! vietc-vk: standalone virtual-keyboard test tool for Viet+ (vietc).
//!
//! Creates a uinput virtual keyboard. vietc (already running, grabbing all
//! keyboards) intercepts the keystrokes, converts them per its VNI/Telex
//! config, and re-injects the result via its own (ignored) uinput device.
//! This mirrors vietc's integration test harness (daemon/tests/daemon_suite.rs):
//! type via virtual keyboard -> read clipboard -> assert.

mod clipboard;
mod dictionary;
mod virtual_keyboard;
mod vncode;

use eframe::egui;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

enum TestMsg {
    Case {
        label: String,
        pass: bool,
        detail: String,
    },
    Done {
        summary: String,
    },
}

struct VkApp {
    vk: Option<std::sync::Arc<std::sync::Mutex<virtual_keyboard::VirtualKeyboard>>>,
    vk_error: Option<String>,
    status: String,
    results: Vec<(String, bool, String)>,
    running: bool,
    method_filter: String,
    paragraph_method: String,
    test_rx: Option<std::sync::mpsc::Receiver<TestMsg>>,
}

impl VkApp {
    fn new() -> Self {
        let mut app = Self {
            vk: None,
            vk_error: None,
            status: String::new(),
            results: Vec::new(),
            running: false,
            method_filter: "all".to_string(),
            paragraph_method: "vni".to_string(),
            test_rx: None,
        };
        match virtual_keyboard::VirtualKeyboard::create("vietc-vk") {
            Ok(vk) => {
                app.vk = Some(std::sync::Arc::new(std::sync::Mutex::new(vk)));
                app.status = "Virtual keyboard ready. Start the vietc daemon AFTER this \
                     window opens so it grabs this device, then click Run self-test."
                    .into();
            }
            Err(e) => {
                app.vk_error = Some(format!(
                    "Failed to create virtual keyboard: {e}. Need /dev/uinput access \
                     (input group + udev rule 99-vietc.rules, or `setcap cap_dac_override+ep` \
                     on this binary)."
                ));
                app.status = "Virtual keyboard unavailable.".into();
            }
        }
        app
    }

    fn send_char(&mut self, ch: char) {
        if let Some(vk) = self.vk.as_ref() {
            if let Err(e) = vk.lock().unwrap().type_char(ch) {
                self.status = format!("type_char error: {e}");
            }
        }
    }

    fn run_self_test(&mut self) {
        let vk = match self.vk.clone() {
            Some(vk) => vk,
            None => {
                self.status = "No virtual keyboard; cannot run self-test.".into();
                return;
            }
        };
        self.running = true;
        self.results.clear();
        let (tx, rx) = std::sync::mpsc::channel();
        self.test_rx = Some(rx);
        let filter = self.method_filter.clone();

        std::thread::spawn(move || {
            let cases = dictionary::cases();
            let mut passed = 0usize;
            let mut total = 0usize;

            let mut vk = vk.lock().unwrap();
            for c in &cases {
                if filter != "all" && filter != c.method {
                    continue;
                }
                total += 1;

                clipboard::clear_clipboard();
                std::thread::sleep(Duration::from_millis(120));

                if let Err(e) = vk.type_text(c.input) {
                    let _ = tx.send(TestMsg::Case {
                        label: format!("{}: '{}'", c.method, c.input),
                        pass: false,
                        detail: format!("type error: {e}"),
                    });
                    continue;
                }

                std::thread::sleep(Duration::from_millis(1500));
                let got = clipboard::read_clipboard().unwrap_or_default();
                let pass = got.contains(c.expected) || got.trim() == c.expected.trim();
                if pass {
                    passed += 1;
                }
                let detail = if pass {
                    format!("OK -> '{}'", got)
                } else {
                    format!("expected '{}', got '{}'", c.expected, got)
                };
                let _ = tx.send(TestMsg::Case {
                    label: format!("{}: '{}'", c.method, c.input),
                    pass,
                    detail,
                });
            }

            drop(vk);
            let _ = tx.send(TestMsg::Done {
                summary: format!("Self-test done: {}/{} passed", passed, total),
            });
        });
    }

    /// Type the full tortoise-and-hare paragraph (encoded to the selected method)
    /// virtual keyboard and verify vietc composes it correctly by reading the
    /// clipboard. This is the end-to-end "input test". Whitespace is normalized
    /// on both sides so newlines/spaces don't cause false failures.
    fn run_paragraph_test(&mut self) {
        let vk = match self.vk.clone() {
            Some(vk) => vk,
            None => {
                self.status = "No virtual keyboard; cannot run paragraph test.".into();
                return;
            }
        };
        self.running = true;
        self.results.clear();
        let (tx, rx) = std::sync::mpsc::channel();
        self.test_rx = Some(rx);

        let paragraph_method = self.paragraph_method.clone();
        std::thread::spawn(move || {
            // Encode to the selected method; flatten newlines to spaces so vietc
            // flushes each line as plain text. A trailing space forces the final
            // word to flush.
            let method = paragraph_method;
            let telex: String = vncode::to_viet(&method, vncode::PARAGRAPH)
                .chars()
                .map(|c| if c == '\n' { ' ' } else { c })
                .collect();
            let telex = format!("{telex} ");

            clipboard::clear_clipboard();
            std::thread::sleep(Duration::from_millis(200));

            {
                let mut vk = vk.lock().unwrap();
                for ch in telex.chars() {
                    if let Err(e) = vk.type_char(ch) {
                        let _ = tx.send(TestMsg::Case {
                            label: "paragraph".into(),
                            pass: false,
                            detail: format!("type error: {e}"),
                        });
                        let _ = tx.send(TestMsg::Done {
                            summary: "Paragraph test failed".into(),
                        });
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(3));
                }
            }

            // Give vietc time to process and flush the composed text to clipboard.
            std::thread::sleep(Duration::from_millis(2500));
            let got = clipboard::read_clipboard().unwrap_or_default();

            let norm = |s: &str| -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") };
            let expected = norm(vncode::PARAGRAPH);
            let got_n = norm(&got);
            let pass = got_n == expected;
            let detail = if pass {
                "OK: paragraph composed correctly through vietc".into()
            } else {
                format!(
                    "expected ({} chars): '{}'\ngot ({} chars): '{}'",
                    expected.chars().count(),
                    expected,
                    got_n.chars().count(),
                    got_n
                )
            };
            let _ = tx.send(TestMsg::Case {
                label: "paragraph roundtrip".into(),
                pass,
                detail,
            });
            let _ = tx.send(TestMsg::Done {
                summary: if pass {
                    "Paragraph test PASSED".into()
                } else {
                    "Paragraph test FAILED".into()
                },
            });
        });
    }
}

impl eframe::App for VkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain self-test messages from the worker thread.
        if let Some(rx) = self.test_rx.as_ref() {
            loop {
                match rx.try_recv() {
                    Ok(TestMsg::Case { label, pass, detail }) => {
                        self.results.push((label, pass, detail));
                    }
                    Ok(TestMsg::Done { summary, .. }) => {
                        self.running = false;
                        self.status = summary;
                        self.test_rx = None;
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.running = false;
                        self.test_rx = None;
                        break;
                    }
                }
            }
            if self.running {
                ctx.request_repaint();
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("vietc-vk — virtual keyboard test for Viet+");
            ui.label(&self.status);
            if let Some(err) = &self.vk_error {
                ui.colored_label(egui::Color32::RED, err);
            }
        });

        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.label("Method filter:");
            ui.radio_value(&mut self.method_filter, "all".to_string(), "All");
            ui.radio_value(&mut self.method_filter, "vni".to_string(), "VNI");
            ui.radio_value(&mut self.method_filter, "telex".to_string(), "Telex");
            ui.separator();
            if ui.button("Run self-test").clicked() && !self.running {
                self.run_self_test();
            }
            if ui.button("Run paragraph test").clicked() && !self.running {
                self.run_paragraph_test();
            }
            ui.label("Paragraph method (must match vietc's input_method):");
            ui.radio_value(&mut self.paragraph_method, "vni".to_string(), "VNI");
            ui.radio_value(&mut self.paragraph_method, "telex".to_string(), "Telex");
            if self.running {
                ui.separator();
                ui.spinner();
                ui.label("Running...");
            }
            ui.separator();
            ui.label("Click keys at right to send keystrokes via the virtual keyboard. Focus any app (e.g. gedit) to watch vietc convert them.");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("On-screen keyboard (sends via uinput virtual keyboard):");
            ui.horizontal_wrapped(|ui| {
                for ch in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                    if ui.button(ch.to_string()).clicked() {
                        self.send_char(ch);
                    }
                }
                if ui.button("SPACE").clicked() {
                    self.send_char(' ');
                }
                if ui.button("ENTER").clicked() {
                    self.send_char('\n');
                }
                if ui.button(".").clicked() {
                    self.send_char('.');
                }
                if ui.button(",").clicked() {
                    self.send_char(',');
                }
            });
            ui.separator();
            ui.label("Self-test results:");
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.results.is_empty() {
                    ui.label("(none yet — click Run self-test)");
                }
                for (label, pass, detail) in &self.results {
                    let color = if *pass {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    };
                    ui.colored_label(
                        color,
                        format!("{}  [{}]", label, if *pass { "PASS" } else { "FAIL" }),
                    );
                    ui.label(detail);
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "vietc-vk",
        options,
        Box::new(|_cc| Box::new(VkApp::new())),
    )
}
