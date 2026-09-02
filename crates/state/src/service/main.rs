//! 云侧独立状态服务入口。`cargo run -p openjiuwen-state --features service --bin openjiuwen-state-service`

fn main() {
    openjiuwen_state::service::serve_placeholder();
    eprintln!("openjiuwen-state-service: skeleton (gRPC not wired yet)");
}
