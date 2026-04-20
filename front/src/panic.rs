pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());

        let msg = format!("Panic at {}: {}", location, info);
        web_sys::console::error_1(&msg.into());
    }));
}
