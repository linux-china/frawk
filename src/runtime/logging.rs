use log::*;

#[ctor::ctor]
fn init() {
    env_logger::builder()
        .filter_module("cranelift_codegen", LevelFilter::Error)
        .filter_module("cranelift_jit", LevelFilter::Error)
        .filter_level(LevelFilter::Debug).init();
}

pub fn log_debug(target: &str, text: &str) {
    debug!(target: target, "{}", text);
}

pub fn log_info(target: &str, text: &str) {
    info!(target: target, "{}", text);
}

pub fn log_warn(target: &str, text: &str) {
    warn!(target: target, "{}", text);
}

pub fn log_error(target: &str, text: &str) {
    error!(target: target, "{}", text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        log_debug("","Hello");
    }
}